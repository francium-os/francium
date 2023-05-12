use crate::block::BlockDevice;
use std::io::SeekFrom;

pub struct BlockAdapter {
    upper: Box<dyn BlockDevice + Send>,
    base_sector: u64,
    offset_bytes: u64,

    current_cache_sector: u64,
    cache: [u8; 1024],
}

impl std::fmt::Debug for BlockAdapter {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> { todo!() }
}

impl BlockAdapter {
    pub fn new(block: Box<dyn BlockDevice + Send>, base: u64) -> BlockAdapter {
        BlockAdapter {
            upper: block,
            base_sector: base,
            offset_bytes: 0,
            current_cache_sector: u64::MAX,
            cache: [0; 1024],
        }
    }

    fn fill_cache(&mut self) {
        if self.offset_bytes / 512 != self.current_cache_sector {
            self.upper.read_sector(
                self.base_sector + self.offset_bytes / 512,
                &mut self.cache[0..512],
            );
            self.upper.read_sector(
                self.base_sector + self.offset_bytes / 512 + 1,
                &mut self.cache[512..1024],
            );

            self.current_cache_sector = self.offset_bytes / 512;

            //println!("Filling cache: {:x?}", self.offset_bytes);
            //println!("{:x?}", self.cache);
        }
    }
}

impl std::io::Read for BlockAdapter {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let cache_offset: usize = (self.offset_bytes % 512) as usize;

        assert!(buf.len() < 1024 - cache_offset);

        //println!("Read! off= 0x{:x} len= 0x{:x}", self.offset_bytes, buf.len());
        self.fill_cache();

        buf.copy_from_slice(&self.cache[cache_offset..cache_offset + buf.len()]);
        self.offset_bytes += buf.len() as u64;

        //println!("{:#x?}", buf);

        Ok(buf.len())
    }
}

impl std::io::Write for BlockAdapter {
    fn write(&mut self, _: &[u8]) -> Result<usize, std::io::Error> {
        todo!()
    }
    fn flush(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

impl std::io::Seek for BlockAdapter {
    fn seek(&mut self, from: SeekFrom) -> Result<u64, std::io::Error> {
        //println!("Seek: {:x?}", from);
        match from {
            SeekFrom::Start(off) => {
                self.offset_bytes = off;
                Ok(self.offset_bytes)
            }
            SeekFrom::End(_off) => {
                panic!();
            }
            SeekFrom::Current(off) => {
                // TODO: Don't have any disks larger than 2**63, I guess
                self.offset_bytes = (self.offset_bytes as i64 + off) as u64;
                Ok(self.offset_bytes)
            }
        }
    }
}
