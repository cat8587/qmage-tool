use crate::bit_writer::{BitWriter, BitWriterError};
use crate::decoder::{QMAGE_DIRS, QMAGE_ORIGINAL_DELTA};
use crate::header::{
    MultiframeData, QMAGE_MAGIC_2, QMAGE_MAGIC_VERSION, QmageCodec, QmageHeader, QmageHeaderError,
    QmagePixelType,
};
use crate::pixels::{Pixel, PixelArray};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct EncoderResult {
    pub header: QmageHeader,
    pub byte_array_1: Vec<u8>,
    pub byte_array_2: Option<Vec<u8>>,
    pub byte_array_3: Vec<u8>,
}

fn should_copy_block(array: &PixelArray, block_x: usize, block_y: usize) -> bool {
    if block_x == 0 || block_x + 4 > array.width() || block_y + 4 > array.height() {
        return false;
    }
    for y in 0..4 {
        for x in 0..4 {
            if array.get_pixel(block_x - 1, block_y + y)
                != array.get_pixel(block_x + x, block_y + y)
            {
                return false;
            }
        }
    }

    return true;
}

fn block_size_with_mode(
    array: &PixelArray,
    block_x: usize,
    block_y: usize,
    mode: usize,
    deltas: &[u16],
) -> usize {
    let width = array.width();
    let height = array.height();

    let mut total = 0;
    for j in 0..4 {
        for i in 0..4 {
            if (block_x + i) < width && (block_y + j) < height {
                let pixel = array.get_pixel(block_x + i, block_y + j);
                let pixel_value = match pixel {
                    Some(Pixel::RGB565(v)) => v,
                    _ => unreachable!(),
                };
                let mut write_pixel = false;
                if (block_x > 0 || QMAGE_DIRS[mode].0 >= 0)
                    && (block_y > 0 || QMAGE_DIRS[mode].1 >= 0)
                {
                    let other = array.get_pixel(
                        ((block_x + i) as isize + QMAGE_DIRS[mode].0) as usize,
                        ((block_y + j) as isize + QMAGE_DIRS[mode].1) as usize,
                    );
                    let other_pixel = match other {
                        Some(Pixel::RGB565(v)) => v,
                        _ => unreachable!(),
                    };
                    if other_pixel == pixel_value {
                    } else {
                        let delta = other_pixel.wrapping_sub(pixel_value);
                        let index = deltas.iter().position(|d| *d == delta);
                        if let Some(i) = index {
                            let i = i + 2;
                            let bits = i.highest_one().unwrap() - 1;
                            total += bits as usize + 4;
                        } else {
                            write_pixel = true;
                        }
                    }
                } else {
                    write_pixel = true;
                }
                if write_pixel {
                    total += 19;
                }
            }
        }
    }

    total
}

pub fn encode_v16shortindex_first_frame(
    array: &PixelArray,
    frame_time: u16,
    total_frames: usize,
    version: u8,
) -> Result<EncoderResult, EncoderError> {
    if array.width() >= 65536 || array.height() >= 65536 {
        return Err(EncoderError::TooBig(array.width(), array.height()));
    }
    if array.width() == 0 || array.height() == 0 {
        return Err(EncoderError::TooSmall(array.width(), array.height()));
    }
    if array.pixel_type() != QmagePixelType::RGB565 {
        return Err(EncoderError::UnsupportedPixelType(array.pixel_type()));
    }

    let width = array.width();
    let height = array.height();

    let header = QmageHeader {
        magic: QMAGE_MAGIC_2,
        version,
        image_type: QmagePixelType::RGB565,
        is_multiframe: true,
        use_chroma_key: false,
        is_compressed: true,
        codec: QmageCodec::V16ShortIndex,
        qp: 0,
        unknown2: 0,
        bit_depth: false,
        alpha_depth: false,
        custom_lookup_table: false,
        width: width as u16,
        height: height as u16,
        unknown3: 0,
        near_lossless_compression: false,
        unknown4: 0,
        is_9_patched: false,
        is_opaque: true,
        pre_multiplied: false,
        use_indexed_color: false,
        is_grayscale: false,
        multiframe_data: Some(MultiframeData {
            total_length: 0,
            total_frame_count: total_frames as u16,
            alpha_encoder_mode: 0,
            current_frame: 1,
            frame_time,
            unknown5: 0,
            animation_repeats: false,
            unknown6: 0,
            unknown7: 0,
            unknown8: 0,
        }),
        unknown9: None,
        header_size: 24,
    };

    let deltas = QMAGE_ORIGINAL_DELTA[(header.version != QMAGE_MAGIC_VERSION) as usize];

    let mut bit_writer_1 = BitWriter::new();
    let mut bit_writer_2 = BitWriter::new();
    let mut byte_writer: Vec<u8> = Vec::new();

    for y in (0..height).step_by(4) {
        for x in (0..width).step_by(4) {
            if should_copy_block(array, x, y) {
                match bit_writer_1.write_bits8(2, 3) {
                    Err(x) => return Err(EncoderError::WriteFailed(x)),
                    _ => {
                        continue;
                    }
                };
            }
            let mut best_mode = 0;
            let mut best_size = block_size_with_mode(array, x, y, best_mode, &deltas);
            let mut candidate;
            let mut candidate_mode;
            candidate_mode = 1;
            candidate = block_size_with_mode(array, x, y, candidate_mode, &deltas);
            if candidate < best_size {
                best_mode = candidate_mode;
                best_size = candidate;
            }
            candidate_mode = 2;
            candidate = block_size_with_mode(array, x, y, candidate_mode, &deltas);
            if candidate < best_size {
                best_mode = candidate_mode;
                best_size = candidate;
            }
            match bit_writer_1.write_bits8(2, best_mode as u8) {
                Err(x) => return Err(EncoderError::WriteFailed(x)),
                _ => {}
            };
            let cbp_position = byte_writer.len();
            let mut cbp = 0u16;
            let mut index = 0;
            byte_writer.push(0);
            byte_writer.push(0);
            for j in 0..4 {
                for i in 0..4 {
                    if (x + i) < width && (y + j) < height {
                        let mut write_pixel = false;
                        let pixel = array.get_pixel(x + i, y + j);
                        let pixel_value = match pixel {
                            Some(Pixel::RGB565(v)) => v,
                            _ => unreachable!(),
                        };
                        let other = array.get_pixel(
                            ((x + i) as isize + QMAGE_DIRS[best_mode].0) as usize,
                            ((y + j) as isize + QMAGE_DIRS[best_mode].1) as usize,
                        );
                        if (x > 0 || QMAGE_DIRS[best_mode].0 >= 0)
                            && (y > 0 || QMAGE_DIRS[best_mode].1 >= 0)
                            && other.is_some()
                        {
                            let other_pixel = match other {
                                Some(Pixel::RGB565(v)) => v,
                                _ => unreachable!(),
                            };
                            if other_pixel == pixel_value {
                                cbp |= 1 << index;
                            } else {
                                let delta = pixel_value.wrapping_sub(other_pixel);
                                let index = deltas.iter().position(|d| *d == delta);
                                if let Some(i) = index
                                    && i < 253
                                {
                                    let i = i + 2;
                                    let bits = i.highest_one().unwrap() as u8 - 1;
                                    bit_writer_2
                                        .write_bits8(3, bits)
                                        .map_err(|e| EncoderError::WriteFailed(e))?;
                                    bit_writer_1
                                        .write_bits8(bits as usize + 1, i as u8 - (2 << bits))
                                        .map_err(|e| EncoderError::WriteFailed(e))?;
                                } else {
                                    write_pixel = true;
                                }
                            }
                        } else {
                            write_pixel = true;
                        }
                        if write_pixel {
                            match bit_writer_2.write_bits8(3, 7) {
                                Err(x) => return Err(EncoderError::WriteFailed(x)),
                                _ => {}
                            };
                            byte_writer.push((pixel_value & 0xff) as u8);
                            byte_writer.push((pixel_value >> 8) as u8);
                        }
                        index += 1;
                    }
                }
            }
            byte_writer[cbp_position] = (cbp & 0xff) as u8;
            byte_writer[cbp_position + 1] = (cbp >> 8) as u8;
        }
    }

    Ok(EncoderResult {
        header,
        byte_array_1: bit_writer_1.get_vector(),
        byte_array_2: Some(bit_writer_2.get_vector()),
        byte_array_3: byte_writer,
    })
}

pub fn encode_v16shortindex_identical_frame(
    reference: &PixelArray,
    header: &QmageHeader,
    index: usize,
) -> Result<EncoderResult, EncoderError> {
    if reference.pixel_type() != QmagePixelType::RGB565 {
        return Err(EncoderError::UnsupportedPixelType(reference.pixel_type()));
    }
    let mut header = header.clone();
    if let Some(md) = header.multiframe_data.as_mut() {
        md.current_frame = (index + 1) as u16;
    } else {
        return Err(EncoderError::NoMultiframeData);
    }
    let width = header.width as usize;
    let height = header.height as usize;

    let mut bit_writer = BitWriter::new();
    let mut byte_writer: Vec<u8> = Vec::new();

    for y in (0..height).step_by(16) {
        for x in (0..width).step_by(16) {
            if x <= (width - 16) && y <= (height - 16) {
                match bit_writer.write_bits8(2, 3) {
                    Err(x) => return Err(EncoderError::WriteFailed(x)),
                    _ => {}
                };
            } else {
                for j in (0..16).step_by(4) {
                    for i in (0..16).step_by(4) {
                        let is_inside = (x + i) <= (width - 4) && (y + j) <= (height - 4);
                        if is_inside {
                            match bit_writer.write_bits8(2, 0) {
                                Err(x) => return Err(EncoderError::WriteFailed(x)),
                                _ => {}
                            };
                        }

                        for l in 0..4 {
                            for k in 0..4 {
                                if (x + i + k) < width && (y + j + l) < height {
                                    if is_inside {
                                        match bit_writer.write_bits8(4, 7) {
                                            Err(x) => return Err(EncoderError::WriteFailed(x)),
                                            _ => {}
                                        };
                                    }
                                    let pixel = reference.get_pixel(x + i + k, y + j + l);
                                    let pixel_value = match pixel {
                                        Some(Pixel::RGB565(v)) => v,
                                        _ => unreachable!(),
                                    };
                                    byte_writer.push((pixel_value & 0xff) as u8);
                                    byte_writer.push((pixel_value >> 8) as u8);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(EncoderResult {
        header,
        byte_array_1: bit_writer.get_vector(),
        byte_array_2: None,
        byte_array_3: byte_writer,
    })
}

fn align_to_4bytes(buffer: &mut Vec<u8>, base: usize) {
    while ((buffer.len() - base) % 4) != 0 {
        buffer.push(0);
    }
}

pub fn write_result_to_buffer(
    mut result: EncoderResult,
    buffer: &mut Vec<u8>,
) -> Result<(usize, Option<usize>), EncoderError> {
    result
        .header
        .is_valid()
        .map_err(|e| EncoderError::HeaderError(e))?;
    let header_index = buffer.len();
    buffer.append(&mut vec![0; result.header.header_size]);

    let pointers_offset = buffer.len();
    buffer.append(&mut vec![0; 8]);

    buffer.append(&mut result.byte_array_1);
    align_to_4bytes(buffer, header_index);
    let mut byte_array_2_offset = 0;
    if let Some(e) = result.byte_array_2.as_mut() {
        byte_array_2_offset = buffer.len();
        buffer.append(e);
        align_to_4bytes(buffer, header_index);
    }
    let byte_array_3_offset = buffer.len();
    buffer.append(&mut result.byte_array_3);
    align_to_4bytes(buffer, header_index);

    if result.byte_array_2.is_some() {
        buffer[pointers_offset..pointers_offset + 4]
            .clone_from_slice(&((byte_array_2_offset - header_index) as u32).to_le_bytes());
        buffer[pointers_offset + 4..pointers_offset + 8]
            .clone_from_slice(&((byte_array_3_offset - header_index) as u32).to_le_bytes());
    } else {
        buffer[pointers_offset..pointers_offset + 4]
            .clone_from_slice(&((byte_array_3_offset - header_index) as u32).to_le_bytes());
    }

    let record_size = if let Some(md) = result.header.multiframe_data.as_mut() {
        let record_size = buffer.len() - header_index;
        let is_divided_by_4 =
            result.header.version > QMAGE_MAGIC_VERSION && result.byte_array_2.is_some();

        let total_length_recorderd = if is_divided_by_4 {
            record_size / 4
        } else {
            record_size
        };

        if result.header.version > QMAGE_MAGIC_VERSION {
            if total_length_recorderd > 0xffff {
                if is_divided_by_4 {
                    return Err(EncoderError::RecordTooBig(record_size, 262140));
                } else {
                    return Err(EncoderError::RecordTooBig(record_size, 65535));
                }
            }
        } else {
            if total_length_recorderd > 0xffffffff {
                return Err(EncoderError::RecordTooBig(record_size, 4294967295));
            }
        }

        md.total_length = total_length_recorderd as u32;
        Some(record_size)
    } else {
        None
    };

    result
        .header
        .write_to_buffer(&mut buffer[header_index..header_index + result.header.header_size])
        .unwrap_or_else(|_| unreachable!());

    Ok((header_index, record_size))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Error)]
pub enum EncoderError {
    #[error("Image is {0}x{1} while the maximum is 65535x65535")]
    TooBig(usize, usize),

    #[error("Image is {0}x{1} while the minimum is 1x1")]
    TooSmall(usize, usize),

    #[error("Unsupported pixel type {0:?}")]
    UnsupportedPixelType(QmagePixelType),

    #[error("{0:?}")]
    WriteFailed(BitWriterError),

    #[error("No multiframe data field in provided header")]
    NoMultiframeData,

    #[error("Record is {0} bytes while the maximum is {1} bytes")]
    RecordTooBig(usize, usize),

    #[error("{0:?}")]
    HeaderError(QmageHeaderError),
}
