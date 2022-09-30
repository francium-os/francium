use core::convert::TryInto;
use common::os_error::{OSResult,OSError,ResultCode,RESULT_OK};
use crate::Handle;

#[thread_local]
#[no_mangle]
pub static mut IPC_BUFFER: [u8; 128] = [0; 128];

pub struct IPCHeader {
	pub id: u32
}

pub struct IPCMessage {
	read_offset: usize,
	write_offset: usize
}

pub trait IPCValue {
	fn read(ipc_buffer: &[u8]) -> (Self, usize) where Self: Sized;
	fn write(ipc_buffer: &mut [u8], val: &Self) -> usize;
}

impl IPCMessage {
	pub fn new() -> IPCMessage {
		IPCMessage {
			read_offset: 0,
			write_offset: 0
		}
	}

	pub fn read_header(&mut self) -> IPCHeader {
		let packed = self.read::<u32>();
		IPCHeader{id: packed}
	}

	pub fn write_header(&mut self, header: &IPCHeader) {
		let packed = header.id; // todo: pack
		self.write::<u32>(packed);
	}

	pub fn read<T: IPCValue>(&mut self) -> T {
		unsafe {
			let (val, offset) = T::read(&IPC_BUFFER[self.read_offset..]);
			self.read_offset += offset;
			val
		}
	}

	pub fn write<T: IPCValue>(&mut self, a: T) {
		unsafe {
			self.write_offset += T::write(&mut IPC_BUFFER[self.write_offset..], &a);
		}
	}
}

impl IPCValue for u64 {
	fn read(buffer: &[u8]) -> (u64, usize) {
		(u64::from_le_bytes(buffer[0..8].try_into().unwrap()), 8)
	}

	fn write(buffer: &mut [u8], val: &u64) -> usize {
		buffer[0..8].copy_from_slice(&u64::to_le_bytes(*val));
		8
	}
}

impl IPCValue for u32 {
	fn read(buffer: &[u8]) -> (u32, usize) {
		(u32::from_le_bytes(buffer[0..4].try_into().unwrap()), 4)
	}

	fn write(buffer: &mut [u8], val: &u32) -> usize {
		buffer[0..4].copy_from_slice(&u32::to_le_bytes(*val));
		4
	}
}

impl IPCValue for ResultCode {
	fn read(buffer: &[u8]) -> (ResultCode, usize) {
		let (res, len) = u32::read(buffer);
		(ResultCode(res), len)
	}

	fn write(buffer: &mut [u8], val: &ResultCode) -> usize {
		u32::write(buffer, &val.0)
	}
}

impl IPCValue for OSError {
	fn read(buffer: &[u8]) -> (OSError, usize) {
		let (res, len) = ResultCode::read(buffer);
		(OSError::from_result_code(res), len)
	}

	fn write(buffer: &mut [u8], val: &OSError) -> usize {
		ResultCode::write(buffer, &OSError::to_result_code(val))
	}
}

impl IPCValue for Handle {
	fn read(buffer: &[u8]) -> (Handle, usize) {
		let (res, len) = u32::read(buffer);
		(Handle(res), len)
	}

	fn write(buffer: &mut [u8], val: &Handle) -> usize {
		u32::write(buffer, &val.0)
	}
}

impl<T: IPCValue> IPCValue for OSResult<T> {
	fn read(buffer: &[u8]) -> (OSResult<T>, usize) {
		// read error code
		let (res, _) = ResultCode::read(buffer);
		if res == RESULT_OK {
			let (inner, len) = T::read(buffer);
			(Ok(inner), len+4)
		} else {
			(Err(OSError::from_result_code(res)), 4)
		}
	}

	fn write(buffer: &mut [u8], res: &OSResult<T>) -> usize {
		match res {
			Ok(x) => {
				let size = ResultCode::write(buffer, &RESULT_OK);
				T::write(&mut buffer[size..], &x) + size
			},
			Err(err) => {
				OSError::write(buffer, &err)
			}
		}
	}
}