use bitflags::bitflags;
use common::Handle;
use francium_x86::io_port::*;
use process::syscalls;

const DATA: u16 = 0x60;
const STATUS_COMMAND: u16 = 0x64;

/* controller command definitions */
const READ_CONTROLLER_CONFIG: u8 = 0x20;
const WRITE_CONTROLLER_CONFIG: u8 = 0x60;

const DISABLE_PORT_2: u8 = 0xa7;
const ENABLE_PORT_2: u8 = 0xa8;
//const TEST_PORT_2: u8 = 0xa9;
const SELF_TEST: u8 = 0xaa;
//const TEST_PORT_1: u8 = 0xab;
const DISABLE_PORT_1: u8 = 0xae;
const ENABLE_PORT_1: u8 = 0xad;
const SECOND_PORT_DATA: u8 = 0xd4;

/* ps2 device commands */
const DEVICE_ENABLE_SCAN: u8 = 0xf4;
const DEVICE_DISABLE_SCAN: u8 = 0xf5;
const DEVICE_IDENTIFY: u8 = 0xf2;
const DEVICE_RESET: u8 = 0xff;

/* ps2 device responses */
//const RESPONSE_OK: u8 = 0xfa;
//const RESPONSE_FAIL: u8 = 0xfc;

/* status bits */
bitflags! {
    struct StatusFlags: u8 {
        const OUTPUT_FULL = 1;
        const INPUT_FULL = 1<<1;
        const SYSTEM_FLAG = 1<<2;
        const COMMAND_DATA = 1<<3;
        /* bit 4 unknown */
        /* bit 5 unknown */
        const TIMEOUT = 1<<6;
        const PARITY_ERROR = 1<<7;
    }
}

pub struct PS2Port {
    is_second_port: bool,
    pub interrupt_event: Handle,
}

impl PS2Port {
    fn write_byte(&self, byte: u8) {
        if self.is_second_port {
            outb(STATUS_COMMAND, SECOND_PORT_DATA);
        }
        outb(DATA, byte);
    }

    fn reset(&self) -> Vec<u8> {
        self.write_byte(DEVICE_RESET);

        let mut res = Vec::new();
        while let Some(x) = read_response_with_timeout() {
            res.push(x);
        }
        res
    }

    fn disable_scan(&self) -> u8 {
        self.write_byte(DEVICE_DISABLE_SCAN);
        read_response()
    }

    fn enable_scan(&self) -> u8 {
        self.write_byte(DEVICE_ENABLE_SCAN);
        read_response()
    }

    fn identify(&self) -> Vec<u8> {
        self.write_byte(DEVICE_IDENTIFY);

        let mut res = Vec::new();
        while let Some(x) = read_response_with_timeout() {
            res.push(x);
        }
        res
    }

    pub fn read(&self) -> Vec<u8> {
        let mut res = Vec::new();
        while let Some(x) = read_response_with_timeout() {
            res.push(x);
        }
        res
    }
}

fn cmd_no_args(cmd: u8) {
    outb(STATUS_COMMAND, cmd);
}

fn cmd_one_arg(cmd: u8, val: u8) {
    outb(STATUS_COMMAND, cmd);
    while (inb(STATUS_COMMAND) & StatusFlags::INPUT_FULL.bits()) != 0 {}
    outb(DATA, val);
}

fn read_response() -> u8 {
    while (inb(STATUS_COMMAND) & StatusFlags::OUTPUT_FULL.bits()) == 0 {}
    inb(DATA)
}

fn read_response_with_timeout() -> Option<u8> {
    // TODO: fix
    //let start_system_tick = syscalls::get_system_tick();
    // Spinwait because sleeping for 1ms probably won't work.
    let mut attempts = 1000;
    while (inb(STATUS_COMMAND) & StatusFlags::OUTPUT_FULL.bits()) == 0 {
        if attempts == 0 {
            return None;
        }
        attempts -= 1;
    }

    Some(inb(DATA))
}

fn read_controller_config() -> u8 {
    cmd_no_args(READ_CONTROLLER_CONFIG);
    read_response()
}

fn write_controller_config(val: u8) {
    cmd_one_arg(WRITE_CONTROLLER_CONFIG, val)
}

fn self_test() -> u8 {
    cmd_no_args(SELF_TEST);
    read_response()
}

fn enable_port_1() {
    cmd_no_args(ENABLE_PORT_1);
}

fn disable_port_1() {
    cmd_no_args(DISABLE_PORT_1);
}

fn enable_port_2() {
    cmd_no_args(ENABLE_PORT_2);
}

fn disable_port_2() {
    cmd_no_args(DISABLE_PORT_2);
}

/*fn test_port_1() -> u8 {
    cmd_no_args(TEST_PORT_1);
    read_response()
}

fn test_port_2() -> u8 {
    cmd_no_args(TEST_PORT_2);
    read_response()
}*/

pub fn scan() -> Vec<PS2Port> {
    let mut ports = Vec::new();

    // to start: disable ps2 devices
    disable_port_1();
    disable_port_2();

    // clear out data reg
    inb(DATA);

    let status = read_controller_config();
    write_controller_config(status & !(1 << 6 | 1 << 1 | 1 << 0));

    if (status & (1 << 5)) == 0 {
        println!("Second port didn't disable");
    }

    let self_test_result = self_test();
    assert!(self_test_result == 0x55);

    enable_port_2();
    if (read_controller_config() & (1 << 5)) == (1 << 5) {
        println!("2nd port didn't enable");
    } else {
        disable_port_2();
    }

    let port_1 = PS2Port {
        is_second_port: false,
        interrupt_event: syscalls::create_event().unwrap(),
    };
    let port_2 = PS2Port {
        is_second_port: true,
        interrupt_event: syscalls::create_event().unwrap(),
    };

    ports.push(port_1);
    ports.push(port_2);

    enable_port_1();
    enable_port_2();

    for p in &ports {
        p.reset();
    }

    for p in &ports {
        p.disable_scan();
    }

    for p in &ports {
        println!("Identify: {:x?}", p.identify());
    }

    for p in &ports {
        p.enable_scan();
    }

    // Bind the interrupt events we created earlier.
    for p in &ports {
        let interrupt_id = if !p.is_second_port { 1 } else { 12 };
        syscalls::bind_interrupt(p.interrupt_event, interrupt_id).unwrap();
    }

    // All good, enable IRQs.
    let status_2 = read_controller_config();
    write_controller_config(status_2 | (1 << 0) | (1 << 1));

    ports
}
