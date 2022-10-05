use core::convert::TryInto;
use core::ops::Deref;
use common::os_error::{OSResult,OSError,ResultCode,RESULT_OK};
use crate::Handle;

#[thread_local]
#[no_mangle]
pub static mut IPC_BUFFER: [u8; 128] = [0; 128];

#[repr(transparent)]
#[derive(Debug)]
pub struct TranslateHandle(pub Handle);

impl Deref for TranslateHandle {
	type Target = Handle;
	fn deref(&self) -> &<Self as Deref>::Target {
		&self.0
	}
}

pub struct IPCHeader {
	pub id: u32,
	pub size: usize,
	pub translate_count: usize
}

pub struct IPCMessage {
	pub read_offset: usize,
	pub write_offset: usize
}

pub trait IPCValue {
	fn read(_msg: &mut IPCMessage, _ipc_buffer: &[u8]) -> Self where Self: Sized {
		unimplemented!();
	}

	fn write(_msg: &mut IPCMessage, _ipc_buffer: &mut [u8], _val: &Self) {
		unimplemented!();
	}
}

impl IPCMessage {
	pub fn new() -> IPCMessage {
		// sizeof(packed header) == 4
		IPCMessage {
			read_offset: 4,
			write_offset: 4
		}
	}

	pub fn read_header(&mut self) -> IPCHeader {
		self.read_offset = 0;

		let packed = self.read::<u32>();

		let message_id = packed & 0xff;
		let message_size = (packed & (0xff<<8))>>8;
		let message_translate_count = (packed & (0xff<<16))>>16;

		IPCHeader{id: message_id, size: message_size as usize, translate_count: message_translate_count as usize }
	}

	pub fn write_header(&mut self, header: &IPCHeader) {
		assert!(header.size < 256);
		assert!(header.translate_count < 256);

		let packed = header.id | (((header.size & 0xff) as u32) << 8) | (((header.translate_count & 0xff) as u32) << 16);
		self.write_offset = 0;
		self.write::<u32>(packed);
	}

	pub fn read<T: IPCValue>(&mut self) -> T {
		unsafe {
			T::read(self, &IPC_BUFFER[self.read_offset..])
		}
	}

	pub fn write<T: IPCValue>(&mut self, a: T) {
		unsafe {
			T::write(self, &mut IPC_BUFFER[self.write_offset..], &a)
		}
	}
}

impl IPCValue for u64 {
	fn read(msg: &mut IPCMessage, buffer: &[u8]) -> u64 {
		let val = u64::from_le_bytes(buffer[0..8].try_into().unwrap());
		msg.read_offset += 8;
		val
	}

	fn write(msg: &mut IPCMessage, buffer: &mut [u8], val: &u64) {
		buffer[0..8].copy_from_slice(&u64::to_le_bytes(*val));
		msg.write_offset += 8;
	}
}

impl IPCValue for u32 {
	fn read(msg: &mut IPCMessage, buffer: &[u8]) -> u32 {
		let val = u32::from_le_bytes(buffer[0..4].try_into().unwrap());
		msg.read_offset += 4;
		val
	}

	fn write(msg: &mut IPCMessage, buffer: &mut [u8], val: &u32) {
		buffer[0..4].copy_from_slice(&u32::to_le_bytes(*val));
		msg.write_offset += 4;
	}
}

impl IPCValue for ResultCode {
	fn read(msg: &mut IPCMessage, buffer: &[u8]) -> ResultCode {
		ResultCode(u32::read(msg, buffer))
	}

	fn write(msg: &mut IPCMessage, buffer: &mut [u8], val: &ResultCode) {
		u32::write(msg, buffer, &val.0)
	}
}

impl IPCValue for OSError {
	fn read(msg: &mut IPCMessage, buffer: &[u8]) -> OSError {
		OSError::from_result_code(ResultCode::read(msg, buffer))
	}

	fn write(msg: &mut IPCMessage, buffer: &mut [u8], val: &OSError) {
		ResultCode::write(msg, buffer, &OSError::to_result_code(val))
	}
}

impl IPCValue for TranslateHandle {
	fn read(msg: &mut IPCMessage, _buffer: &[u8]) -> TranslateHandle {
		msg.read::<u32>();
		let value = msg.read::<u32>();
		TranslateHandle(Handle(value))
	}

	fn write(msg: &mut IPCMessage, _buffer: &mut [u8], value: &TranslateHandle) {
		msg.write::<u32>(0);
		msg.write::<u32>(value.0.0)
	}
}

impl<T: IPCValue> IPCValue for OSResult<T> {
	fn read(msg: &mut IPCMessage, buffer: &[u8]) -> OSResult<T> {
		// read error code
		let res = ResultCode::read(msg, buffer);
		if res == RESULT_OK {
			Ok(T::read(msg, buffer))
		} else {
			Err(OSError::from_result_code(res))
		}
	}

	fn write(msg: &mut IPCMessage, buffer: &mut [u8], res: &OSResult<T>) {
		match res {
			Ok(x) => {
				ResultCode::write(msg, buffer, &RESULT_OK);
				// TODO: sizeof(resultcode) == 4
				T::write(msg, &mut buffer[4..], &x)
			},
			Err(err) => {
				OSError::write(msg, buffer, &err)
			}
		}
	}
}

impl IPCValue for () {
	fn read(_msg: &mut IPCMessage, _: &[u8]) {
	}

	fn write(_msg: &mut IPCMessage, _: &mut [u8], _: &()) {
	}
}