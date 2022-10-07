use core::convert::TryInto;
use common::os_error::{OSResult,OSError,ResultCode,RESULT_OK};
use common::ipc::*;
use common::Handle;

#[thread_local]
#[no_mangle]
pub static mut IPC_BUFFER: [u8; 128] = [0; 128];

pub struct IPCMessage {
	pub header: IPCHeader,
	pub read_offset: usize,
	pub write_offset: usize,
	pub current_translate: usize,
	pub translate_entries: [TranslateEntry; MAX_TRANSLATE]
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
			header: IPCHeader { id: 0, size: 0, translate_count: 0 },
			read_offset: 4,
			write_offset: 4,
			current_translate: 0,
			translate_entries: [TranslateEntry::None; MAX_TRANSLATE]
		}
	}

	pub fn read_header(&mut self) {
		let packed = unsafe {
			u32::from_le_bytes(IPC_BUFFER[0..4].try_into().unwrap())
		};

		self.header = IPCHeader::unpack(packed);
	}

	pub fn write_header_for(&mut self, method_id: u32) {
		self.header = IPCHeader { id: method_id, size: self.write_offset, translate_count: self.current_translate };
		let packed = IPCHeader::pack(&self.header);

		unsafe {
			IPC_BUFFER[0..4].copy_from_slice(&u32::to_le_bytes(packed));
		}
	}

	pub fn write_translates(&mut self) {
		//println!("Write translates: {:?} {:?} {:?}", self.write_offset, self.header.size, self.current_translate);

		unsafe {
			for i in 0..self.current_translate {
				let entry = self.translate_entries[i];
				match entry {
					TranslateEntry::MoveHandle(handle) => {
						let off = self.write_offset + i * 16;

						let buffer = &mut IPC_BUFFER[off .. off + 16];
						buffer[0..8].copy_from_slice(&u64::to_le_bytes(TRANSLATE_TYPE_MOVE_HANDLE));
						buffer[8..16].copy_from_slice(&u64::to_le_bytes(handle.0 as u64));
					},
					TranslateEntry::CopyHandle(handle) => {
						let off = self.write_offset + i * 16;

						let buffer = &mut IPC_BUFFER[off .. off + 16];
						buffer[0..8].copy_from_slice(&u64::to_le_bytes(TRANSLATE_TYPE_COPY_HANDLE));
						buffer[8..16].copy_from_slice(&u64::to_le_bytes(handle.0 as u64));
					},
					_ => { unimplemented!(); }
				}
			}
		}
	}

	pub fn read_translates(&mut self) {
		unsafe {
			//println!("read off: {:?}, size: {:?}", self.read_offset, self.header.size);

			for i in 0..self.header.translate_count {
				let off = self.header.size + i * 16;
				let buffer = &IPC_BUFFER[off .. off + 16];
				let translate_type = u64::from_le_bytes(buffer[0..8].try_into().unwrap());
				let translate_payload = u64::from_le_bytes(buffer[8..16].try_into().unwrap());
				
				match translate_type {
					TRANSLATE_TYPE_MOVE_HANDLE => {
						self.translate_entries[i] = TranslateEntry::MoveHandle(Handle(translate_payload as u32));
					},
					TRANSLATE_TYPE_COPY_HANDLE => {
						self.translate_entries[i] = TranslateEntry::CopyHandle(Handle(translate_payload as u32));
					},
					_ => { unimplemented!(); }
				}
			}
		}
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

impl IPCValue for TranslateMoveHandle {
	fn read(msg: &mut IPCMessage, _buffer: &[u8]) -> TranslateMoveHandle {
		if let TranslateEntry::MoveHandle(handle) = msg.translate_entries[msg.current_translate] {
			msg.current_translate += 1;
			TranslateMoveHandle(handle)
		} else {
			println!("{:?} of {:?}", msg.current_translate, msg.header.translate_count);
			panic!("Invalid translate! Expected Move, got {:?}", msg.translate_entries[msg.current_translate]);
		}
	}

	fn write(msg: &mut IPCMessage, _buffer: &mut [u8], value: &TranslateMoveHandle) {
		msg.translate_entries[msg.current_translate] = TranslateEntry::MoveHandle(value.0);
		msg.current_translate += 1;
	}
}

impl IPCValue for TranslateCopyHandle {
	fn read(msg: &mut IPCMessage, _buffer: &[u8]) -> TranslateCopyHandle {
		if let TranslateEntry::CopyHandle(handle) = msg.translate_entries[msg.current_translate] {
			msg.current_translate += 1;
			TranslateCopyHandle(handle)
		} else {
			println!("{:?} of {:?}", msg.current_translate, msg.header.translate_count);
			panic!("Invalid translate! Expected Copy, got {:?}", msg.translate_entries[msg.current_translate]);
		}
	}

	fn write(msg: &mut IPCMessage, _buffer: &mut [u8], value: &TranslateCopyHandle) {
		msg.translate_entries[msg.current_translate] = TranslateEntry::CopyHandle(value.0);
		msg.current_translate += 1;
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