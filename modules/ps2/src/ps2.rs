use francium_x86::io_port::*;
use bitflags::bitflags;

const DATA: u16 = 0x60;
const STATUS_COMMAND:u16 = 0x64;

const READ_CONTROLLER_CONFIG: u8 = 0x20;
const WRITE_CONTROLLER_CONFIG: u8 = 0x60;

const DISABLE_PORT_2 : u8 = 0xa7;
const ENABLE_PORT_2 : u8 = 0xa8;
const TEST_PORT_2 : u8 = 0xa9;
const SELF_TEST: u8 = 0xaa;
const TEST_PORT_1 : u8 = 0xab;
const DISABLE_PORT_1 : u8 = 0xae;
const ENABLE_PORT_1 : u8 = 0xad;

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

fn cmd_no_args(cmd: u8) {
    outb(STATUS_COMMAND, cmd);
}

fn cmd_one_arg(cmd: u8, val: u8) {
    outb(STATUS_COMMAND, cmd);
    while (inb(STATUS_COMMAND) & StatusFlags::INPUT_FULL.bits()) != 0 {}
    outb(DATA, val);
}

fn read_response() -> u8 {
    let status = inb(STATUS_COMMAND);
    while (inb(STATUS_COMMAND) & StatusFlags::OUTPUT_FULL.bits()) == 0 {}
    inb(DATA)
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

fn test_port_1() -> u8 {
    cmd_no_args(TEST_PORT_1);
    read_response()
}

fn test_port_2() -> u8 {
    cmd_no_args(TEST_PORT_2);
    read_response()
}

pub fn scan() {
    // to start: disable ps2 devices
    outb(STATUS_COMMAND, 0xad);
    outb(STATUS_COMMAND, 0xa7);
    // clear out data reg
    inb(DATA);

    let status = read_controller_config();
    println!("{:x}", status);
    write_controller_config(status & !(1<<6 | 1<<1 | 1<<0));

    if (status & (1<<5)) == 0 {
        println!("Second port didn't disable");
    }

    let self_test_result = self_test();
    println!("self test: {:x?}", self_test_result);

    enable_port_2();
    if (read_controller_config() & (1<<5)) == (1<<5) {
        println!("2nd port didn't enable");
    } else {
        disable_port_2();
    }

    println!("{:x}", test_port_1());
    println!("{:x}", test_port_2());

    enable_port_1();
    enable_port_2();

    outb(DATA, 0xff);
    println!("Reset port 1: {:x} {:x}", read_response(), read_response());

    outb(STATUS_COMMAND, 0xd4);
    outb(DATA, 0xff);
    println!("Reset port 2: {:x} {:x}", read_response(), read_response());

    outb(DATA, 0xf5);
    println!("{:0x}", read_response());

    println!("identify");
    outb(DATA, 0xf2);
    println!("{:0x}", read_response());
    println!("{:0x}", read_response());
    println!("{:0x}", read_response());

    println!("port 2");
    outb(STATUS_COMMAND, 0xd4);
    outb(DATA, 0xf5);
    println!("{:0x}", read_response());

    println!("identify");
    outb(STATUS_COMMAND, 0xd4);
    outb(DATA, 0xf2);
    println!("{:0x}", read_response());
    println!("{:0x}", read_response());
    println!("{:0x}", read_response());
}