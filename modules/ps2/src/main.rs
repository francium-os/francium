use process::syscalls;
use process::Handle;

#[cfg(target_arch = "x86_64")]
mod ps2;

#[cfg(target_arch = "x86_64")]
fn main() {
    println!("Hello from ps2!");
    let ps2_ports = ps2::scan();

    let port_interrupt_events: Vec<Handle> = ps2_ports.iter().map(|x| x.interrupt_event).collect();
    loop {
        //println!("Waiting...");
        let index = syscalls::wait_many(&port_interrupt_events).unwrap();
        //println!("Got port {}", index);
        println!("port {} got scan {:x?}", index, ps2_ports[index].read());
        syscalls::clear_event(port_interrupt_events[index]).unwrap();
    }
    //syscalls::exit_process();
}

#[cfg(not(target_arch = "x86_64"))]
fn main() {
    syscalls::exit_process();
}
