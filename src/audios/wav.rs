use super::Audio;

#[derive(Clone, Debug)]
pub struct Wav {
    header: FileHeader,
    payload: Payload,
}

#[derive(Clone, Debug)]
/// 44 bytes in memory || RIFF WAVE ESPECIFICATION
struct FileHeader {
    //Master RIFF chunk
    type_bloc_id: [char; 4],
    file_size: u32,
    format_id: [char; 4],

    //Chunk describing the data format
    format_bloc_id: [char; 4],
    bloc_size: u32,
    audio_format: u16,
    nbr_channels: u16,
    frequency: u32,
    byte_per_sec: u32,
    byte_per_bloc: u16,
    bits_per_sample: u16,

    //Chunk containing the sampled data
    data_bloc_id: [char; 4],
    data_size: u32,
}

#[derive(Clone, Debug)]
struct Payload {
    data: Vec<u8>,
}

// Impls
impl Audio for Wav {
    fn open(path: impl Into<String>) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn save(&mut self, path: impl Into<String>, overwrite: bool) -> std::io::Result<()> {
        todo!()
    }

    fn sample_rate(&self) -> u32 {
        todo!()
    }

    fn channels(&self) -> u8 {
        todo!()
    }

    fn bit_depth(&self) -> u8 {
        todo!()
    }
}
