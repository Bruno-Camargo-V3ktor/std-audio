use std::{
    fs::File,
    io::{self, Cursor, Read, Write},
};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

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

impl FileHeader {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let mut cursor = Cursor::new(bytes);

        Self {
            //
            type_bloc_id: [
                cursor.read_u8().unwrap() as char,
                cursor.read_u8().unwrap() as char,
                cursor.read_u8().unwrap() as char,
                cursor.read_u8().unwrap() as char,
            ],
            file_size: cursor.read_u32::<LittleEndian>().unwrap(),
            format_id: [
                cursor.read_u8().unwrap() as char,
                cursor.read_u8().unwrap() as char,
                cursor.read_u8().unwrap() as char,
                cursor.read_u8().unwrap() as char,
            ],

            //
            format_bloc_id: [
                cursor.read_u8().unwrap() as char,
                cursor.read_u8().unwrap() as char,
                cursor.read_u8().unwrap() as char,
                cursor.read_u8().unwrap() as char,
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
                cursor.read_u8().unwrap() as char,
                cursor.read_u8().unwrap() as char,
                cursor.read_u8().unwrap() as char,
                cursor.read_u8().unwrap() as char,
            ],
            data_size: cursor.read_u32::<LittleEndian>().unwrap(),
        }
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut bytes = Vec::with_capacity(44);

        bytes.write_u8(self.type_bloc_id[0] as u8)?;
        bytes.write_u8(self.type_bloc_id[1] as u8)?;
        bytes.write_u8(self.type_bloc_id[2] as u8)?;
        bytes.write_u8(self.type_bloc_id[3] as u8)?;

        bytes.write_u32::<LittleEndian>(self.file_size)?;

        bytes.write_u8(self.format_id[0] as u8)?;
        bytes.write_u8(self.format_id[1] as u8)?;
        bytes.write_u8(self.format_id[2] as u8)?;
        bytes.write_u8(self.format_id[3] as u8)?;

        //
        bytes.write_u8(self.format_bloc_id[0] as u8)?;
        bytes.write_u8(self.format_bloc_id[1] as u8)?;
        bytes.write_u8(self.format_bloc_id[2] as u8)?;
        bytes.write_u8(self.format_bloc_id[3] as u8)?;

        bytes.write_u32::<LittleEndian>(self.bloc_size)?;
        bytes.write_u16::<LittleEndian>(self.audio_format)?;
        bytes.write_u16::<LittleEndian>(self.nbr_channels)?;
        bytes.write_u32::<LittleEndian>(self.frequency)?;
        bytes.write_u32::<LittleEndian>(self.byte_per_sec)?;
        bytes.write_u16::<LittleEndian>(self.byte_per_bloc)?;
        bytes.write_u16::<LittleEndian>(self.bits_per_sample)?;

        //
        bytes.write_u8(self.data_bloc_id[0] as u8)?;
        bytes.write_u8(self.data_bloc_id[1] as u8)?;
        bytes.write_u8(self.data_bloc_id[2] as u8)?;
        bytes.write_u8(self.data_bloc_id[3] as u8)?;
        bytes.write_u32::<LittleEndian>(self.data_size)?;

        Ok(bytes)
    }
}

#[derive(Clone, Debug)]
struct Payload {
    data: Vec<u8>,
}

impl Payload {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            data: bytes.to_vec(),
        }
    }

    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        Ok(self.data.clone())
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
        let payload = Payload::from_bytes(&bytes[44..]);

        Ok(Self { header, payload })
    }

    fn save(&mut self, path: impl Into<String>, overwrite: bool) -> std::io::Result<()> {
        let mut all_bytes = Vec::new();

        all_bytes.append(&mut self.header.to_bytes()?);
        all_bytes.append(&mut self.payload.to_bytes()?);

        if overwrite {
            let mut file = File::create(path.into())?;
            file.write(&all_bytes)?;
            return Ok(());
        }

        let mut file = File::create_new(path.into())?;
        file.write(&all_bytes)?;
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
}
