use crate::platform::DEFAULT_UART;

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        let _ = core::fmt::Write::write_fmt(
            &mut $crate::print::Writer,
            format_args!($($arg)*)
        );
    }};
}

#[macro_export]
macro_rules! println {
    () => {{
        print!("\n");
    }};
    ($($arg:tt)*) => {{
        let writer = &mut $crate::print::Writer;
        let _ = core::fmt::Write::write_fmt(
            writer,
            format_args!($($arg)*)
        );
        let _ = core::fmt::Write::write_str(writer, "\r\n");
    }}
}

/// writes characters to the system log device
pub struct Writer;
impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        DEFAULT_UART.lock().write_string(s);
        Ok(())
    }
}
