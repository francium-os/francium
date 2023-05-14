use crate::block::BlockDevice;
use std::io::SeekFrom;
use std::sync::Arc;
use std::sync::Mutex;

pub struct BlockAdapter {
    upper: Arc<Mutex<dyn BlockDevice + Send>>,
    base_sector: u64,
    offset_bytes: u64,

    current_cache_sector: u64,
    cache: [u8; 1024],
}

impl std::fmt::Debug for BlockAdapter {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        todo!()
    }
}

impl BlockAdapter {
    pub fn new(block: Arc<Mutex<dyn BlockDevice + Send>>, base: u64) -> BlockAdapter {
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
            let mut locked = self.upper.lock().unwrap();
            locked.read_sector(
                self.base_sector + self.offset_bytes / 512,
                &mut self.cache[0..512],
            );
            locked.read_sector(
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

        let cache_hit_max_size = 1024 - cache_offset;
        self.fill_cache();
        let cache_hit_size = std::cmp::min(cache_hit_max_size, buf.len());

        buf[0..cache_hit_size]
            .copy_from_slice(&self.cache[cache_offset..cache_offset + cache_hit_size]);
        self.offset_bytes += cache_hit_size as u64;

        // Should be 0 mod 512.
        let remainder = buf.len() - cache_hit_size;
        if remainder != 0 {
            assert!(remainder % 512 == 0);
            assert!(self.offset_bytes % 512 == 0);

            //println!("Large block read! size = {}, remainder = {}", cache_hit_size, remainder);
            let mut locked = self.upper.lock().unwrap();
            for sector in 0..(remainder / 512) {
                let buf_byte_offset = cache_hit_size + sector * 512;
                locked.read_sector(
                    self.base_sector + self.offset_bytes / 512 + sector as u64,
                    &mut buf[buf_byte_offset..buf_byte_offset + 512],
                );
            }
            self.offset_bytes += remainder as u64;
        }

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
            SeekFrom::End(off) => {
                self.offset_bytes = (self.upper.lock().unwrap().get_size() as i64 + off) as u64;
                Ok(self.offset_bytes)
            }
            SeekFrom::Current(off) => {
                // TODO: Don't have any disks larger than 2**63, I guess
                self.offset_bytes = (self.offset_bytes as i64 + off) as u64;
                Ok(self.offset_bytes)
            }
        }
    }
}
