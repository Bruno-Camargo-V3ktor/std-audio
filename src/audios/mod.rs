mod wav;

pub use wav::*;

use std::io::Result as IOResult;

#[derive(Clone, Debug)]
pub enum SampleBits {
    I16bits(Vec<i16>),
    I32bits(Vec<i32>),
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
}
