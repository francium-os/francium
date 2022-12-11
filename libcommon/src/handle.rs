#[derive(Copy, Clone, Debug, Default)]
#[repr(transparent)]
pub struct Handle(pub u32);
pub const INVALID_HANDLE: Handle = Handle(0xffffffff);
