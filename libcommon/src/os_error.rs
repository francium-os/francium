use num_enum::TryFromPrimitive;

#[derive(Debug, PartialEq)]
#[repr(transparent)]
pub struct ResultCode(pub u32);
pub const RESULT_OK: ResultCode = ResultCode(0);

impl ResultCode {
    pub fn new(module: Module, reason: Reason) -> ResultCode {
        OSError::to_result_code(&OSError { module, reason })
    }
}

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
pub enum Reason {
    None = 0,
    NotImplemented = 1,
    NotAllowed = 2,
    InvalidHandle = 3,
    NotFound = 4,
    TryAgain = 5,
    Unknown = 0xffff
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct OSError {
	pub module: Module,
    pub reason: Reason
}

impl OSError {
    pub fn new(module: Module, reason: Reason) -> OSError {
        OSError { module, reason }
    }

    pub fn from_result_code(r: ResultCode) -> OSError {
        OSError { 
            module: Module::try_from((r.0 & 0xffff) as u16).unwrap_or(Module::Unknown),
            reason: Reason::try_from(((r.0 & 0xffff0000) >> 16) as u16).unwrap_or(Reason::Unknown)
        }
    }

    pub fn to_result_code(&self) -> ResultCode {
        ResultCode((self.module as u32) | (self.reason as u32) << 16)
    }
}

pub type OSResult<T> = Result<T, OSError>;