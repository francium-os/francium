use common::os_error::{OSResult,OSError};
use crate::Handle;

pub struct IPCHeader {
	pub id: u32
}

pub struct IPCMessage {}

pub trait IPCRead {
	fn read() -> Self;
}

pub trait IPCWrite {
	fn write(val: Self);
}

impl IPCMessage {
	pub fn new() -> IPCMessage {
		IPCMessage {}
	}

	pub fn read_header(&self) -> IPCHeader {
		IPCHeader {id:0}
	}

	pub fn write_header(&self) {

	}

	pub fn read<T: IPCRead>(&self) -> T {
		T::read()
	}

	pub fn write<T: IPCWrite>(&self, a: T) {
		T::write(a)
	}
}

impl IPCRead for u64 {
	fn read() -> u64 {
		unimplemented!()
	}
}
impl IPCWrite for u64 {
	fn write(_: u64) {
		unimplemented!()
	}
}

impl IPCWrite for OSError {
	fn write(_: OSError) {
		unimplemented!()
	}
}

impl IPCRead for Handle {
	fn read() -> Handle {
		unimplemented!()
	}
}

impl IPCWrite for Handle {
	fn write(_: Handle) {
		unimplemented!()
	}
}

impl<T: IPCRead> IPCRead for OSResult<T> {
	fn read() -> OSResult<T> {
		// read error code
		Ok(T::read())
	}
}

impl<T: IPCWrite> IPCWrite for OSResult<T> {
	fn write(res: OSResult<T>) {
		match res {
			Ok(x) => T::write(x),
			Err(_) => {}
		}
	}
}