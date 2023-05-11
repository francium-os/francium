use common::os_error::OSError as CommonError;
pub use common::os_error::{ResultCode, Module, Reason, RESULT_OK};

#[derive(Debug)]
pub struct OSError {
	common: CommonError
}

impl OSError {
	pub fn new(m: Module, r: Reason) -> OSError {
		OSError { common: CommonError::new(m, r) }
	}

	pub fn from_result_code(r: ResultCode) -> OSError {
		OSError { common: CommonError::from_result_code(r) }
	}
	pub fn to_result_code(e: &OSError) -> ResultCode {
		CommonError::to_result_code(&e.common)
	}
}

impl From<CommonError> for OSError {
    fn from(common: CommonError) -> Self { 
    	OSError { common: common }
    }
}

impl From<tokio::task::JoinError> for OSError {
    fn from(_join_err: tokio::task::JoinError) -> Self { 
    	OSError { common: CommonError::new(Module::LibProcess, Reason::Unknown) }
    }
}

impl From<std::io::Error> for OSError {
    fn from(_e: std::io::Error) -> Self { 
    	OSError { common: CommonError::new(Module::LibProcess, Reason::Unknown) }
    }
}

pub type OSResult<T> = Result<T, OSError>;