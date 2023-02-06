#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        use core::ops::DerefMut;
        let mut lock_guard = crate::platform::DEFAULT_UART.lock();
        $crate::drivers::uart_print!(lock_guard.deref_mut(), $($arg)*);
    }};
}

#[macro_export]
macro_rules! println {
    () => {{
        use core::ops::DerefMut;
        let mut lock_guard = crate::platform::DEFAULT_UART.lock();
        $crate::drivers::uart_print!(lock_guard.deref_mut(), "\n");
    }};
    ($($arg:tt)*) => {{
        use core::ops::DerefMut;
        let mut lock_guard = crate::platform::DEFAULT_UART.lock();
        $crate::drivers::uart_println!(lock_guard.deref_mut(), $($arg)*);
    }}
}
