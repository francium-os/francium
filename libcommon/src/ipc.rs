use crate::Handle;

pub const MAX_TRANSLATE: usize = 4;
pub const TRANSLATE_TYPE_MOVE_HANDLE: u64 = 1;
pub const TRANSLATE_TYPE_COPY_HANDLE: u64 = 2;

#[derive(Debug)]
pub struct IPCHeader {
    pub id: u32,
    pub size: usize,
    pub translate_count: usize,
}

#[repr(transparent)]
#[derive(Debug)]
pub struct TranslateCopyHandle(pub Handle);

#[repr(transparent)]
#[derive(Debug)]
pub struct TranslateMoveHandle(pub Handle);

#[derive(Copy, Clone, Debug)]
pub enum TranslateEntry {
    None,
    MoveHandle(Handle),
    CopyHandle(Handle),
    MemoryStatic(),
    MemoryMap(),
}

impl TranslateEntry {
    pub fn read(buffer: &[u8; 16]) -> TranslateEntry {
        let translate_type = u64::from_le_bytes(buffer[0..8].try_into().unwrap());
        let translate_payload = u64::from_le_bytes(buffer[8..16].try_into().unwrap());

        match translate_type {
            TRANSLATE_TYPE_MOVE_HANDLE => {
                TranslateEntry::MoveHandle(Handle(translate_payload as u32))
            }
            TRANSLATE_TYPE_COPY_HANDLE => {
                TranslateEntry::CopyHandle(Handle(translate_payload as u32))
            }
            _ => {
                unimplemented!();
            }
        }
    }

    pub fn write(buffer: &mut [u8], entry: TranslateEntry) {
        match entry {
            TranslateEntry::MoveHandle(handle) => {
                buffer[0..8].copy_from_slice(&u64::to_le_bytes(TRANSLATE_TYPE_MOVE_HANDLE));
                buffer[8..16].copy_from_slice(&u64::to_le_bytes(handle.0 as u64));
            }
            TranslateEntry::CopyHandle(handle) => {
                buffer[0..8].copy_from_slice(&u64::to_le_bytes(TRANSLATE_TYPE_COPY_HANDLE));
                buffer[8..16].copy_from_slice(&u64::to_le_bytes(handle.0 as u64));
            }
            _ => {
                unimplemented!();
            }
        }
    }
}

impl IPCHeader {
    pub fn pack(header: &IPCHeader) -> u32 {
        assert!(header.size < 256);
        assert!(header.translate_count < 256);

        let packed = header.id
            | (((header.size & 0xff) as u32) << 8)
            | (((header.translate_count & 0xff) as u32) << 16)
            | (0xaa << 24);
        packed
    }

    pub fn unpack(packed: u32) -> IPCHeader {
        let message_id = packed & 0xff;
        let message_size = (packed & (0xff << 8)) >> 8;
        let message_translate_count = (packed & (0xff << 16)) >> 16;

        assert!((packed & (0xff << 24)) >> 24 == 0xaa);

        IPCHeader {
            id: message_id,
            size: message_size as usize,
            translate_count: message_translate_count as usize,
        }
    }
}
