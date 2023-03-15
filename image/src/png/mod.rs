//! Implements a [PNG](https://www.w3.org/TR/png) decoder

// The chunk types don't necessarily start with uppercase characters and renaming them would be silly
#![allow(non_upper_case_globals)]

pub mod chunks;

use anyhow::{Context, Result};
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;
use thiserror::Error;

use compression::zlib;

use hash::CRC32;

const PNG_HEADER: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

const IHDR: [u8; 4] = [73, 72, 68, 82];
const PLTE: [u8; 4] = [80, 76, 84, 69];
const IDAT: [u8; 4] = [73, 68, 65, 84];
const IEND: [u8; 4] = [73, 69, 78, 68];
const cHRM: [u8; 4] = [99, 72, 82, 77];
const dSIG: [u8; 4] = [100, 83, 73, 71];
const eXIF: [u8; 4] = [101, 88, 73, 102];
const gAMA: [u8; 4] = [103, 65, 77, 65];
const hIST: [u8; 4] = [104, 73, 83, 84];
const iCCP: [u8; 4] = [105, 67, 67, 80];
const iTXt: [u8; 4] = [105, 84, 88, 116];
const pHYs: [u8; 4] = [112, 72, 89, 115];
const sBIT: [u8; 4] = [115, 66, 73, 84];
const sPLT: [u8; 4] = [115, 80, 76, 84];
const sRGB: [u8; 4] = [115, 82, 71, 66];
const sTER: [u8; 4] = [115, 84, 69, 82];
const tEXt: [u8; 4] = [116, 69, 88, 116];
const tIME: [u8; 4] = [116, 73, 77, 69];
const tRNS: [u8; 4] = [116, 82, 78, 83];
const zTXt: [u8; 4] = [122, 84, 88, 116];

#[derive(Error, Debug)]
pub enum PNGError {
    #[error("The given file is not a png file")]
    NotAPng,
    #[error("Expected a IHDR block, found {:?}", .0)]
    ExpectedIHDR(Chunk),
    #[error("Unknown Chunktype: {:?}", String::from_utf8_lossy(.0))]
    UnknownChunk([u8; 4]),
    #[error("Mismatched CRC32, expected 0x{expected:0>8x}, found 0x{found:0>8x}")]
    MismatchedChecksum { expected: u32, found: u32 },
    #[error("Unexpected block length, expected 0x{expected:0>8x}, found 0x{found:0>8x}")]
    IncorrectChunkLengthExpectedExactly { expected: usize, found: usize },
    #[error("IEND chunk must not contain data")]
    NonEmptyIEND,
    #[error("Unexpected IDAT chunk, IDAT chunk's must be consecutive")]
    NonConsecutiveIDATChunk,
    #[error("Expected the length of the decompressed zlib stream ({}) to be a multiple of the scanline width plus the filter byte ({})", .0, .1)]
    MismatchedDecompressedZlibSize(usize, usize),
    #[error("Unknown filter method: {}", .0)]
    UnknownFilterType(u8),
    #[error("Image is color-indexed but does not contain a PLTE chunk")]
    IndexedImageWithoutPLTE,
}

pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<canvas::Canvas> {
    let mut file_contents = vec![];
    fs::File::open(&path)
        .with_context(|| format!("reading png data from {}", path.as_ref().display()))?
        .read_to_end(&mut file_contents)?;
    decode(&file_contents)
}

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub enum Chunk {
    /// Image Header
    IHDR(chunks::ImageHeader),
    /// Color Palette
    PLTE(chunks::Palette),
    /// Image Data
    IDAT(chunks::ImageData),
    /// Image End
    IEND,
    cHRM(chunks::Chromacities),
    /// Digital Signatures
    dSIG,
    /// Exif Metadata
    eXIf,
    gAMA,
    /// Color Histogram
    hIST,
    /// ICC color profile
    iCCP,
    iTXt,
    pHYs,
    sBIT,
    sPLT,
    sRGB,
    sTER,
    tEXt,
    tIME,
    tRNS,
    zTXt,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ParserStage {
    BeforeIDAT,
    DuringIDAT,
    AfterIDAT,
}

pub fn decode(bytes: &[u8]) -> Result<canvas::Canvas> {
    let mut reader = Cursor::new(bytes);

    let mut signature = [0; 8];
    reader.read_exact(&mut signature)?;

    if signature != PNG_HEADER {
        return Err(PNGError::NotAPng.into());
    }

    let ihdr_chunk = read_chunk(&mut reader)?;
    let image_header = if let Chunk::IHDR(image_header) = ihdr_chunk {
        image_header
    } else {
        return Err(PNGError::ExpectedIHDR(ihdr_chunk).into());
    };

    let mut parser_stage = ParserStage::BeforeIDAT;
    let mut idat = vec![];
    let mut palette = None;

    loop {
        let chunk = read_chunk(&mut reader)?;

        if parser_stage == ParserStage::DuringIDAT && !matches!(chunk, Chunk::IDAT(_)) {
            parser_stage = ParserStage::AfterIDAT;
        }

        match chunk {
            Chunk::IEND => break,
            Chunk::IDAT(data) => {
                match parser_stage {
                    ParserStage::BeforeIDAT => parser_stage = ParserStage::DuringIDAT,
                    ParserStage::AfterIDAT => return Err(PNGError::NonConsecutiveIDATChunk.into()),
                    _ => {},
                }
                idat.extend(data.bytes());
            },
            Chunk::PLTE(plte) => palette = Some(plte),
            _ => {},
        }
    }

    if image_header.image_type == chunks::ihdr::ImageType::IndexedColor && palette.is_none() {
        return Err(PNGError::IndexedImageWithoutPLTE.into());
    }

    let decompressed_body = zlib::decode(&idat).context("Failed to decompress PNG image data")?;

    let scanline_width = image_header.width as usize * image_header.image_type.pixel_width();

    // NOTE: need to add 1 here because each scanline also contains a byte specifying a filter type
    if decompressed_body.len() % (scanline_width + 1) != 0 {
        return Err(PNGError::MismatchedDecompressedZlibSize(
            decompressed_body.len(),
            scanline_width + 1,
        )
        .into());
    }

    let mut image_data = vec![0; image_header.height as usize * scanline_width];
    apply_filters(
        &decompressed_body,
        &mut image_data,
        scanline_width,
        image_header.image_type.pixel_width(),
    )?;

    Ok(canvas::Canvas::new(
        image_data,
        image_header.width as usize,
        image_header.height as usize,
        image_header.image_type.into(),
    ))
}

fn read_chunk<R: Read>(reader: &mut R) -> Result<Chunk> {
    let mut length_bytes = [0; 4];
    reader.read_exact(&mut length_bytes)?;
    let length = u32::from_be_bytes(length_bytes) as usize;

    let mut chunk_name_bytes = [0; 4];
    reader.read_exact(&mut chunk_name_bytes)?;

    let mut data = vec![0; length];
    reader.read_exact(data.as_mut_slice())?;

    let mut crc_bytes = [0; 4];
    reader.read_exact(&mut crc_bytes)?;
    let expected_crc = u32::from_be_bytes(crc_bytes);

    let mut hasher = CRC32::default();
    hasher.write(&chunk_name_bytes);
    hasher.write(&data);
    let computed_crc = hasher.finish();

    if expected_crc != computed_crc {
        return Err(PNGError::MismatchedChecksum {
            expected: expected_crc,
            found: computed_crc,
        }
        .into());
    }

    let chunk = match chunk_name_bytes {
        IHDR => {
            if length != 13 {
                return Err(PNGError::IncorrectChunkLengthExpectedExactly {
                    expected: 13,
                    found: length,
                }
                .into());
            }

            Chunk::IHDR(chunks::ImageHeader::new(
                u32::from_be_bytes(data[0..4].try_into().unwrap()),
                u32::from_be_bytes(data[4..8].try_into().unwrap()),
                data[8],
                data[9].try_into()?,
                data[10],
                data[11],
                data[12].try_into()?,
            )?)
        },
        PLTE => Chunk::PLTE(chunks::Palette::new(&data)?),
        IDAT => Chunk::IDAT(chunks::ImageData::new(data)),
        IEND => {
            if length != 0 {
                return Err(PNGError::NonEmptyIEND.into());
            }

            Chunk::IEND
        },
        cHRM => {
            if length != 32 {
                return Err(PNGError::IncorrectChunkLengthExpectedExactly {
                    expected: 32,
                    found: length,
                }
                .into());
            }

            let white_point = (
                u32::from_be_bytes(data[0..4].try_into().unwrap()),
                u32::from_be_bytes(data[4..8].try_into().unwrap()),
            );
            let red_point = (
                u32::from_be_bytes(data[8..12].try_into().unwrap()),
                u32::from_be_bytes(data[12..16].try_into().unwrap()),
            );
            let green_point = (
                u32::from_be_bytes(data[16..20].try_into().unwrap()),
                u32::from_be_bytes(data[20..24].try_into().unwrap()),
            );
            let blue_point = (
                u32::from_be_bytes(data[24..28].try_into().unwrap()),
                u32::from_be_bytes(data[28..32].try_into().unwrap()),
            );
            Chunk::cHRM(chunks::Chromacities::new(
                white_point,
                red_point,
                green_point,
                blue_point,
            ))
        },
        dSIG => Chunk::dSIG,
        eXIF => Chunk::eXIf,
        gAMA => Chunk::gAMA,
        hIST => Chunk::hIST,
        iCCP => Chunk::iCCP,
        iTXt => Chunk::iTXt,
        pHYs => Chunk::pHYs,
        sBIT => Chunk::sBIT,
        sPLT => Chunk::sPLT,
        sRGB => Chunk::sRGB,
        sTER => Chunk::sTER,
        tEXt => Chunk::tEXt,
        tIME => Chunk::tIME,
        tRNS => Chunk::tRNS,
        zTXt => Chunk::zTXt,
        _ => return Err(PNGError::UnknownChunk(chunk_name_bytes).into()),
    };

    Ok(chunk)
}

/// Apply one of the filter specified in <https://www.w3.org/TR/png/#9-table91> to a scanline
fn apply_filters(
    from: &[u8],
    to: &mut [u8],
    scanline_width: usize,
    pixel_width: usize,
) -> Result<()> {
    // For each scanline, apply a filter (which is the first byte) to the scanline data (which is the rest)
    for (index, scanline_data_and_filter_method) in
        from.chunks_exact(scanline_width + 1).enumerate()
    {
        let (filter_type, filter_scanline_data) = (
            scanline_data_and_filter_method[0],
            &scanline_data_and_filter_method[1..],
        );

        let scanline_base_index = index * scanline_width;
        match filter_type {
            0 => {
                // None
                to[scanline_base_index..][..scanline_width].copy_from_slice(filter_scanline_data);
            },
            1 => {
                // Sub
                // First pixel always stays the same
                to[scanline_base_index..][..pixel_width]
                    .copy_from_slice(&filter_scanline_data[..pixel_width]);
                for i in pixel_width..filter_scanline_data.len() {
                    to[scanline_base_index + i] = filter_scanline_data[i]
                        .wrapping_add(to[scanline_base_index + i - pixel_width]);
                }
            },
            2 => {
                // Up
                if index == 0 {
                    // First row stays the same
                    to[scanline_base_index..][..scanline_width]
                        .copy_from_slice(filter_scanline_data);
                } else {
                    for i in 0..scanline_width {
                        to[scanline_base_index + i] = filter_scanline_data[i]
                            .wrapping_add(to[scanline_base_index - scanline_width + i]);
                    }
                }
            },
            3 => {
                // Average
                todo!("Average filter type")
            },
            4 => {
                // Paeth
                todo!("Paeth filter type")
            },
            _ => return Err(PNGError::UnknownFilterType(filter_type).into()),
        }
    }
    Ok(())
}