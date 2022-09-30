use num_enum::TryFromPrimitive;

#[derive(Debug, PartialEq)]
#[repr(transparent)]
pub struct ResultCode(pub u32);
pub const RESULT_OK: ResultCode = ResultCode(0);

#[derive(Copy, Clone, Debug, TryFromPrimitive)]
#[repr(u16)]
pub enum Module {
    None = 0,
    Kernel = 1,
    SM = 2,
    FS = 3,
    Unknown = 0xffff
}

#[derive(Copy, Clone, Debug, TryFromPrimitive)]
#[repr(u16)]
pub enum Error {
    None = 0,
    NotImplemented = 1,
    NotAllowed = 2,
    Unknown = 0xffff
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct OSError {
	pub module: Module,
    pub err: Error
}

impl OSError {
    pub fn from_result_code(r: ResultCode) -> OSError {
        OSError { 
            module: Module::try_from((r.0 & 0xffff) as u16).unwrap_or(Module::Unknown),
            err: Error::try_from(((r.0 & 0xffff0000) >> 16) as u16).unwrap_or(Error::Unknown)
        }
    }

    pub fn to_result_code(&self) -> ResultCode {
        ResultCode((self.module as u32) | (self.err as u32) << 16)
    }
}

pub type OSResult<T> = Result<T, OSError>;