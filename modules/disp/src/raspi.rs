use francium_common::types::{MapType, PagePermission};
use process::syscalls;
use std::convert::TryInto;

#[derive(Debug)]
#[repr(u32)]
enum MailboxRequest {
    AllocateBuffer(u32) = 0x00040001,

    //	GetPhysicalSize = 0x00040003,
    SetPhysicalSize(u32, u32) = 0x00048003,

    SetVirtualSize(u32, u32) = 0x00048004,
    //	GetDepth = 0x00040005,
    SetDepth(u32) = 0x00048005,
    //	GetPitch = 0x00040008
}

impl MailboxRequest {
    fn discriminant(&self) -> u32 {
        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u32>() }
    }
}

#[derive(Debug)]
enum MailboxReply {
    /*	AllocateBuffer(u32, u32),
    SetPhysicalSize(u32, u32),
    SetVirtualSize(u32, u32),
    SetDepth(u32),
    GetPitch(u32),
    GetPhysicalSize(u32, u32),
    GetDepth(u32)*/
}

struct Mailbox {
    base_address: usize,
}

//const VIDEOCORE_MBOX: usize = 0x0000B880;
const MBOX_CH0_READ: usize = /*VIDEOCORE_MBOX +*/ 0x0;
//const MBOX_CH0_POLL:      usize = /*VIDEOCORE_MBOX +*/ 0x10;
//const MBOX_CH0_SENDER:    usize = /*VIDEOCORE_MBOX +*/ 0x14;
const MBOX_CH0_STATUS: usize = /*VIDEOCORE_MBOX +*/ 0x18;
//const MBOX_CH0_CONFIG:    usize = /*VIDEOCORE_MBOX +*/ 0x1C;
const MBOX_CH1_WRITE: usize = /*VIDEOCORE_MBOX +*/ 0x20;

const MBOX_CH1_STATUS: usize = /*VIDEOCORE_MBOX +*/ 0x38;

//const MBOX_RESPONSE:  u32   = 0x80000000;
const MBOX_FULL: u32 = 0x80000000;
const MBOX_EMPTY: u32 = 0x40000000;

// #define ARM_0_MAIL0	0x00
// #define ARM_0_MAIL1	0x20

/*
#define MAIL0_RD	(ARM_0_MAIL0 + 0x00)
#define MAIL0_POL	(ARM_0_MAIL0 + 0x10)
#define MAIL0_STA	(ARM_0_MAIL0 + 0x18)
#define MAIL0_CNF	(ARM_0_MAIL0 + 0x1C)
#define MAIL1_WRT	(ARM_0_MAIL1 + 0x00)
#define MAIL1_STA	(ARM_0_MAIL1 + 0x18)
*/

impl Mailbox {
    pub fn new(peripheral_base: usize) -> Mailbox {
        let mailbox_virt = syscalls::map_device_memory(
            peripheral_base + 0xb000,
            0,
            0x1000,
            MapType::Device,
            PagePermission::USER_READ_WRITE,
        )
        .unwrap()
            + 0x880;
        Mailbox {
            base_address: mailbox_virt,
        }
    }

    unsafe fn read_ch0_status(&self) -> u32 {
        ((self.base_address + MBOX_CH0_STATUS) as *mut u32).read_volatile()
    }

    unsafe fn read_ch1_status(&self) -> u32 {
        ((self.base_address + MBOX_CH1_STATUS) as *mut u32).read_volatile()
    }

    unsafe fn write_ch1_address(&self, addr: u32) {
        ((self.base_address + MBOX_CH1_WRITE) as *mut u32).write_volatile(addr)
    }

    unsafe fn read_ch0_address(&self) -> u32 {
        ((self.base_address + MBOX_CH0_READ) as *mut u32).read_volatile()
    }

    pub fn send(&mut self, buffer: usize) -> Option<()> {
        // Always use channel 8.
        // **With the exception of the property tags mailbox channel**, when passing memory addresses as the data part of a mailbox message, the addresses should be bus addresses as seen from the VC
        let r: u32 = (buffer as u32) | 8;
        println!("{:x}", r);

        unsafe {
            println!("ch1: {:x}", self.read_ch1_status());

            while (self.read_ch1_status() & MBOX_FULL) == MBOX_FULL {}

            self.write_ch1_address(r);

            // This _Should _be uncontested. Just in case.
            for _ in 0..5 {
                let mut got_reply = false;
                // Wait for a reply. This can take a bit longer. Wait up to 1 second.
                for _ in 0..20 {
                    if (self.read_ch0_status() & MBOX_EMPTY) != MBOX_EMPTY {
                        got_reply = true;
                        break;
                    }
                    syscalls::sleep_ns(50_000_000);
                }

                if !got_reply {
                    // pain
                    println!("firmware left us on read");
                    return None;
                }

                let a = self.read_ch0_address();
                // Is it a reply to our message?
                if r == a {
                    break;
                } else {
                    println!("Got a reply for a different channel? {:x} != {:x}", r, a);
                }

                // just because - happens if pi3 is tried on pi4
                if a == 0 {
                    panic!("Got zero - this shouldn't happen!");
                }

                println!("Helo");
            }
        }
        Some(())
    }
}

pub struct MailboxAdapter {
    mailbox: Mailbox,
    mailbox_buffer_virt: &'static mut [u8],
    mailbox_buffer_phys: usize,
}

struct MailboxMessage<'a> {
    buffer: &'a mut [u8],
    offset: usize,
}

impl MailboxMessage<'_> {
    pub fn new(buffer: &mut [u8]) -> MailboxMessage {
        MailboxMessage {
            buffer: buffer,
            offset: 0,
        }
    }

    pub fn write_u32(&mut self, val: u32) {
        self.buffer[self.offset..self.offset + 4].copy_from_slice(&u32::to_le_bytes(val));
        self.offset += 4;
    }

    pub fn write_u32_at(&mut self, custom_off: usize, val: u32) {
        self.buffer[custom_off..custom_off + 4].copy_from_slice(&u32::to_le_bytes(val));
    }

    pub fn send(&mut self, requests: &[MailboxRequest]) {
        // Length placeholder
        self.write_u32(0);
        // Request/Response
        self.write_u32(0);

        // Tags
        for tag in requests {
            self.write_u32(tag.discriminant());

            let buffer_size_offset = self.offset;
            // Filled later
            self.offset += 4;

            // Request code
            self.write_u32(0);

            match tag {
                MailboxRequest::SetDepth(one) => {
                    self.write_u32(*one);
                }
                MailboxRequest::AllocateBuffer(one) => {
                    self.write_u32(*one);
                    self.write_u32(0);
                }
                MailboxRequest::SetPhysicalSize(one, two)
                | MailboxRequest::SetVirtualSize(one, two) => {
                    self.write_u32(*one);
                    self.write_u32(*two);
                }
            }

            let tag_data_length = self.offset as u32 - buffer_size_offset as u32 - 8;
            //println!("{:?}", tag_data_length);
            self.write_u32_at(buffer_size_offset, tag_data_length);
        }

        // Write a zero
        self.write_u32(0);

        assert!(self.offset % 4 == 0);

        self.write_u32_at(0, self.offset as u32);

        for i in (0..self.offset).step_by(4) {
            print!(
                "0x{:x}, ",
                u32::from_le_bytes(self.buffer[i..i + 4].try_into().unwrap())
            );
        }
        println!();
    }

    pub fn recv(&self) -> Vec<MailboxReply> {
        println!("Recv:");
        for i in (0..self.offset).step_by(4) {
            print!(
                "0x{:x}, ",
                u32::from_le_bytes(self.buffer[i..i + 4].try_into().unwrap())
            );
        }
        println!();

        Vec::new()
    }
}

// For DMA: clear to point of coherency, NOT point of unification
pub unsafe fn clear_cache_for_address(addr: usize) {
    std::arch::asm!("dc cvac, {addr}", addr = in (reg) (addr));
    // Sledgehammer
    std::arch::asm!("isb sy; dsb sy");
}

impl MailboxAdapter {
    pub fn new(peripheral_base: usize) -> MailboxAdapter {
        let mailbox_buffer_virt =
            syscalls::map_memory(0, 0x1000, PagePermission::USER_READ_WRITE).unwrap();
        let mailbox_buffer_phys = syscalls::query_physical_address(mailbox_buffer_virt).unwrap();

        let adapter = MailboxAdapter {
            mailbox: Mailbox::new(peripheral_base),
            mailbox_buffer_virt: unsafe {
                core::slice::from_raw_parts_mut(mailbox_buffer_virt as *mut u8, 4096)
            },
            mailbox_buffer_phys: mailbox_buffer_phys,
        };
        adapter
    }

    fn send_mailbox_messages(&mut self, requests: &[MailboxRequest]) -> Vec<MailboxReply> {
        let mut msg = MailboxMessage::new(self.mailbox_buffer_virt);
        msg.send(requests);
        /* Flush cache */

        for i in (0..4096).step_by(64) {
            unsafe {
                clear_cache_for_address((msg.buffer.as_ptr() as usize) + i);
            }
        }

        if let None = self.mailbox.send(self.mailbox_buffer_phys) {
            // TODO: cry
            panic!("Firmware is angy");
        }

        msg.recv()
    }

    pub fn set_mode(&mut self, x: usize, y: usize) {
        let replies = self.send_mailbox_messages(&[
            MailboxRequest::AllocateBuffer(16),
            MailboxRequest::SetPhysicalSize(x as u32, y as u32),
            MailboxRequest::SetVirtualSize(x as u32, y as u32),
            MailboxRequest::SetDepth(24),
        ]);
        println!("{:?}", replies);
    }

    pub fn fill(&self) {}
}

/*
https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface
https://forums.raspberrypi.com/viewtopic.php?t=250591
https://www.rpi4os.com/part5-framebuffer/

Where is the mailbox base documented??? bus address 7E00B880 = 3F00B880
*/
