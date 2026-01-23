use super::Audio;
use crate::SampleBits;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    fs::File,
    io::{self, Cursor, Read, Write},
};

#[derive(Clone, Debug)]
pub struct Wav {
    //Master RIFF chunk
    //type_bloc_id: [u8; 4] -> RIFF,
    //file_size: u32,
    //format_id: [u8; 4] -> WAVE,
    file_size: u32,
    header: FileHeader,
    metadata: Metadata,
    payload: Payload,
}

impl Default for Wav {
    fn default() -> Self {
        Self {
            file_size: 32,
            header: FileHeader::default(),
            metadata: Metadata::default(),
            payload: Payload::default(),
        }
    }
}

#[derive(Clone, Debug)]
/// RIFF WAVE ESPECIFICATION
struct FileHeader {
    //Chunk describing the data format
    //format_bloc_id: [u8; 4] -> fmt_,
    bloc_size: u32,
    audio_format: u16,
    nbr_channels: u16,
    frequency: u32,
    byte_per_sec: u32,
    byte_per_bloc: u16,
    bits_per_sample: u16,
}

#[derive(Clone, Debug)]
struct Metadata {
    pub data: Vec<u8>,
}

#[derive(Clone, Debug)]
struct Payload {
    //Chunk containing the sampled data
    //data_bloc_id: [u8; 4] -> data,
    samples: SampleBits,
    total_bytes: usize,
}

// Impl Wav
impl Audio for Wav {
    fn open(path: impl Into<String>) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut file = File::open(path.into())?;
        let mut bytes = Vec::with_capacity(100);

        let _size = file.read_to_end(&mut bytes)?;

        let mut cursor = Cursor::new(&bytes[4..8]);

        let header = FileHeader::from_bytes(&bytes[12..36]);
        let (metadata, index) = Metadata::from_bytes(&bytes[36..]);
        let payload = Payload::from_bytes(header.bits_per_sample, &bytes[36 + index..]);

        Ok(Self {
            file_size: cursor.read_u32::<LittleEndian>().unwrap(),
            header,
            metadata,
            payload,
        })
    }

    fn save(&mut self, path: impl Into<String>, overwrite: bool) -> std::io::Result<()> {
        let mut file = if overwrite {
            File::create(path.into())?
        } else {
            File::create_new(path.into())?
        };

        file.write(b"RIFF")?;
        file.write_u32::<LittleEndian>(self.file_size)?;
        file.write(b"WAVE")?;

        file.write_all(&self.header.to_bytes()?)?;
        file.write_all(&self.metadata.to_bytes()?)?;
        file.write_all(&self.payload.to_bytes()?)?;

        file.flush()?;
        Ok(())
    }

    fn sample_rate(&self) -> u32 {
        self.header.frequency
    }

    fn channels(&self) -> u16 {
        self.header.nbr_channels
    }

    fn bit_depth(&self) -> u16 {
        self.header.bits_per_sample
    }

    fn set_volume(&mut self, volume: f32) {
        if volume < 0.0 {
            panic!("Volume is less 0.0");
        }

        match &mut self.payload.samples {
            SampleBits::I16bits(samples) => {
                for sample in samples {
                    let mut value = *sample as f32;
                    value *= volume;

                    *sample = (value.clamp(i16::MIN as f32, i16::MAX as f32)) as i16;
                }
            }

            SampleBits::I32bits(samples) => {
                for sample in samples {
                    let mut value = *sample as f32;
                    value *= volume;

                    *sample = (value.clamp(i32::MIN as f32, i32::MAX as f32)) as i32;
                }
            }
        }
    }
}

// Impl FileHeader
impl FileHeader {
    pub fn new(audio_format: u16, nbr_channels: u16, frequency: u32, bits_per_sample: u16) -> Self {
        let byte_per_bloc = nbr_channels * bits_per_sample / 8;
        let byte_per_sec = frequency * byte_per_bloc as u32;

        Self {
            bloc_size: 16,
            audio_format,
            nbr_channels,
            frequency,
            byte_per_sec,
            byte_per_bloc,
            bits_per_sample,
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut cursor = Cursor::new(bytes);
        let _ = cursor.read_u32::<LittleEndian>();

        Self {
            bloc_size: cursor.read_u32::<LittleEndian>().unwrap(),
            audio_format: cursor.read_u16::<LittleEndian>().unwrap(),
            nbr_channels: cursor.read_u16::<LittleEndian>().unwrap(),
            frequency: cursor.read_u32::<LittleEndian>().unwrap(),
            byte_per_sec: cursor.read_u32::<LittleEndian>().unwrap(),
            byte_per_bloc: cursor.read_u16::<LittleEndian>().unwrap(),
            bits_per_sample: cursor.read_u16::<LittleEndian>().unwrap(),
        }
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(44);

        bytes.write_all(&[102, 109, 116, 32])?; // fmt␣
        bytes.write_u32::<LittleEndian>(self.bloc_size)?;
        bytes.write_u16::<LittleEndian>(self.audio_format)?;
        bytes.write_u16::<LittleEndian>(self.nbr_channels)?;
        bytes.write_u32::<LittleEndian>(self.frequency)?;
        bytes.write_u32::<LittleEndian>(self.byte_per_sec)?;
        bytes.write_u16::<LittleEndian>(self.byte_per_bloc)?;
        bytes.write_u16::<LittleEndian>(self.bits_per_sample)?;

        Ok(bytes)
    }
}

impl Default for FileHeader {
    fn default() -> Self {
        FileHeader::new(1, 2, 44100, 16)
    }
}

// Impl Payload
impl Payload {
    pub fn new(samples: SampleBits) -> Self {
        let size = samples.len();
        Self {
            total_bytes: size,
            samples,
        }
    }

    pub fn from_bytes(bits_per_sample: u16, bytes: &[u8]) -> Self {
        let mut cursor = Cursor::new(&bytes[0..4]);
        let size = cursor.read_u32::<LittleEndian>().unwrap();

        let bytes = &bytes[4..];

        let samples = match bits_per_sample {
            16 => {
                let mut samples = Vec::with_capacity(size as usize);
                bytes.chunks(2).for_each(|v| {
                    let mut cursor = Cursor::new(v);
                    samples.push(cursor.read_i16::<LittleEndian>().unwrap());
                });

                SampleBits::I16bits(samples)
            }

            32 => {
                let mut samples = Vec::with_capacity(size as usize);
                bytes.chunks(4).for_each(|v| {
                    let mut cursor = Cursor::new(v);
                    samples.push(cursor.read_i32::<LittleEndian>().unwrap());
                });

                SampleBits::I32bits(samples)
            }
            _ => panic!(),
        };

        Self {
            samples,
            total_bytes: bytes.len(),
        }
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::with_capacity(self.total_bytes);
        bytes.write_all(b"data")?;
        bytes.write_u32::<LittleEndian>(self.total_bytes as u32)?;

        match &self.samples {
            SampleBits::I16bits(samples) => {
                for sample in samples {
                    bytes.write_i16::<LittleEndian>(*sample)?;
                }
            }

            SampleBits::I32bits(samples) => {
                for sample in samples {
                    bytes.write_i32::<LittleEndian>(*sample)?;
                }
            }
        }

        Ok(bytes)
    }
}

impl Default for Payload {
    fn default() -> Self {
        Self::new(SampleBits::I16bits(vec![]))
    }
}

// Impl Metadata
impl Metadata {
    pub fn from_bytes(bytes: &[u8]) -> (Self, usize) {
        let mut data = Vec::new();

        let block = b"data";
        let mut end = 0;

        for i in 0..bytes.len() {
            let v = &bytes[i..i + 4];

            if v == block {
                end = i + 4;
                break;
            }

            data.push(bytes[i]);
        }

        (Self { data }, end)
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        Ok(self.data.clone())
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata { data: Vec::new() }
    }
}

// tests
#[cfg(test)]
mod tests {
    use crate::{Audio, Wav};

    #[test]
    pub fn test_duplicate_file() {
        let mut audio = Wav::open("./audios/suzume_no_tojimari.wav").unwrap();
        audio.save("./audios/duplicate.wav", true).unwrap();

        assert_eq!(audio.bit_depth(), 16);
        assert_eq!(audio.channels(), 2);
        assert_eq!(audio.sample_rate(), 44100);

        let audio2 = Wav::open("./audios/duplicate.wav").unwrap();
        assert_eq!(audio2.bit_depth(), 16);
        assert_eq!(audio2.channels(), 2);
        assert_eq!(audio2.sample_rate(), 44100);
    }

    #[test]
    pub fn test_up_volume() {
        let mut audio = Wav::open("./audios/suzume_no_tojimari.wav").unwrap();

        audio.set_volume(0.05);

        audio
            .save("./audios/suzume_no_tojimari_x05.wav", true)
            .unwrap();
    }
}
