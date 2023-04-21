use crate::ecam::*;
use smallvec::SmallVec;

fn get_function_header(
    block: usize,
    bus: u8,
    device: u8,
    function: u8,
) -> &'static mut ConfigurationSpaceHeader {
    let offset: usize =
        ((bus as usize) << 20) + ((device as usize) << 15) + ((function as usize) << 12);
    unsafe {
        ((block + offset) as *mut ConfigurationSpaceHeader)
            .as_mut()
            .unwrap()
    }
}

fn get_function_type0(
    block: usize,
    bus: u8,
    device: u8,
    function: u8,
) -> &'static mut ConfigurationSpaceType0 {
    let offset: usize =
        ((bus as usize) << 20) + ((device as usize) << 15) + ((function as usize) << 12);
    unsafe {
        ((block + offset) as *mut ConfigurationSpaceType0)
            .as_mut()
            .unwrap()
    }
}

// TODO: hotplug exists

#[derive(Debug)]
pub struct PCIFunction {
    pub num: u8,
    pub inner: &'static mut ConfigurationSpaceType0,
}

#[derive(Debug)]
pub struct PCIDevice {
    pub num: u8,
    pub functions: SmallVec<[PCIFunction; 1]>,
}

impl PCIDevice {
    fn new(block: usize, bus_num: u8, device_num: u8) -> Option<PCIDevice> {
        let mut device_obj = PCIDevice {
            num: device_num,
            functions: SmallVec::new(),
        };

        let mut device_ok = false;
        for function_num in 0..=255 {
            let (function_valid, is_multifunction) =
                device_obj.discover_function(block, bus_num, device_num, function_num);
            if !function_valid {
                break;
            }
            device_ok = true;

            if !is_multifunction {
                break;
            }
        }

        if device_ok {
            Some(device_obj)
        } else {
            None
        }
    }

    fn discover_function(
        &mut self,
        block: usize,
        bus_num: u8,
        device_num: u8,
        function_num: u8,
    ) -> (bool /* valid */, bool /* is multifunction */) {
        let function = get_function_header(block, bus_num, device_num, function_num);

        if function.vendor_id != 0xffff {
            /*println!(
                "vid: {:04x}, pid: {:04x} class = {:02x} subclass = {:02x} type={:04x}",
                { function.vendor_id },
                { function.device_id },
                { function.class },
                { function.subclass },
                { function.header_type }
            );*/
            let header_type = function.header_type & 0x7f;
            match header_type {
                /* device */
                0 => {
                    let function_type0 =
                        get_function_type0(block, bus_num, device_num, function_num);
                    /*for bar in function_type0.bars {
                        println!(
                            "{:08x}: Bar type: {}, location: 0b{:02b}, prefetchable: {}",
                            bar,
                            bar & 1,
                            (bar & (3 << 1)) >> 1,
                            (bar & (1 << 3)) >> 3
                        );
                    }*/
                    self.functions.push(PCIFunction {
                        num: function_num,
                        inner: function_type0,
                    });
                }
                /* pci bridge */
                1 => {
                    println!("Skipping PCI bridge!");
                },
                _ => todo!(),
            }

            (true, (function.header_type & 0x80) == 0x80)
        } else {
            (false, false)
        }
    }
}

#[derive(Debug)]
pub struct PCIBus {
    pub num: u8,
    pub devices: Vec<PCIDevice>,
}

impl PCIBus {
    pub fn new(block: usize, bus_num: u8) -> Option<PCIBus> {
        let mut bus_obj = PCIBus {
            num: bus_num,
            devices: Vec::new(),
        };

        let mut bus_ok = false;
        for device_num in 0..=255 {
            match PCIDevice::new(block, bus_num, device_num) {
                Some(some_device) => {
                    bus_obj.devices.push(some_device);
                }

                None => break,
            }

            bus_ok = true;
        }

        if bus_ok {
            Some(bus_obj)
        } else {
            None
        }
    }
}
