use crate::ResultCode;

#[derive(Debug)]
pub struct OSError {
	res: ResultCode
}

pub fn result_to_error(r: ResultCode) -> OSError {
	OSError{res: r}
}