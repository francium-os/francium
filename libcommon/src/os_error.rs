use num_enum::TryFromPrimitive;

#[derive(Debug, PartialEq)]
#[repr(transparent)]
pub struct ResultCode(u32);
pub const RESULT_OK: ResultCode = ResultCode(0);

#[derive(Debug, TryFromPrimitive)]
#[repr(u16)]
pub enum Module {
    None = 0,
    Kernel = 1,
    Unknown = 0xffff
}

#[derive(Debug, TryFromPrimitive)]
#[repr(u16)]
pub enum Error {
    None = 0,
    NotAllowed = 1,
    Unknown = 0xffff
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct OSError {
	module: Module,
    err: Error
}

impl OSError {
    pub fn from_result_code(r: ResultCode) -> OSError {
        OSError { 
            module: Module::try_from((r.0 & 0xffff) as u16).unwrap_or(Module::Unknown),
            err: Error::try_from(((r.0 & 0xffff0000) >> 16) as u16).unwrap_or(Error::Unknown)
        }
    }
}

pub type OSResult<T> = Result<T, OSError>;