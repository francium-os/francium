use crate::SerialPort;

#[macro_export]
macro_rules! uart_print {
    ($uart:expr, $($arg:tt)*) => {{
        let _ = core::fmt::Write::write_fmt(
            &mut $crate::print::Writer{uart: $uart},
            format_args!($($arg)*)
        );
    }};
}

#[macro_export]
macro_rules! uart_println {
    () => {{
        print!("\n");
    }};
    ($uart:expr, $($arg:tt)*) => {{
        let writer = &mut $crate::print::Writer{uart: $uart};

        let _ = core::fmt::Write::write_fmt(
            writer,
            format_args!($($arg)*)
        );
        let _ = core::fmt::Write::write_str(writer, "\r\n");
    }}
}

/// writes characters to the system log device
pub struct Writer<'a, T: SerialPort> {
    pub uart: &'a mut T
}

impl<'a, T: SerialPort> core::fmt::Write for Writer<'a, T> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.uart.write_string(s);
        Ok(())
    }
}