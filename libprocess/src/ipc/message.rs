use common::ipc::*;
use common::os_error::{OSError, OSResult, ResultCode, RESULT_OK};
use core::convert::TryInto;

#[thread_local]
pub static mut IPC_BUFFER: [u8; 128] = [0; 128];

pub struct IPCMessage<'a> {
    pub header: IPCHeader,
    pub read_offset: usize,
    pub write_offset: usize,
    pub current_translate: usize,
    pub translate_entries: [TranslateEntry; MAX_TRANSLATE],
    pub buffer: &'a mut [u8],
}

pub trait IPCValue {
    fn read(_msg: &mut IPCMessage) -> Self
    where
        Self: Sized,
    {
        unimplemented!();
    }

    fn write(_msg: &mut IPCMessage, _val: &Self) {
        unimplemented!();
    }
}

impl IPCMessage<'_> {
    pub fn new(buffer: &mut [u8]) -> IPCMessage {
        // sizeof(packed header) == 4
        IPCMessage {
            header: IPCHeader {
                id: 0,
                size: 0,
                translate_count: 0,
            },
            read_offset: 4,
            write_offset: 4,
            current_translate: 0,
            translate_entries: [TranslateEntry::None; MAX_TRANSLATE],
            buffer: buffer,
        }
    }

    pub fn read_header(&mut self) {
        let packed = u32::from_le_bytes(self.buffer[0..4].try_into().unwrap());
        self.header = IPCHeader::unpack(packed);
    }

    pub fn write_header_for(&mut self, method_id: u32) {
        self.header = IPCHeader {
            id: method_id,
            size: self.write_offset,
            translate_count: self.current_translate,
        };
        let packed = IPCHeader::pack(&self.header);

        self.buffer[0..4].copy_from_slice(&u32::to_le_bytes(packed));
    }

    pub fn write_translates(&mut self) {
        for i in 0..self.current_translate {
            let entry = self.translate_entries[i];
            let off = self.write_offset + i * 16;
            let buffer = &mut self.buffer[off..off + 16];
            TranslateEntry::write(buffer.try_into().unwrap(), entry);
        }
    }

    pub fn read_translates(&mut self) {
        for i in 0..self.header.translate_count {
            let off = self.header.size + i * 16;
            let buffer = &self.buffer[off..off + 16];
            self.translate_entries[i] = TranslateEntry::read(buffer.try_into().unwrap());
        }
    }

    pub fn read<T: IPCValue>(&mut self) -> T {
        T::read(self)
    }

    pub fn write<T: IPCValue>(&mut self, a: T) {
        T::write(self, &a)
    }
}

impl IPCValue for u64 {
    fn read(msg: &mut IPCMessage) -> u64 {
        let val = u64::from_le_bytes(
            msg.buffer[msg.read_offset..msg.read_offset + 8]
                .try_into()
                .unwrap(),
        );
        msg.read_offset += 8;
        val
    }

    fn write(msg: &mut IPCMessage, val: &u64) {
        msg.buffer[msg.write_offset..msg.write_offset + 8].copy_from_slice(&u64::to_le_bytes(*val));
        msg.write_offset += 8;
    }
}

impl IPCValue for u32 {
    fn read(msg: &mut IPCMessage) -> u32 {
        let val = u32::from_le_bytes(
            msg.buffer[msg.read_offset..msg.read_offset + 4]
                .try_into()
                .unwrap(),
        );
        msg.read_offset += 4;
        val
    }

    fn write(msg: &mut IPCMessage, val: &u32) {
        msg.buffer[msg.write_offset..msg.write_offset + 4].copy_from_slice(&u32::to_le_bytes(*val));
        msg.write_offset += 4;
    }
}

impl IPCValue for u16 {
    fn read(msg: &mut IPCMessage) -> u16 {
        let val = u16::from_le_bytes(
            msg.buffer[msg.read_offset..msg.read_offset + 2]
                .try_into()
                .unwrap(),
        );
        msg.read_offset += 2;
        val
    }

    fn write(msg: &mut IPCMessage, val: &u16) {
        msg.buffer[msg.write_offset..msg.write_offset + 2].copy_from_slice(&u16::to_le_bytes(*val));
        msg.write_offset += 2;
    }
}

impl IPCValue for u8 {
    fn read(msg: &mut IPCMessage) -> u8 {
        let val = msg.buffer[msg.read_offset];
        msg.read_offset += 1;
        val
    }

    fn write(msg: &mut IPCMessage, val: &u8) {
        msg.buffer[msg.write_offset] = *val;
        msg.write_offset += 1;
    }
}

// TODO: sizeof(usize)=4?
impl IPCValue for usize {
    fn read(msg: &mut IPCMessage) -> usize {
        let val = u64::from_le_bytes(
            msg.buffer[msg.read_offset..msg.read_offset + 8]
                .try_into()
                .unwrap(),
        );
        msg.read_offset += 8;
        val as usize
    }

    fn write(msg: &mut IPCMessage, val: &usize) {
        msg.buffer[msg.write_offset..msg.write_offset + 8]
            .copy_from_slice(&u64::to_le_bytes(*val as u64));
        msg.write_offset += 8;
    }
}

impl IPCValue for bool {
    fn read(msg: &mut IPCMessage) -> bool {
        let val = msg.buffer[msg.read_offset];
        msg.read_offset += 1;
        val != 0
    }

    fn write(msg: &mut IPCMessage, val: &bool) {
        if *val {
            msg.buffer[msg.write_offset] = 1;
        } else {
            msg.buffer[msg.write_offset] = 0;
        }
        msg.write_offset += 1;
    }
}

impl IPCValue for ResultCode {
    fn read(msg: &mut IPCMessage) -> ResultCode {
        ResultCode(u32::read(msg))
    }

    fn write(msg: &mut IPCMessage, val: &ResultCode) {
        u32::write(msg, &val.0)
    }
}

impl IPCValue for OSError {
    fn read(msg: &mut IPCMessage) -> OSError {
        OSError::from_result_code(ResultCode::read(msg))
    }

    fn write(msg: &mut IPCMessage, val: &OSError) {
        ResultCode::write(msg, &OSError::to_result_code(val))
    }
}

impl IPCValue for TranslateMoveHandle {
    fn read(msg: &mut IPCMessage) -> TranslateMoveHandle {
        if let TranslateEntry::MoveHandle(handle) = msg.translate_entries[msg.current_translate] {
            msg.current_translate += 1;
            TranslateMoveHandle(handle)
        } else {
            println!(
                "{:?} of {:?}",
                msg.current_translate, msg.header.translate_count
            );
            panic!(
                "Invalid translate! Expected Move, got {:?}",
                msg.translate_entries[msg.current_translate]
            );
        }
    }

    fn write(msg: &mut IPCMessage, value: &TranslateMoveHandle) {
        msg.translate_entries[msg.current_translate] = TranslateEntry::MoveHandle(value.0);
        msg.current_translate += 1;
    }
}

impl IPCValue for TranslateCopyHandle {
    fn read(msg: &mut IPCMessage) -> TranslateCopyHandle {
        if let TranslateEntry::CopyHandle(handle) = msg.translate_entries[msg.current_translate] {
            msg.current_translate += 1;
            TranslateCopyHandle(handle)
        } else {
            println!(
                "{:?} of {:?}",
                msg.current_translate, msg.header.translate_count
            );
            panic!(
                "Invalid translate! Expected Copy, got {:?}",
                msg.translate_entries[msg.current_translate]
            );
        }
    }

    fn write(msg: &mut IPCMessage, value: &TranslateCopyHandle) {
        msg.translate_entries[msg.current_translate] = TranslateEntry::CopyHandle(value.0);
        msg.current_translate += 1;
    }
}

impl<T: IPCValue> IPCValue for OSResult<T> {
    fn read(msg: &mut IPCMessage) -> OSResult<T> {
        // read error code
        let res = ResultCode::read(msg);
        if res == RESULT_OK {
            Ok(T::read(msg))
        } else {
            Err(OSError::from_result_code(res))
        }
    }

    fn write(msg: &mut IPCMessage, res: &OSResult<T>) {
        match res {
            Ok(x) => {
                ResultCode::write(msg, &RESULT_OK);
                // TODO: sizeof(resultcode) == 4
                T::write(msg, &x)
            }
            Err(err) => OSError::write(msg, &err),
        }
    }
}

impl<T: IPCValue> IPCValue for Option<T> {
    fn read(msg: &mut IPCMessage) -> Option<T> {
        // read error code
        let present = bool::read(msg);
        if present {
            Some(T::read(msg))
        } else {
            None
        }
    }

    fn write(msg: &mut IPCMessage, res: &Option<T>) {
        match res {
            Some(x) => {
                bool::write(msg, &true);
                T::write(msg, &x);
            }
            None => bool::write(msg, &false),
        }
    }
}

impl IPCValue for () {
    fn read(_msg: &mut IPCMessage) {}

    fn write(_msg: &mut IPCMessage, _: &()) {}
}

impl<T> IPCValue for (T,)
where
    T: IPCValue,
{
    fn read(msg: &mut IPCMessage) -> (T,) {
        (T::read(msg),)
    }

    fn write(msg: &mut IPCMessage, val: &(T,)) {
        T::write(msg, &val.0)
    }
}

impl<T, U> IPCValue for (T, U)
where
    T: IPCValue,
    U: IPCValue,
{
    fn read(msg: &mut IPCMessage) -> (T, U) {
        (T::read(msg), U::read(msg))
    }

    fn write(msg: &mut IPCMessage, val: &(T, U)) {
        T::write(msg, &val.0);
        U::write(msg, &val.1);
    }
}

impl<T> IPCValue for Vec<T>
where
    T: IPCValue,
{
    fn read(msg: &mut IPCMessage) -> Vec<T> {
        let length = usize::read(msg);
        println!("read vec! len={:?}", length);

        let mut new_vec = Vec::with_capacity(length);
        for _ in 0..length {
            new_vec.push(T::read(msg))
        }

        new_vec
    }

    fn write(msg: &mut IPCMessage, value: &Vec<T>) {
        println!("write vec! len={:?}", value.len());

        usize::write(msg, &value.len());
        for item in value {
            T::write(msg, &item)
        }
    }
}
