use std::{
    fs::File,
    io::{self, Cursor, Read, Write},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::SampleBits;

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
    type_bloc_id: [u8; 4],
    file_size: u32,
    format_id: [u8; 4],

    //Chunk describing the data format
    format_bloc_id: [u8; 4],
    bloc_size: u32,
    audio_format: u16,
    nbr_channels: u16,
    frequency: u32,
    byte_per_sec: u32,
    byte_per_bloc: u16,
    bits_per_sample: u16,

    //Chunk containing the sampled data
    data_bloc_id: [u8; 4],
    data_size: u32,
}

impl FileHeader {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut cursor = Cursor::new(bytes);

        Self {
            //
            type_bloc_id: [
                cursor.read_u8().unwrap(),
                cursor.read_u8().unwrap(),
                cursor.read_u8().unwrap(),
                cursor.read_u8().unwrap(),
            ],
            file_size: cursor.read_u32::<LittleEndian>().unwrap(),
            format_id: [
                cursor.read_u8().unwrap(),
                cursor.read_u8().unwrap(),
                cursor.read_u8().unwrap(),
                cursor.read_u8().unwrap(),
            ],

            //
            format_bloc_id: [
                cursor.read_u8().unwrap(),
                cursor.read_u8().unwrap(),
                cursor.read_u8().unwrap(),
                cursor.read_u8().unwrap(),
            ],
            bloc_size: cursor.read_u32::<LittleEndian>().unwrap(),
            audio_format: cursor.read_u16::<LittleEndian>().unwrap(),
            nbr_channels: cursor.read_u16::<LittleEndian>().unwrap(),
            frequency: cursor.read_u32::<LittleEndian>().unwrap(),
            byte_per_sec: cursor.read_u32::<LittleEndian>().unwrap(),
            byte_per_bloc: cursor.read_u16::<LittleEndian>().unwrap(),
            bits_per_sample: cursor.read_u16::<LittleEndian>().unwrap(),

            //
            data_bloc_id: [
                cursor.read_u8().unwrap(),
                cursor.read_u8().unwrap(),
                cursor.read_u8().unwrap(),
                cursor.read_u8().unwrap(),
            ],
            data_size: cursor.read_u32::<LittleEndian>().unwrap(),
        }
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(44);

        bytes.write_all(&self.type_bloc_id)?;
        bytes.write_u32::<LittleEndian>(self.file_size)?;
        bytes.write_all(&self.format_id)?;

        //
        bytes.write_all(&self.format_bloc_id)?;
        bytes.write_u32::<LittleEndian>(self.bloc_size)?;
        bytes.write_u16::<LittleEndian>(self.audio_format)?;
        bytes.write_u16::<LittleEndian>(self.nbr_channels)?;
        bytes.write_u32::<LittleEndian>(self.frequency)?;
        bytes.write_u32::<LittleEndian>(self.byte_per_sec)?;
        bytes.write_u16::<LittleEndian>(self.byte_per_bloc)?;
        bytes.write_u16::<LittleEndian>(self.bits_per_sample)?;

        //
        bytes.write_all(&self.data_bloc_id)?;
        bytes.write_u32::<LittleEndian>(self.data_size)?;

        Ok(bytes)
    }
}

#[derive(Clone, Debug)]
struct Payload {
    samples: SampleBits,
    total_bytes: usize,
}

impl Payload {
    pub fn from_bytes(bits_per_sample: u16, bytes: &[u8]) -> Self {
        let samples = match bits_per_sample {
            16 => {
                let mut samples = Vec::with_capacity(bytes.len());
                bytes.chunks(2).for_each(|v| {
                    let mut cursor = Cursor::new(v);
                    samples.push(cursor.read_i16::<LittleEndian>().unwrap());
                });

                SampleBits::I16bits(samples)
            }

            32 => {
                let mut samples = Vec::with_capacity(bytes.len());
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

// Impls
impl Audio for Wav {
    fn open(path: impl Into<String>) -> std::io::Result<Self>
    where
        Self: Sized,
    {
        let mut file = File::open(path.into())?;
        let mut bytes = Vec::with_capacity(100);

        let _size = file.read_to_end(&mut bytes)?;

        let header = FileHeader::from_bytes(&bytes[0..44]);
        let payload = Payload::from_bytes(header.bits_per_sample, &bytes[44..]);

        Ok(Self { header, payload })
    }

    fn save(&mut self, path: impl Into<String>, overwrite: bool) -> std::io::Result<()> {
        let mut file = if overwrite {
            File::create(path.into())?
        } else {
            File::create_new(path.into())?
        };

        println!("{}", self.header.data_size);
        println!("{}", self.header.file_size);

        let payload_bytes = self.payload.to_bytes()?;
        self.header.data_size = 26;
        self.header.file_size = 36 + payload_bytes.len() as u32;

        println!("{}", payload_bytes.len());
        println!("{}", self.header.data_size);
        println!("{}", self.header.file_size);

        file.write_all(&self.header.to_bytes()?)?;
        file.write_all(&mut self.payload.to_bytes()?)?;

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

// tests
#[cfg(test)]
mod tests {
    use crate::{Audio, Wav};

    #[test]
    pub fn test_duplicate_file() {
        let mut audio = Wav::open("./audios/suzume_no_tojimari.wav").unwrap();
        audio.save("./audios/duplicate2.wav", true).unwrap();

        assert_eq!(audio.bit_depth(), 16);
        assert_eq!(audio.channels(), 2);
        assert_eq!(audio.sample_rate(), 44100);

        let audio2 = Wav::open("./audios/duplicate2.wav").unwrap();
        assert_eq!(audio2.bit_depth(), 16);
        assert_eq!(audio2.channels(), 2);
        assert_eq!(audio2.sample_rate(), 44100);
    }

    #[test]
    pub fn test_up_volume() {
        let mut audio = Wav::open("./audios/duplicate.wav").unwrap();

        println!("{}", audio.payload.to_bytes().unwrap().len());
        audio.set_volume(0.5);

        audio.save("./audios/duplicate.wav", true).unwrap();

        let audio = Wav::open("./audios/duplicate.wav").unwrap();
        println!("{}", audio.payload.to_bytes().unwrap().len());
    }
}
