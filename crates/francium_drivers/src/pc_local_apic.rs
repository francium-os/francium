use tock_registers::register_structs;
use tock_registers::registers::*;
use tock_registers::interfaces::*;
use crate::InterruptController;

register_structs! {
	PaddedRegister {
		( 0 => reg: ReadOnly<u32> ),
		( 4 => _reserved ),
		( 0x10 => @END ),
	}
}

register_structs! {
    LocalApicRegs {
    	(0 => _reserved),

        (0x20 => id: ReadWrite<u32>),
        (0x24 => _reserved2),

        (0x30 => version: ReadOnly<u32>),
        (0x34 => _reserved3),

        (0x80 => task_priority: ReadWrite<u32>),
        (0x84 => _reserved4),

        (0x90 => arbitration_priority: ReadOnly<u32>),
 		(0x94 => _reserved5),

		(0xa0 => processor_priority: ReadOnly<u32>),
		(0xa4 => _reserved6),

		(0xb0 => end_of_interrupt: WriteOnly<u32>),
		(0xb4 => _reserved7),

		(0xc0 => remote_read: ReadOnly<u32>),
		(0xc4 => _reserved8),

		(0xd0 => logical_dest: ReadOnly<u32>),
		(0xd4 => _reserved9),

		(0xe0 => dest_format: ReadWrite<u32>),
		(0xe4 => _reserved10),

		(0xf0 => spurious_interrupt_vector: ReadWrite<u32>),
		(0xf4 => _reserved11),

		(0x100 => in_service: [PaddedRegister; 8]),
		(0x180 => trigger_mode: [PaddedRegister; 8]),
		(0x200 => interrupt_request: [PaddedRegister; 8]),

		(0x280 => error_status: ReadOnly<u32>),
		(0x284 => _reserved15),

		(0x2f0 => corrected_machine_check: ReadWrite<u32>),
		(0x2f4 => _reserved16),

		(0x300 => interrupt_command: ReadWrite<u32>),
		(0x304 => _reserved17),
		(0x310 => interrupt_command_upper: ReadWrite<u32>),
		(0x314 => _reserved18),

		(0x320 => lvt_timer: ReadWrite<u32>),
		(0x324 => _reserved19),
		(0x330 => lvt_thermal_sensor: ReadWrite<u32>),
		(0x334 => _reserved20),
		(0x340 => lvt_perf_counters: ReadWrite<u32>),
		(0x344 => _reserved21),
		(0x350 => lvt_lint0: ReadWrite<u32>),
		(0x354 => _reserved22),
		(0x360 => lvt_lint1: ReadWrite<u32>),
		(0x364 => _reserved23),
		(0x370 => lvt_error: ReadWrite<u32>),
		(0x374 => _reserved24),
		(0x380 => timer_initial_count: ReadWrite<u32>),
		(0x384 => _reserved25),
		(0x390 => timer_current_count: ReadOnly<u32>),
		(0x394 => _reserved26),

		(0x3e0 => timer_divide_config: ReadOnly<u32>),
		(0x3e4 => _reserved27),

        // The end of the struct is marked as follows.
        (0x400 => @END),
    }
}

pub struct LocalApic {
	regs: &'static mut LocalApicRegs
}

impl LocalApic {
	pub fn new(base_address_virt: usize) -> LocalApic {
		LocalApic {
			regs: unsafe { 
				(base_address_virt as *mut LocalApicRegs).as_mut().unwrap()
			}
		}
	}
}

use francium_x86::msr;

impl InterruptController for LocalApic {
	fn init(&mut self) {
		/* Ensure APIC is enabled. */
		unsafe {
			msr::write_apic_base(msr::read_apic_base() | 0x800);
		}

		/* Ensure PIC is disabled */
		crate::pic_interrupt_controller::disable_pic();

		self.regs.spurious_interrupt_vector.set(0x1ff);
	}

	fn ack_interrupt(&mut self, n: u32) {
		self.regs.end_of_interrupt.set(0);
	}

	const NUM_PENDING: u32 = 1;
	fn read_pending(&self, i: u32) -> u32 {
		// We can take this shortcut, because x86 has IRQ numbers.
		//unimplemented!();
		0
	}
}