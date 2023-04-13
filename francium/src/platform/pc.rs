use crate::arch::msr;
use crate::drivers::pc_io_apic::IoApic;
use crate::drivers::pc_local_apic::LocalApic;
use crate::drivers::pc_uart::COMPort;
use crate::drivers::pit_timer::PIT;
use crate::drivers::Timer;
use crate::drivers::{InterruptController, InterruptDistributor};
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

    let mut controller_lock = INTERRUPT_CONTROLLER.lock();
    let mut distributor_lock = INTERRUPT_DISTRIBUTOR.lock();
    controller_lock.init();
    distributor_lock.init();
    distributor_lock.enable_interrupt(timer_irq);

    // enable arch timer, 100hz
    let mut timer_lock = DEFAULT_TIMER.lock();
    timer_lock.set_period_us(10000);
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
