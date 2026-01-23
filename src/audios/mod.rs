mod wav;
pub use wav::*;

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Result as IOResult};

#[derive(Clone, Debug)]
pub enum SampleBits {
    I16bits(Vec<i16>),
    I32bits(Vec<i32>),
}

impl SampleBits {
    pub fn len(&self) -> usize {
        match self {
            Self::I16bits(v) => v.len(),
            Self::I32bits(v) => v.len(),
        }
    }

    pub fn write_raw(&mut self, raw: &[u8]) {
        match self {
            Self::I16bits(v) => {
                raw.chunks(2).for_each(|bytes| {
                    let mut cursor = Cursor::new(bytes);
                    v.push(cursor.read_i16::<LittleEndian>().unwrap());
                });
            }

            SampleBits::I32bits(v) => {
                raw.chunks(4).for_each(|bytes| {
                    let mut cursor = Cursor::new(bytes);
                    v.push(cursor.read_i32::<LittleEndian>().unwrap());
                });
            }
        }
    }
}

pub trait Audio {
    fn open(path: impl Into<String>) -> IOResult<Self>
    where
        Self: Sized;

    fn save(&mut self, path: impl Into<String>, overwrite: bool) -> IOResult<()>;

    fn sample_rate(&self) -> u32;

    fn bit_depth(&self) -> u16;

    fn channels(&self) -> u16;

    fn set_volume(&mut self, volume: f32);

    fn write_raw_samples(&mut self, raw: &[u8]);
}
