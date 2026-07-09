const SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";

#[derive(Debug)]
pub enum PngError {
    BadSignature,
    UnexpectedEof,
    InvalidIhdr,
}

pub struct Chunk {
    length: u32,
    chunk_type: [u8; 4],
    data: Vec<u8>,
    crc: u32,
}

pub struct PngImage {
    pub ihdr: Ihdr,
    idat_data: Vec<u8>,
}

pub struct Ihdr {
    pub width: u32,
    pub height: u32,
    bit_depth: u8,
    color_type: u8,
    compression: u8,
    filter: u8,
    interlace: u8,
}

pub fn parse_png(bytes: &[u8], debug: bool) -> Result<PngImage, PngError> {
    if debug {
        println!("read {} bytes", bytes.len());
    }

    if !bytes.starts_with(SIGNATURE) {
        return Err(PngError::BadSignature);
    }

    if debug {
        println!("sig match");
    }

    let chunks = parse_chunks(&bytes[8..], debug)?;
    interpret_chunks(&chunks, debug)
}

fn parse_chunks(bytes: &[u8], debug: bool) -> Result<Vec<Chunk>, PngError> {
    let mut chunks = Vec::new();
    let mut offset = 0;

    while offset < bytes.len() {
        // need at least 8 bytes for len + type
        if offset + 8 > bytes.len() {
            return Err(PngError::UnexpectedEof);
        }

        let length = u32::from_be_bytes(bytes[offset..offset + 4].try_into().unwrap());
        let chunk_type: [u8; 4] = bytes[offset + 4..offset + 8].try_into().unwrap();

        let data_start = offset + 8;
        let data_end = data_start + length as usize;

        if data_end + 4 > bytes.len() {
            return Err(PngError::UnexpectedEof);
        }

        let data = bytes[data_start..data_end].to_vec();
        let crc = u32::from_be_bytes(bytes[data_end..data_end + 4].try_into().unwrap());

        if debug {
            println!(
                "chunk: type={} length={} crc={:#010x}",
                std::str::from_utf8(&chunk_type).unwrap_or("????"),
                length,
                crc
            );
        }

        chunks.push(Chunk {
            length,
            chunk_type,
            data,
            crc,
        });
        offset = data_end + 4;

        if &chunk_type == b"IEND" {
            break;
        }
    }

    Ok(chunks)
}

fn parse_ihdr(data: &[u8]) -> Result<Ihdr, PngError> {
    if data.len() != 13 {
        return Err(PngError::InvalidIhdr);
    }

    Ok(Ihdr {
        width: u32::from_be_bytes(data[0..4].try_into().unwrap()),
        height: u32::from_be_bytes(data[4..8].try_into().unwrap()),
        bit_depth: data[8],
        color_type: data[9],
        compression: data[10],
        filter: data[11],
        interlace: data[12],
    })
}

fn interpret_chunks(chunks: &[Chunk], debug: bool) -> Result<PngImage, PngError> {
    // first chunk MUST be IHDR
    let ihdr = parse_ihdr(&chunks[0].data)?;

    // collect IDAT data
    let idat_data: Vec<u8> = chunks
        .iter()
        .filter(|c| &c.chunk_type == b"IDAT")
        .flat_map(|c| c.data.iter().copied())
        .collect();

    if debug {
        println!(
            "IHDR: {}x{} depth={} color_type={} compression={} filter={} interlace={}",
            ihdr.width,
            ihdr.height,
            ihdr.bit_depth,
            ihdr.color_type,
            ihdr.compression,
            ihdr.filter,
            ihdr.interlace
        );
        println!("IDAT: {} bytes total", idat_data.len());
    }

    Ok(PngImage { ihdr, idat_data })
}
