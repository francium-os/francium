use francium_common::types::{MapType, PagePermission};
use num_enum::TryFromPrimitive;
use process::syscalls;
use std::convert::TryInto;

#[derive(Debug, TryFromPrimitive)]
#[repr(u32)]
enum MailboxTag {
    AllocateBuffer = 0x00040001,
    ReleaseBuffer = 0x00048001,
    BlankScreen = 0x00040002,
    GetPhysicalSize = 0x00040003,
    TestPhysicalSize = 0x00044003,
    SetPhysicalSize = 0x00048003,
    GetVirtualSize = 0x00040004,
    TestVirtualSize = 0x00044004,
    SetVirtualSize = 0x00048004,
    GetDepth = 0x00040005,
    TestDepth = 0x00044005,
    SetDepth = 0x00048005,
    GetPixelOrder = 0x00040006, /* 0: BGR, 1: RGB */
    TestPixelOrder = 0x00044006,
    SetPixelOrder = 0x00048006,
    GetAlphaMode = 0x00040007,
    TestAlphaMode = 0x00044007,
    SetAlphaMode = 0x00048007,
    GetPitch = 0x00040008,
    GetVirtualOffset = 0x00040009,
    TestVirtualOffset = 0x00044009,
    SetVirtualOffset = 0x00048009,
    GetOverscan = 0x0004000a,
    TestOverscan = 0x0004400a,
    SetOverscan = 0x0004800a,
    GetPalette = 0x0004000b,
    TestPalette = 0x0004400b,
    SetPalette = 0x0004800b,
    SetCursorInfo = 0x00008010,
    SetCursorState = 0x00008011,
    SetScreenGamma = 0x00008012, /* Pi3 only? */
}

#[derive(Debug)]
#[repr(u32)]
#[allow(dead_code)]
enum MailboxRequest {
    AllocateBuffer(u32) = MailboxTag::AllocateBuffer as u32,
    ReleaseBuffer = MailboxTag::ReleaseBuffer as u32,
    BlankScreen(u32) = MailboxTag::BlankScreen as u32,
    GetPhysicalSize = MailboxTag::GetPhysicalSize as u32,
    TestPhysicalSize(u32, u32) = MailboxTag::TestPhysicalSize as u32,
    SetPhysicalSize(u32, u32) = MailboxTag::SetPhysicalSize as u32,
    GetVirtualSize = MailboxTag::GetVirtualSize as u32,
    TestVirtualSize(u32, u32) = MailboxTag::TestVirtualSize as u32,
    SetVirtualSize(u32, u32) = MailboxTag::SetVirtualSize as u32,
    GetDepth = MailboxTag::GetDepth as u32,
    TestDepth(u32) = MailboxTag::TestDepth as u32,
    SetDepth(u32) = MailboxTag::SetDepth as u32,
    GetPixelOrder = MailboxTag::GetPixelOrder as u32, /* 0: BGR, 1: RGB */
    TestPixelOrder(u32) = MailboxTag::TestPixelOrder as u32,
    SetPixelOrder(u32) = MailboxTag::SetPixelOrder as u32,
    GetAlphaMode = MailboxTag::GetAlphaMode as u32,
    TestAlphaMode(u32) = MailboxTag::TestAlphaMode as u32,
    SetAlphaMode(u32) = MailboxTag::SetAlphaMode as u32,
    GetPitch = MailboxTag::GetPitch as u32,
    GetVirtualOffset = MailboxTag::GetVirtualOffset as u32,
    TestVirtualOffset(u32, u32) = MailboxTag::TestVirtualOffset as u32,
    SetVirtualOffset(u32, u32) = MailboxTag::SetVirtualOffset as u32,
    GetOverscan = MailboxTag::GetOverscan as u32,
    TestOverscan(u32, u32, u32, u32) = MailboxTag::TestOverscan as u32,
    SetOverscan(u32, u32, u32, u32) = MailboxTag::SetOverscan as u32,
    GetPalette = MailboxTag::GetPalette as u32, /* TODO: these are bad */
    TestPalette = MailboxTag::TestPalette as u32, // etc
    SetPalette = MailboxTag::SetPalette as u32, // etc
    SetCursorInfo(u32, u32, u32, u32, u32, u32) = MailboxTag::SetCursorInfo as u32,
    SetCursorState(u32, u32, u32, u32) = MailboxTag::SetCursorState as u32,
    SetScreenGamma(u32, u32) = MailboxTag::SetScreenGamma as u32, // todo: cursed
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
#[allow(dead_code)]
enum MailboxReply {
    AllocateBuffer(u32, u32),
    ReleaseBuffer,
    BlankScreen(u32),
    GetPhysicalSize(u32, u32),
    TestPhysicalSize(u32, u32),
    SetPhysicalSize(u32, u32),
    GetVirtualSize(u32, u32),
    TestVirtualSize(u32, u32),
    SetVirtualSize(u32, u32),
    GetDepth(u32),
    TestDepth(u32),
    SetDepth(u32),
    GetPixelOrder(u32),
    TestPixelOrder(u32),
    SetPixelOrder(u32),
    GetAlphaMode(u32),
    TestAlphaMode(u32),
    SetAlphaMode(u32),
    GetPitch(u32),
    GetVirtualOffset(u32, u32),
    TestVirtualOffset(u32, u32),
    SetVirtualOffset(u32, u32),
    GetOverscan(u32, u32, u32, u32),
    TestOverscan(u32, u32, u32, u32),
    SetOverscan(u32, u32, u32, u32),
    GetPalette,  // todo: cursed
    TestPalette, // etc
    SetPalette,  // etc
    SetCursorInfo(u32),
    SetCursorState(u32),
    SetScreenGamma,
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

        unsafe {
            while (self.read_ch1_status() & MBOX_FULL) == MBOX_FULL {}

            self.write_ch1_address(r);

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
            if r != a {
                panic!("Got a reply for a different channel? {:x} != {:x}", r, a);
            }

            // just because - happens if pi3 is tried on pi4
            if a == 0 {
                panic!("Got zero - this shouldn't happen!");
            }
        }
        Some(())
    }
}

pub struct MailboxAdapter {
    mailbox: Mailbox,
    mailbox_buffer_virt: &'static mut [u8],
    mailbox_buffer_phys: usize,

    framebuffer_virt: usize,
    framebuffer_size: usize,
    current_x: usize,
    current_y: usize,
    pitch: usize,
}

struct MailboxMessage<'a> {
    buffer: &'a mut [u8],
    read_offset: usize,
    write_offset: usize,
}

impl MailboxMessage<'_> {
    pub fn new(buffer: &mut [u8]) -> MailboxMessage {
        MailboxMessage {
            buffer: buffer,
            read_offset: 0,
            write_offset: 0,
        }
    }

    pub fn write_u32(&mut self, val: u32) {
        self.buffer[self.write_offset..self.write_offset + 4]
            .copy_from_slice(&u32::to_le_bytes(val));
        self.write_offset += 4;
    }

    pub fn write_u32_at(&mut self, custom_off: usize, val: u32) {
        self.buffer[custom_off..custom_off + 4].copy_from_slice(&u32::to_le_bytes(val));
    }

    pub fn read_u32(&mut self) -> u32 {
        let val = u32::from_le_bytes(
            self.buffer[self.read_offset..self.read_offset + 4]
                .try_into()
                .unwrap(),
        );
        self.read_offset += 4;
        val
    }

    #[allow(dead_code,unused_variables)]
    pub fn send(&mut self, requests: &[MailboxRequest]) {
        // Length placeholder
        self.write_u32(0);
        // Request/Response
        self.write_u32(0);

        // Tags
        for tag in requests {
            self.write_u32(tag.discriminant());

            let buffer_size_offset = self.write_offset;
            // Filled later
            self.write_offset += 4;

            // Request code
            self.write_u32(0);

            match tag {
                MailboxRequest::AllocateBuffer(alignment) => {
                    // request: 4, reply: 8
                    self.write_u32(*alignment);
                    self.write_u32(0);
                }
                MailboxRequest::ReleaseBuffer => {
                    // 0, 0
                }
                MailboxRequest::GetPhysicalSize
                | MailboxRequest::GetVirtualSize
                | MailboxRequest::GetVirtualOffset => {
                    // request: 0, reply: 8
                    self.write_u32(0);
                    self.write_u32(0);
                }
                MailboxRequest::TestPhysicalSize(x, y)
                | MailboxRequest::SetPhysicalSize(x, y)
                | MailboxRequest::TestVirtualSize(x, y)
                | MailboxRequest::SetVirtualSize(x, y)
                | MailboxRequest::TestVirtualOffset(x, y)
                | MailboxRequest::SetVirtualOffset(x, y) => {
                    // request: 8, reply: 8
                    self.write_u32(*x);
                    self.write_u32(*y);
                }
                MailboxRequest::GetDepth
                | MailboxRequest::GetPixelOrder
                | MailboxRequest::GetAlphaMode
                | MailboxRequest::GetPitch => {
                    // request: 0, reply: 4
                    self.write_u32(0);
                }
                MailboxRequest::TestDepth(val)
                | MailboxRequest::SetDepth(val)
                | MailboxRequest::TestPixelOrder(val)
                | MailboxRequest::SetPixelOrder(val)
                | MailboxRequest::TestAlphaMode(val)
                | MailboxRequest::SetAlphaMode(val)
                | MailboxRequest::BlankScreen(val) => {
                    // request: 4, reply: 4
                    self.write_u32(*val);
                }
                MailboxRequest::GetOverscan => {
                    // request: 0, reply: 16
                    unimplemented!();
                }
                MailboxRequest::TestOverscan(a, b, c, d) => {
                    unimplemented!()
                }
                MailboxRequest::SetOverscan(a, b, c, d) => {
                    unimplemented!()
                }

                MailboxRequest::SetCursorInfo(width, height, unk, ptr, hotspot_x, hotspot_y) => {
                    unimplemented!()
                }
                MailboxRequest::SetCursorState(enable, x, y, flags) => {
                    unimplemented!()
                }
                MailboxRequest::SetScreenGamma(display, table_ptr) => {
                    unimplemented!()
                }

                MailboxRequest::GetPalette
                | MailboxRequest::TestPalette
                | MailboxRequest::SetPalette => {
                    unimplemented!();
                }
            }

            let tag_data_length = self.write_offset as u32 - buffer_size_offset as u32 - 8;
            //println!("{:?}", tag_data_length);
            self.write_u32_at(buffer_size_offset, tag_data_length);
        }

        // Write a zero
        self.write_u32(0);

        assert!(self.write_offset % 4 == 0);

        // Finally: write length
        self.write_u32_at(0, self.write_offset as u32);
    }

    pub fn recv(&mut self) -> Vec<MailboxReply> {
        let mut replies = Vec::new();

        let _buffer_size = self.read_u32();
        let buffer_response_code = self.read_u32();
        if buffer_response_code != 0x80000000 {
            panic!(
                "Invalid response code, expected 0x80000000 got {:x}",
                buffer_response_code
            );
        }

        loop {
            let tag = self.read_u32();
            if tag == 0 {
                break;
            }

            let tag = tag.try_into().unwrap();

            let buffer_size = self.read_u32();
            let response_code = self.read_u32();
            if (response_code & 0x80000000) == 0 {
                panic!("Tag {:?} not responded to!", tag);
            }

            // Make sure the GPU firmware didn't suddenly grow more fields or something.
            assert!(buffer_size == response_code & 0x7fffffff);

            let reply = match tag {
                MailboxTag::AllocateBuffer => {
                    MailboxReply::AllocateBuffer(self.read_u32(), self.read_u32())
                }
                MailboxTag::ReleaseBuffer => MailboxReply::ReleaseBuffer,
                MailboxTag::BlankScreen => MailboxReply::BlankScreen(self.read_u32()),
                MailboxTag::GetPhysicalSize => {
                    MailboxReply::GetPhysicalSize(self.read_u32(), self.read_u32())
                }
                MailboxTag::TestPhysicalSize => {
                    MailboxReply::TestPhysicalSize(self.read_u32(), self.read_u32())
                }
                MailboxTag::SetPhysicalSize => {
                    MailboxReply::SetPhysicalSize(self.read_u32(), self.read_u32())
                }
                MailboxTag::GetVirtualSize => {
                    MailboxReply::GetVirtualSize(self.read_u32(), self.read_u32())
                }
                MailboxTag::TestVirtualSize => {
                    MailboxReply::TestVirtualSize(self.read_u32(), self.read_u32())
                }
                MailboxTag::SetVirtualSize => {
                    MailboxReply::SetVirtualSize(self.read_u32(), self.read_u32())
                }
                MailboxTag::GetDepth => MailboxReply::GetDepth(self.read_u32()),
                MailboxTag::TestDepth => MailboxReply::TestDepth(self.read_u32()),
                MailboxTag::SetDepth => MailboxReply::SetDepth(self.read_u32()),
                MailboxTag::GetPixelOrder => MailboxReply::GetPixelOrder(self.read_u32()),
                MailboxTag::TestPixelOrder => MailboxReply::TestPixelOrder(self.read_u32()),
                MailboxTag::SetPixelOrder => MailboxReply::SetPixelOrder(self.read_u32()),
                MailboxTag::GetAlphaMode => MailboxReply::GetAlphaMode(self.read_u32()),
                MailboxTag::TestAlphaMode => MailboxReply::TestAlphaMode(self.read_u32()),
                MailboxTag::SetAlphaMode => MailboxReply::SetAlphaMode(self.read_u32()),
                MailboxTag::GetPitch => MailboxReply::GetPitch(self.read_u32()),
                MailboxTag::GetVirtualOffset => {
                    MailboxReply::GetVirtualOffset(self.read_u32(), self.read_u32())
                }
                MailboxTag::TestVirtualOffset => {
                    MailboxReply::TestVirtualOffset(self.read_u32(), self.read_u32())
                }
                MailboxTag::SetVirtualOffset => {
                    MailboxReply::SetVirtualOffset(self.read_u32(), self.read_u32())
                }
                MailboxTag::GetOverscan => MailboxReply::GetOverscan(
                    self.read_u32(),
                    self.read_u32(),
                    self.read_u32(),
                    self.read_u32(),
                ),
                MailboxTag::TestOverscan => MailboxReply::TestOverscan(
                    self.read_u32(),
                    self.read_u32(),
                    self.read_u32(),
                    self.read_u32(),
                ),
                MailboxTag::SetOverscan => MailboxReply::SetOverscan(
                    self.read_u32(),
                    self.read_u32(),
                    self.read_u32(),
                    self.read_u32(),
                ),
                MailboxTag::GetPalette => unimplemented!(),
                MailboxTag::TestPalette => unimplemented!(),
                MailboxTag::SetPalette => unimplemented!(),
                MailboxTag::SetCursorInfo => MailboxReply::SetCursorInfo(self.read_u32()),
                MailboxTag::SetCursorState => MailboxReply::SetCursorState(self.read_u32()),
                MailboxTag::SetScreenGamma => MailboxReply::SetScreenGamma,
            };

            replies.push(reply);
        }

        replies
    }
}

// For DMA: clear to point of coherency, NOT point of unification
pub unsafe fn clear_cache_for_address(addr: usize) {
    std::arch::asm!("dc civac, {addr}", addr = in (reg) (addr));
}

impl<'a> MailboxAdapter {
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

            framebuffer_virt: 0,
            framebuffer_size: 0,
            current_x: 0,
            current_y: 0,
            pitch: 0,
        };
        adapter
    }

    fn send_mailbox_messages(&mut self, requests: &[MailboxRequest]) -> Vec<MailboxReply> {
        let mut msg = MailboxMessage::new(self.mailbox_buffer_virt);
        msg.send(requests);

        // as linux: wmb
        unsafe {
            std::arch::asm!("dmb ishst");
        }

        // Unclear if we need to actually _clear_ the caches. but. w/e
        for i in (0..4096).step_by(64) {
            unsafe {
                clear_cache_for_address((msg.buffer.as_ptr() as usize) + i);
            }
        }

        if let None = self.mailbox.send(self.mailbox_buffer_phys) {
            // TODO: cry
            panic!("Firmware is angy");
        }

        // as linux: rmb
        unsafe {
            std::arch::asm!("dmb ishld");
        }

        msg.recv()
    }

    pub fn set_mode(&mut self, x: usize, y: usize) {
        let replies = self.send_mailbox_messages(&[
            MailboxRequest::SetPhysicalSize(x as u32, y as u32),
            MailboxRequest::SetVirtualSize(x as u32, y as u32),
            MailboxRequest::SetVirtualOffset(0, 0),
            MailboxRequest::SetDepth(32),
            MailboxRequest::SetPixelOrder(1),
            MailboxRequest::AllocateBuffer(16),
            MailboxRequest::GetPitch,
        ]);
        println!("{:?}", replies);

        for r in replies {
            match r {
                MailboxReply::AllocateBuffer(fb_base, fb_size) => {
                    let arm_fb_base = fb_base & 0x3fffffff;
                    self.framebuffer_virt = syscalls::map_device_memory(
                        arm_fb_base as usize,
                        0,
                        fb_size as usize,
                        MapType::NormalUncachable,
                        PagePermission::USER_READ_WRITE,
                    )
                    .unwrap();
                    self.framebuffer_size = fb_size as usize;
                }
                MailboxReply::SetPhysicalSize(x, y) => {
                    self.current_x = x as usize;
                    self.current_y = y as usize;
                }
                MailboxReply::GetPitch(pitch) => {
                    self.pitch = pitch as usize;
                }
                _ => {}
            }
        }
    }

    pub fn get_framebuffer(&self) -> &'a mut [u32] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.framebuffer_virt as *mut u32,
                (self.pitch / 4) * self.current_y,
            )
        }
    }
}

/*
https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface
https://forums.raspberrypi.com/viewtopic.php?t=250591
https://www.rpi4os.com/part5-framebuffer/

Where is the mailbox base documented??? bus address 7E00B880 = 3F00B880
*/
