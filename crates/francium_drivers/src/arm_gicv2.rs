use tock_registers::interfaces::*;
use tock_registers::register_structs;
use tock_registers::registers::*;

use crate::{InterruptController, InterruptDistributor};

register_structs! {
    Gicv2DistributorRegs {
        ( 0x000 => gicd_ctlr: ReadWrite<u32> ),
        ( 0x004 => gicd_typer: ReadOnly<u32> ),
        ( 0x008 => gicd_iidr: ReadOnly<u32> ),
        ( 0x00c => _reserved ),
        ( 0x080 => gicd_igroupr: ReadWrite<u32> ),
        ( 0x084 => _reserved2 ),
        ( 0x100 => gicd_isenabler: [ReadWrite<u32>; 32] ),
        ( 0x180 => gicd_icenabler: [ReadWrite<u32>; 32] ),
        ( 0x200 => gicd_ispendr: [ReadWrite<u32>; 32] ),
        ( 0x280 => gicd_icpendr: [ReadWrite<u32>; 32] ),
        ( 0x300 => gicd_isactiver: [ReadWrite<u32>; 32] ),
        ( 0x380 => gicd_icactiver: [ReadWrite<u32>; 32] ),
        ( 0x400 => gicd_ipriorityr: [ReadWrite<u32>; 255] ),
        ( 0x7fc => _reserved3 ),
        ( 0x800 => gicd_itargetsr: [ReadWrite<u32>; 255] ),
        ( 0xbfc => _reserved4 ),
        ( 0xc00 => gicd_icfgr: [ReadWrite<u32>; 64] ),
        ( 0xd00 => _reserved5 ),
        ( 0xe00 => gicd_nsacr: [ReadWrite<u32>; 64] ),
        ( 0xf00 => gicd_sgir: WriteOnly<u32> ),
        ( 0xf04 => _reserved6 ),
        ( 0xf10 => gicd_cpendsgir: [ReadWrite<u32>; 4] ),
        ( 0xf20 => gicd_spendsgir: [ReadWrite<u32>; 4] ),
        ( 0xf30 => _reserved7 ),
        ( 0x1000 => @END ),
    }
}

register_structs! {
    Gicv2CpuRegs {
        (0x0000 => gicc_ctlr: ReadWrite<u32>),
        (0x0004 => gicc_pmr: ReadWrite<u32>),
        (0x0008 => gicc_bpr: ReadWrite<u32>),
        (0x000C => gicc_iar: ReadOnly<u32>),
        (0x0010 => gicc_eoir: WriteOnly<u32>),
        (0x0014 => gicc_rpr: ReadOnly<u32>),
        (0x0018 => gicc_hppir: ReadOnly<u32>),
        (0x001C => gicc_abpr: ReadWrite<u32>),
        (0x0020 => gicc_aiar: ReadOnly<u32>),
        (0x0024 => gicc_aeoir: WriteOnly<u32>),
        (0x0028 => gicc_ahppir: ReadOnly<u32>),
        (0x002C => _reserved),
        (0x00D0 => gicc_apr: [ReadWrite<u32>; 3]),
        (0x00DC => _reserved2),
        (0x00E0 => gicc_nsapr: [ReadWrite<u32>; 3]),
        (0x00EC => _reserved3),
        (0x00FC => gicc_iidr: ReadOnly<u32>),
        (0x0100 => _reserved4),
        (0x1000 => gicc_dir: WriteOnly<u32>),
        (0x1004 => @END),
    }
}

pub struct Gicv2Distributor {
    regs: &'static mut Gicv2DistributorRegs,
}

pub struct Gicv2Cpu {
    regs: &'static mut Gicv2CpuRegs,
}

impl Gicv2Distributor {
    pub fn new(gicd_base: usize) -> Gicv2Distributor {
        Gicv2Distributor {
            regs: unsafe { (gicd_base as *mut Gicv2DistributorRegs).as_mut().unwrap() },
        }
    }

    fn set_config(&mut self, interrupt: u32, is_level_triggered: bool) {
        // XXX: hella broken

        let bit = if is_level_triggered { 0 } else { 2 };

        let value = self.regs.gicd_icfgr[interrupt as usize / 16].get();
        let offset = (interrupt % 16) * 2;

        self.regs.gicd_icfgr[interrupt as usize / 16].set((value & !(3 << offset)) | bit << offset);
    }
}

impl Gicv2Cpu {
    pub fn new(gicc_base: usize) -> Gicv2Cpu {
        Gicv2Cpu {
            regs: unsafe { (gicc_base as *mut Gicv2CpuRegs).as_mut().unwrap() },
        }
    }
}

impl InterruptDistributor for Gicv2Distributor {
    fn init(&mut self) {
        self.regs.gicd_ctlr.set(1);

        // HACK: Virt PCIe interrupts are level triggered
        self.set_config(35, true);
        self.set_config(35, true);
        self.set_config(36, true);
        self.set_config(37, true);
    }

    fn enable_interrupt(&mut self, interrupt: u32) {
        self.regs.gicd_isenabler[interrupt as usize / 32].set(1 << (interrupt % 32));
    }

    fn disable_interrupt(&mut self, interrupt: u32) {
        self.regs.gicd_icenabler[interrupt as usize / 32].set(1 << (interrupt % 32));
    }
}

impl InterruptController for Gicv2Cpu {
    fn init(&mut self) {
        self.regs.gicc_ctlr.set(1);
        self.regs.gicc_pmr.set(0xff);
    }

    fn ack_interrupt(&mut self, interrupt: u32) {
        self.regs.gicc_eoir.set(interrupt);
    }

    // TODO: Correct value?
    const NUM_PENDING: u32 = 1;
    fn read_pending(&self, _i: u32) -> u32 {
        0
    }

    fn next_pending(&self) -> Option<u32> {
        let interrupt_num = self.regs.gicc_iar.get() & 0x3ff;

        if interrupt_num == 1023 {
            None
        } else {
            Some(interrupt_num)
        }
    }
}
