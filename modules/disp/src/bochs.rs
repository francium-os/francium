use francium_common::types::{MapType, PagePermission};
use process::ipc;
use process::syscalls;

#[derive(Debug)]
pub struct BochsAdapter {
    framebuffer_virt: usize,
    io_virt: usize,

    current_x: usize,
    current_y: usize,
    pitch: usize,
}

const VBE_DISPI_INDEX_ID: usize = 0;
const VBE_DISPI_INDEX_XRES: usize = 1;
const VBE_DISPI_INDEX_YRES: usize = 2;
const VBE_DISPI_INDEX_BPP: usize = 3;
const VBE_DISPI_INDEX_ENABLE: usize = 4;
//const VBE_DISPI_INDEX_BANK: usize = 5;
const VBE_DISPI_INDEX_VIRT_WIDTH: usize = 6;
const VBE_DISPI_INDEX_VIRT_HEIGHT: usize = 7;
const VBE_DISPI_INDEX_X_OFFSET: usize = 8;
const VBE_DISPI_INDEX_Y_OFFSET: usize = 9;
const VBE_DISPI_INDEX_VIDEO_MEMORY_64K: usize = 0xa;

impl<'a> BochsAdapter {
    pub fn new() -> Option<BochsAdapter> {
        /* something something shared mem */
        let device_id = *ipc::pcie::get_devices_by_vidpid(0x1234, 0x1111).get(0)?;
        let framebuffer_bar = ipc::pcie::get_bar(device_id, 0).ok()?;
        let bochs_io_bar = ipc::pcie::get_bar(device_id, 2).ok()?;

        ipc::pcie::enable(device_id).unwrap();

        // TODO: Move this to be shared memory. But that requires the concept of shared memory.
        let fb_virt = syscalls::map_device_memory(
            framebuffer_bar.0,
            0,
            framebuffer_bar.1,
            MapType::NormalCachable,
            PagePermission::USER_READ_WRITE,
        )
        .unwrap();
        let io_virt = syscalls::map_device_memory(
            bochs_io_bar.0,
            0,
            bochs_io_bar.1,
            MapType::NormalCachable,
            PagePermission::USER_READ_WRITE,
        )
        .unwrap();

        let adapter = BochsAdapter {
            framebuffer_virt: fb_virt,
            io_virt: io_virt,
            current_x: 0,
            current_y: 0,
            pitch: 0,
        };
        assert!(adapter.bochs_io_read(VBE_DISPI_INDEX_ID) == 0xb0c5);

        println!("how much VGA memory: {}k", adapter.bochs_io_read(VBE_DISPI_INDEX_VIDEO_MEMORY_64K) * 64);
        Some(adapter)
    }

    fn bochs_io_read(&self, index: usize) -> u16 {
        unsafe { core::ptr::read_volatile((self.io_virt + 0x500 + index * 2) as *const u16) }
    }

    fn bochs_io_write(&self, index: usize, val: u16) {
        unsafe { core::ptr::write_volatile((self.io_virt + 0x500 + index * 2) as *mut u16, val) }
    }

    pub fn set_mode(&mut self, x: usize, y: usize) {
        // lets go
        self.current_x = x;
        self.current_y = y;
        self.pitch = y * 4;  // for a 32bpp mode

        self.bochs_io_write(VBE_DISPI_INDEX_ENABLE, 0);

        self.bochs_io_write(VBE_DISPI_INDEX_XRES, x as u16);
        self.bochs_io_write(VBE_DISPI_INDEX_YRES, y as u16);
        self.bochs_io_write(VBE_DISPI_INDEX_VIRT_WIDTH, y as u16);
        self.bochs_io_write(VBE_DISPI_INDEX_VIRT_HEIGHT, y as u16);

        self.bochs_io_write(VBE_DISPI_INDEX_X_OFFSET, 0);
        self.bochs_io_write(VBE_DISPI_INDEX_Y_OFFSET, 0);

        self.bochs_io_write(VBE_DISPI_INDEX_BPP, 32);

        // VBE_DISPI_LFB_ENABLED flag (0x40)
        self.bochs_io_write(VBE_DISPI_INDEX_ENABLE, 0x40 | 1);
    }

    pub fn get_framebuffer(&self) -> &'a mut [u32] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.framebuffer_virt as *mut u32,
                self.pitch * self.current_y,
            )
        }
    }
}
