use alloc::alloc::{Layout, alloc_zeroed};
use alloc::vec::Vec;
use crate::arch::msr;
use crate::drivers::pc_io_apic::IoApic;
use crate::drivers::pc_local_apic::LocalApic;
use crate::drivers::pc_uart::COMPort;
use crate::drivers::pit_timer::PIT;
use crate::drivers::Timer;
use crate::drivers::{InterruptController, InterruptDistributor};
use crate::mmu;
use acpi::platform::ProcessorState::WaitingForSipi;
use core::arch::asm;
use francium_common::types::PhysAddr;
use spin::Mutex;

pub const PHYS_MEM_BASE: usize = 0;
pub const PHYS_MEM_SIZE: usize = 0x80000000; // 2gb?? for now

unsafe fn turn_on_floating_point() {
    asm!(
        "
		mov rax, cr0
		and ax, 0xFFFB
		or ax, 0x2
		mov cr0, rax
		mov rax, cr4
		or ax, 3 << 9
		mov cr4, rax
	"
    );
}

use acpi::{AcpiHandler, AcpiTables, PhysicalMapping};
use core::ptr::NonNull;

#[derive(Copy, Clone)]
struct FranciumACPIHandler {}
impl AcpiHandler for FranciumACPIHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        PhysicalMapping::new(
            physical_address,
            NonNull::new(crate::mmu::phys_to_virt(PhysAddr(physical_address)) as *mut T).unwrap(),
            size,
            size,
            *self,
        )
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {}
}

lazy_static! {
    pub static ref DEFAULT_UART: Mutex<COMPort> = Mutex::new(COMPort::new(0x3f8));
    pub static ref DEFAULT_TIMER: Mutex<PIT> = Mutex::new(PIT::new());

    //pub static ref INTERRUPT_CONTROLLER: Mutex<PIC> = Mutex::new(PIC::new());
    //pub static ref INTERRUPT_DISTRIBUTOR: Mutex<PICDist> = Mutex::new(PICDist::new());

    pub static ref INTERRUPT_CONTROLLER: Mutex<LocalApic> = {
        if let acpi::platform::interrupt::InterruptModel::Apic(apic_model) = &PLATFORM_INFO.interrupt_model {
            Mutex::new(LocalApic::new(crate::constants::PERIPHERAL_BASE + apic_model.local_apic_address as usize))
        } else {
            panic!("No apic?");
        }
    };

    pub static ref INTERRUPT_DISTRIBUTOR: Mutex<IoApic> = {
        if let acpi::platform::interrupt::InterruptModel::Apic(apic_model) = &PLATFORM_INFO.interrupt_model {
            assert!(apic_model.io_apics.len() == 1);
            Mutex::new(IoApic::new(crate::constants::PERIPHERAL_BASE + apic_model.io_apics[0].address as usize))
        } else {
            panic!("No apic?");
        }
    };

    pub static ref PLATFORM_INFO: acpi::PlatformInfo<'static, alloc::alloc::Global> = {
        let rsdp_addr = unsafe {
            crate::arch::x86_64::info::SYSTEM_INFO_RSDP_ADDR.unwrap()
        };

        let handler = FranciumACPIHandler {};
        let tables = unsafe { AcpiTables::from_rsdp(handler, rsdp_addr as usize).unwrap() };
        acpi::platform::PlatformInfo::new_in(&tables, &alloc::alloc::Global).unwrap()
    };
}

pub fn platform_specific_init() {}

pub fn scheduler_pre_init() {
    // enable timer irq
    let timer_irq = 2; // PIC on IRQ 2...

    log::debug!("before ... locking????");
    let mut controller_lock = INTERRUPT_CONTROLLER.lock();
    let mut distributor_lock = INTERRUPT_DISTRIBUTOR.lock();
    log::debug!("APIC init?");
    controller_lock.init();
    log::debug!("APIC init?");
    distributor_lock.init();
    log::debug!("APIC init?");
    distributor_lock.enable_interrupt(timer_irq);

    // enable arch timer, 100hz
    let mut timer_lock = DEFAULT_TIMER.lock();
    log::debug!("timer init?");
    timer_lock.set_period_us(10000);
    log::debug!("timer init?");
    timer_lock.reset_timer();
}

pub fn scheduler_post_init() {
    unsafe {
        turn_on_floating_point();
    }

    // XXX give this a constant
    // Enable XN
    unsafe {
        msr::write_efer(msr::read_efer() | (1 << 11));
    }
}

extern "C" {
    fn ap_trampoline();
    #[link_name = "ap_trampoline_end"]
    static mut AP_TRAMPOLINE_END: u8;
    #[link_name = "__ap_stack_pointers"]
    static mut AP_STACK_POINTERS: *mut usize;
}
static mut AP_BOOTSTRAP_STACKS: Vec<usize> = Vec::new();

use crate::mmu::PageTable;
use francium_common::types::{PagePermission, MapType};
lazy_static! {
    static ref TRAMPOLINE_PAGETABLE: PageTable = {
        let mut pg = crate::KERNEL_ADDRESS_SPACE.read().page_table.user_process();
        pg.map_4k(PhysAddr(0x8000), 0x8000, PagePermission::KERNEL_RWX, MapType::NormalCachable);
        pg
    };
}

pub fn bringup_other_cpus() {
    let mut lapic = INTERRUPT_CONTROLLER.lock();

    // We need to write some trampoline code to the start of memory.
    let trampoline_ptr = mmu::phys_to_virt(PhysAddr(0x8000));

    unsafe {
        let ap_trampoline_start = ap_trampoline as *const u8 as usize;
        let ap_trampoline_end = &AP_TRAMPOLINE_END as *const u8 as usize;

        core::ptr::copy_nonoverlapping(
            ap_trampoline_start as *const u8,
            trampoline_ptr as *mut u8,
            ap_trampoline_end - ap_trampoline_start,
        );

        let kernel = crate::KERNEL_ADDRESS_SPACE.read();
        let cr3_phys = kernel.page_table.virt_to_phys(&*TRAMPOLINE_PAGETABLE as *const PageTable as usize).unwrap();
        ((trampoline_ptr + 0x18) as *mut usize).write_volatile(cr3_phys.0);
    }
    
    // TODO: Flush caches? maybe.

    let processor_info = PLATFORM_INFO.processor_info.as_ref().unwrap();

    unsafe {
        AP_BOOTSTRAP_STACKS.push(0);
    }

    for _ in processor_info.application_processors.iter() {
        unsafe {
            let cpu_stack = alloc_zeroed(Layout::from_size_align(0x1000, 64).unwrap()) as usize + 0x1000;
            AP_BOOTSTRAP_STACKS.push(cpu_stack);
        }
    }

    unsafe {
        AP_STACK_POINTERS = AP_BOOTSTRAP_STACKS.as_mut_ptr();
    }
    
    for ap in processor_info.application_processors.iter() {
        println!("{:?}", ap);
        assert!(ap.state == WaitingForSipi);
        // Bochs wants an init IPI first.
        lapic.send_init_ipi(ap.local_apic_id);
        lapic.send_sipi(ap.local_apic_id);
    }
}

pub fn get_cpu_count() -> usize {
    let processor_info = PLATFORM_INFO.processor_info.as_ref().unwrap();
    processor_info.application_processors.len() + 1
}