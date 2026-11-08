use crate::sound::{CodecMus, MusInputStream};
use jvm::io::{ByteArrayInputStream, InputStream};
use paulscode::sound::SoundSystemConfig;

pub fn install() { SoundSystemConfig::registerStreamFilter(CodecMus::CLASS_NAME, decodeMus); }

fn decodeMus(url: &str, bytes: Vec<u8>) -> Vec<u8>
{
    let length: usize = bytes.len();
    let mut stream: MusInputStream<ByteArrayInputStream> =
        CodecMus::openInputStream(url, ByteArrayInputStream::new(bytes));
    let mut decoded: Vec<u8> = vec![0; length];
    let mut offset: usize = 0;
    while offset < length
    {
        match stream.readInto(&mut decoded, offset as i32, (length - offset) as i32)
        {
            Ok(read) if read > 0 => offset += read as usize,
            Ok(_) => break,
            Err(throwable) =>
            {
                throwable.printStackTrace();
                break;
            }
        }
    }
    decoded.truncate(offset);
    decoded
}
