mod wav;

pub use wav::*;

use std::io::Result as IOResult;

pub trait Audio {
    fn open(path: impl Into<String>) -> IOResult<Self>
    where
        Self: Sized;

    fn save(&mut self, path: impl Into<String>, overwrite: bool) -> IOResult<()>;

    fn sample_rate(&self) -> u32;

    fn bit_depth(&self) -> u8;

    fn channels(&self) -> u8;
}
