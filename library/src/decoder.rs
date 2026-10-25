use crate::bit_reader::BitReader;
use crate::header::{QMAGE_MAGIC_VERSION, QmageCodec, QmageHeader, QmagePixelType};
use crate::pixels::{Pixel, PixelArray};
use thiserror::Error;

const QMAGE_ORIGINAL_DELTA: [[u16; 256]; 2] = [
    [
        0xffe0, 0x0020, 0xffff, 0xf7e0, 0xf800, 0xffdf, 0x0001, 0x0800, 0x0021, 0x0820, 0x0841,
        0xf7bf, 0xf7df, 0x0821, 0xf7c0, 0xffbf, 0x0041, 0xffc0, 0x0861, 0x0840, 0x001f, 0x0862,
        0xffe1, 0xf79f, 0x0040, 0xf7ff, 0xf820, 0x0842, 0x1082, 0x07e0, 0xf79e, 0x0801, 0x1061,
        0x1062, 0x07ff, 0xef7e, 0xef9f, 0xf801, 0xf7be, 0xefbf, 0x0022, 0x0882, 0xefe0, 0xefc0,
        0x07df, 0x1041, 0x081f, 0x10a3, 0x0042, 0x1081, 0xef9e, 0xf7e1, 0xf821, 0xf77e, 0xff9f,
        0x10a2, 0x1040, 0xffbe, 0x1083, 0x1020, 0xef5d, 0xef7f, 0xffde, 0xef5e, 0x0002, 0x0061,
        0xf7a0, 0x18c3, 0xef7d, 0x0822, 0xff9e, 0xf000, 0x0860, 0xe73d, 0xefa0, 0x0881, 0xefdf,
        0x07bf, 0xf77f, 0x0062, 0xfffe, 0x18a3, 0x10c3, 0x0883, 0x18e4, 0x1060, 0x0863, 0x1000,
        0x18a2, 0xe71c, 0xf7de, 0x1021, 0x1042, 0x18c4, 0xef3d, 0x18e3, 0xf841, 0xe75d, 0x1881,
        0xe73c, 0xe7c0, 0x18c2, 0xe75e, 0xf77d, 0x10a4, 0x07c0, 0xf81f, 0x10c4, 0xff7e, 0x003f,
        0xf840, 0xe79f, 0x2104, 0xffa0, 0xffc1, 0x18a1, 0xe77f, 0xefbe, 0xf79d, 0x1882, 0x10a1,
        0xe71d, 0x1063, 0xe7a0, 0xe7e0, 0xf7c1, 0x1861, 0x08a3, 0x1840, 0xe73e, 0x083f, 0xf001,
        0xdefc, 0xf75e, 0x0060, 0x2125, 0x07e1, 0x0fff, 0xef5f, 0xe77e, 0xefe1, 0xe6fc, 0x10c2,
        0x08a2, 0x0023, 0x1860, 0xef3c, 0xef3e, 0x0043, 0xf020, 0xf75d, 0x079f, 0x101f, 0xdedb,
        0xef5c, 0xffe2, 0xf822, 0x20e4, 0x001e, 0x0843, 0xe7bf, 0x20e3, 0x0082, 0xdefb, 0xef9d,
        0x1904, 0xff7f, 0x1820, 0xe75f, 0x0063, 0xef80, 0x2105, 0x1905, 0xdefd, 0x2124, 0xdf1c,
        0x2103, 0x1080, 0xdf1d, 0xe6fb, 0x07be, 0x2945, 0xdedc, 0xf021, 0x0fe0, 0xe71b, 0xefff,
        0xf842, 0x20c2, 0x079e, 0xf802, 0x0fdf, 0x20e2, 0x18e2, 0xe780, 0x07de, 0x1880, 0xdfa0,
        0x20c1, 0xffbd, 0x08a4, 0x10e4, 0x18c1, 0xdfc0, 0x103f, 0x0003, 0xdeba, 0xff7d, 0x1841,
        0xe800, 0x20a1, 0xefc1, 0xff9d, 0xf780, 0x18e5, 0x10e3, 0x1084, 0x0081, 0x2966, 0xffdd,
        0x08c3, 0x18a4, 0xdeda, 0xff5e, 0xdf7f, 0x1883, 0xdebb, 0x2145, 0xd6bb, 0xf7bd, 0x20c3,
        0x1903, 0x077e, 0x0884, 0xf861, 0xe71e, 0x1001, 0xe75c, 0x0083, 0xd6ba, 0x07fe, 0x2102,
        0xdf80, 0x2081, 0x0880, 0xdf3e, 0x2946, 0xe6fd, 0x2146, 0xef1c, 0xdf9f, 0xdf1e, 0xe73f,
        0xe79e, 0x20a2, 0xd69a,
    ],
    [
        0xffe0, 0x0020, 0xffff, 0x0001, 0xf800, 0x0800, 0xf7df, 0xffdf, 0xf7e0, 0x0021, 0xf7bf,
        0x0821, 0x0820, 0x0841, 0xf7ff, 0x0801, 0xffc0, 0x07e0, 0xf820, 0xf801, 0x07ff, 0xf7c0,
        0xffe1, 0x0040, 0x001f, 0xffbf, 0xf79f, 0x0840, 0xffff, 0x0041, 0x0861, 0xef7e, 0x1082,
        0xef9e, 0xf81f, 0xf7a0, 0x0001, 0xe73d, 0xef9f, 0x1062, 0x0860, 0x1061, 0x18c3, 0x07df,
        0xf7be, 0xef5e, 0xefbf, 0xf79e, 0xf7e1, 0xf821, 0x10a2, 0xef7f, 0x0842, 0x07e1, 0xdefc,
        0x1041, 0x081f, 0x0862, 0x0002, 0x1081, 0x2104, 0xf81f, 0xefc0, 0x07e1, 0xfffe, 0xffbe,
        0xe75d, 0x2945, 0xffa0, 0xf780, 0x1040, 0xd6bb, 0x0042, 0x18a3, 0x18e3, 0x0880, 0x0822,
        0xff9f, 0x0022, 0xffc1, 0x003f, 0xffde, 0xe71d, 0x3186, 0xef5f, 0xf7de, 0xce7a, 0x0060,
        0x10a1, 0xe75e, 0xf77e, 0xdedc, 0x39c7, 0xf77f, 0xc639, 0x18a2, 0xefe0, 0x2124, 0x1080,
        0x20e4, 0x0061, 0x1020, 0x1000, 0xefa0, 0xff9e, 0x07c0, 0xdf1c, 0xbdf8, 0x1060, 0xf000,
        0x4208, 0xd69b, 0x0881, 0x5acb, 0x2965, 0x0882, 0xef80, 0xf840, 0xe77f, 0xa535, 0xe73e,
        0xce5a, 0x21ae, 0xf760, 0x1881, 0x4a49, 0xefbe, 0xb5b7, 0x0062, 0xf7c1, 0x083f, 0x2925,
        0xe73f, 0x4a4a, 0xef3f, 0x08a0, 0xe75f, 0x1042, 0xad76, 0x18c2, 0x3126, 0xf7fe, 0x528a,
        0xc619, 0x31a6, 0xd6db, 0xff7e, 0x10c1, 0xe77e, 0xceda, 0x18c1, 0xb5b6, 0x07bf, 0xf83f,
        0x630c, 0xde52, 0x39e7, 0xefdf, 0xef5d, 0x1861, 0x10a0, 0xe79f, 0x9cf4, 0xce9a, 0xff7f,
        0x18a1, 0x1021, 0x0802, 0xef7d, 0x1882, 0x3166, 0x0003, 0xf841, 0xf7bd, 0xe71f, 0xdf7f,
        0x10a3, 0x07c1, 0x18e1, 0x94b3, 0x2081, 0xbdd8, 0x6b4d, 0xef60, 0x1083, 0xdf1d, 0xff7d,
        0x0863, 0xf79d, 0xdf3e, 0xf77d, 0x4228, 0xad56, 0xf7dd, 0x20e3, 0x52aa, 0xb597, 0x3967,
        0xc699, 0x20c2, 0x8c72, 0xff80, 0x001e, 0x39a7, 0xe6ff, 0xff5f, 0xc659, 0x0082, 0x1901,
        0x1063, 0x4a69, 0xef9d, 0x0843, 0xd6dc, 0xfffd, 0x738e, 0x07fe, 0x8431, 0xffe2, 0xe7a0,
        0xef40, 0x7bf0, 0xe71c, 0xff9d, 0xef3d, 0xefff, 0x2924, 0xa515, 0xd73e, 0x8410, 0x28c2,
        0x7bcf, 0x1043, 0xf75e, 0x41e8, 0x10c3, 0xbe18, 0x5aeb, 0xd6dd, 0xdf5f, 0x9cd4, 0x0883,
        0x20a2, 0xf802, 0x632c, 0xe780, 0x0083, 0xdf5e, 0xefbd, 0xff5d, 0x0823, 0x1880, 0x1022,
        0xdefd, 0x18e4, 0xefde,
    ],
];

const QMAGE_DIRS: [(isize, isize); 3] = [(-1, 0), (0, -1), (-1, -1)];

fn decode_pixel_in_block<'a>(
    array: &mut PixelArray,
    reference: Option<&PixelArray>,
    bitstream1: &mut BitReader<'a>,
    bitstream2: Option<&mut BitReader<'a>>,
    byte_stream: &mut dyn Iterator<Item = &u8>,
    x: usize,
    y: usize,
    relative_x: isize,
    relative_y: isize,
    copy: Option<bool>,
    deltas: &[u16],
) {
    // TODO: Remove all the .unwrap() calls

    let source_x = (x as isize + relative_x) as usize;
    let source_y = (y as isize + relative_y) as usize;
    let copy = copy.unwrap_or_else(|| bitstream1.read_bits8(1) > 0);
    if copy {
        let pixel = reference
            .unwrap_or(array)
            .get_pixel_zero(source_x, source_y);
        array.set_pixel(x, y, pixel).unwrap();
    } else {
        let n_bits = bitstream2.unwrap_or(bitstream1).read_bits8(3) as usize;
        if n_bits == 7 {
            let byte_a = byte_stream.next().unwrap();
            let byte_b = byte_stream.next().unwrap();
            let pixel = ((*byte_b as u16) << 8) | (*byte_a as u16);
            array.set_pixel(x, y, Pixel::RGB565(pixel)).unwrap();
        } else {
            let index = bitstream1.read_bits8(n_bits + 1) as usize;
            let delta = deltas[index + (2 << n_bits) - 2];
            let pixel = match reference
                .unwrap_or(array)
                .get_pixel_zero(source_x, source_y)
            {
                Pixel::RGB565(p) => p,
                _ => panic!("Not RGB565 pixel array"),
            };
            let pixel = pixel.wrapping_add(delta);
            array.set_pixel(x, y, Pixel::RGB565(pixel)).unwrap();
        }
    }
}

fn copy_block(
    array: &mut PixelArray,
    reference: Option<&PixelArray>,
    source_x: usize,
    source_y: usize,
    dest_x: usize,
    dest_y: usize,
    width: usize,
    height: usize,
) {
    let pixel_size = array.pixel_type().bytes_per_pixel();
    let line_size = pixel_size * array.width();
    let buffer = array.buffer_mut();
    for dy in 0..height {
        let offset_a = (dest_y + dy) * line_size + dest_x * pixel_size;
        if let Some(r) = reference {
            let offset_b = (dest_y + dy) * line_size + (dest_x + width) * pixel_size;
            let ref_pixel_size = r.pixel_type().bytes_per_pixel();
            let ref_line_size = ref_pixel_size * r.width();
            let offset_c = (source_y + dy) * ref_line_size + source_x * ref_pixel_size;
            let offset_d = (source_y + dy) * ref_line_size + (source_x + width) * ref_pixel_size;
            buffer[offset_a..offset_b].copy_from_slice(&r.buffer()[offset_c..offset_d]);
        } else {
            let offset_c = (source_y + dy) * line_size + source_x * pixel_size;
            let offset_d = (source_y + dy) * line_size + (source_x + width) * pixel_size;
            buffer.copy_within(offset_c..offset_d, offset_a);
        }
    }
}

fn copy_edge(
    array: &mut PixelArray,
    reference: Option<&PixelArray>,
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) {
    for j in 0..height {
        let pixel = reference.unwrap_or(array).get_pixel_zero(x - 1, y + j);
        for i in 0..width {
            array.set_pixel(x + i, y + j, pixel);
        }
    }
}

fn decode_block2_animated(
    array: &mut PixelArray,
    reference: Option<&PixelArray>,
    header: &QmageHeader,
    bitstream: &mut BitReader,
    byte_stream: &mut dyn Iterator<Item = &u8>,
    x: usize,
    y: usize,
    deltas: &[u16],
    mode: Option<usize>,
) -> Result<(), QmageDecodeError> {
    let mode = mode.unwrap_or_else(|| bitstream.read_bits8(2) as usize);
    if mode > 3 {
        return Err(QmageDecodeError::FieldOutOfRange("mode"));
    }
    if header.qp == 0 || (bitstream.read_bits8(1) > 0) {
        if mode < 3 {
            for j in 0..4 {
                for i in 0..4 {
                    decode_pixel_in_block(
                        array,
                        None,
                        bitstream,
                        None,
                        byte_stream,
                        (x + i) as usize,
                        (y + j) as usize,
                        QMAGE_DIRS[mode].0,
                        QMAGE_DIRS[mode].1,
                        None,
                        &deltas,
                    );
                }
            }
        } else {
            if x > 0 {
                copy_edge(
                    array,
                    reference,
                    x,
                    y,
                    4.min(header.width as usize - x),
                    4.min(header.height as usize - y),
                );
            }
        }
    } else {
        return Err(QmageDecodeError::UnknownError);
    }

    Ok(())
}

fn decode_block3_animated(
    array: &mut PixelArray,
    reference: Option<&PixelArray>,
    header: &QmageHeader,
    bitstream: &mut BitReader,
    byte_stream: &mut dyn Iterator<Item = &u8>,
    x: usize,
    y: usize,
    mv_x: isize,
    mv_y: isize,
    deltas: &[u16],
) -> Result<(), QmageDecodeError> {
    let mode = bitstream.read_bits8(3);
    if mode <= 3 {
        return decode_block2_animated(
            array,
            None,
            header,
            bitstream,
            byte_stream,
            x,
            y,
            deltas,
            Some(mode as usize),
        );
    }
    match mode {
        4 => {
            for j in 0..4 {
                for i in 0..4 {
                    decode_pixel_in_block(
                        array,
                        reference,
                        bitstream,
                        None,
                        byte_stream,
                        (x + i) as usize,
                        (y + j) as usize,
                        0,
                        0,
                        None,
                        &deltas,
                    );
                }
            }
        }
        5 => {
            if reference.is_some() {
                copy_block(array, reference, x, y, x, y, 4, 4);
            }
        }
        6 => {
            for j in 0..4 {
                for i in 0..4 {
                    decode_pixel_in_block(
                        array,
                        reference,
                        bitstream,
                        None,
                        byte_stream,
                        x + i,
                        y + j,
                        mv_x,
                        mv_y,
                        None,
                        &deltas,
                    );
                }
            }
        }
        7 => {
            if (x as isize) + mv_x < 0
                || (x as isize) + mv_x > (header.width as isize)
                || (y as isize) + mv_y < 0
                || (y as isize) + mv_y > (header.height as isize)
            {
                return Err(QmageDecodeError::OffscreenMove);
            }

            if reference.is_some() {
                copy_block(
                    array,
                    reference,
                    ((x as isize) + mv_x) as usize,
                    ((y as isize) + mv_y) as usize,
                    x,
                    y,
                    4,
                    4,
                );
            }
        }
        0..=3 => unreachable!(),
        8.. => unreachable!(),
    }

    Ok(())
}

fn decode_mbedge_animated(
    array: &mut PixelArray,
    header: &QmageHeader,
    bitstream: &mut BitReader,
    byte_stream: &mut dyn Iterator<Item = &u8>,
    x_position: usize,
    y_position: usize,
    deltas: &[u16],
) -> Result<(), QmageDecodeError> {
    let width = header.width as usize;
    let height = header.height as usize;
    if array.pixel_type() != QmagePixelType::RGB565 {
        return Err(QmageDecodeError::UnsupportedPixelType(array.pixel_type()));
    }
    if bitstream.read_bits8(1) > 0 {
        return Err(QmageDecodeError::UnknownError);
    }
    for y in (y_position..(y_position + 16).min(height)).step_by(4) {
        for x in (x_position..(x_position + 16).min(width)).step_by(4) {
            if x + 4 <= width && y + 4 <= height {
                decode_block2_animated(
                    array,
                    None,
                    header,
                    bitstream,
                    byte_stream,
                    x,
                    y,
                    deltas,
                    None,
                )?;
            } else {
                for j in 0..4 {
                    for i in 0..4 {
                        if x + i < width && y + j < height {
                            let byte_a = match byte_stream.next() {
                                Some(byte) => (*byte) as u16,
                                None => return Err(QmageDecodeError::RanOutOfBytes),
                            };
                            let byte_b = match byte_stream.next() {
                                Some(byte) => (*byte) as u16,
                                None => return Err(QmageDecodeError::RanOutOfBytes),
                            };
                            let pixel = (byte_a << 8) | byte_b;
                            array
                                .set_pixel(x + i, y + j, Pixel::RGB565(pixel))
                                .unwrap_or_else(|| unreachable!());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn decode_mb_animated(
    array: &mut PixelArray,
    reference: Option<&PixelArray>,
    header: &QmageHeader,
    bitstream: &mut BitReader,
    byte_stream: &mut dyn Iterator<Item = &u8>,
    x: usize,
    y: usize,
    deltas: &[u16],
) -> Result<(), QmageDecodeError> {
    if bitstream.read_bits8(1) > 0 {
        if bitstream.read_bits8(1) > 0 {
            if reference.is_some() {
                copy_block(array, reference, x, y, x, y, 16, 16);
            }
        } else {
            let mv_x: isize;
            let mv_y: isize;
            if bitstream.read_bits8(1) > 0 {
                mv_x = 0;
                mv_y = 0;
            } else {
                mv_x = bitstream.read_bits8(8) as isize - 0x7f;
                mv_y = bitstream.read_bits8(7) as isize - 0x3f;

                if (x as isize) + mv_x < 0
                    || (x as isize) + mv_x > (header.width as isize)
                    || (y as isize) + mv_y < 0
                    || (y as isize) + mv_y > (header.height as isize)
                {
                    return Err(QmageDecodeError::OffscreenMove);
                }

                if bitstream.read_bits8(1) > 0 {
                    copy_block(
                        array,
                        reference,
                        ((x as isize) + mv_x) as usize,
                        ((y as isize) + mv_y) as usize,
                        x,
                        y,
                        16,
                        16,
                    );
                    return Ok(());
                }
            }

            for j in (0..16).step_by(4) {
                for i in (0..16).step_by(4) {
                    decode_block3_animated(
                        array,
                        reference,
                        header,
                        bitstream,
                        byte_stream,
                        x + i,
                        y + j,
                        mv_x,
                        mv_y,
                        deltas,
                    )?;
                }
            }
        }
    } else {
        for j in (0..16).step_by(4) {
            for i in (0..16).step_by(4) {
                decode_block2_animated(
                    array,
                    None,
                    header,
                    bitstream,
                    byte_stream,
                    x + i,
                    y + j,
                    deltas,
                    None,
                )?;
            }
        }
    }

    Ok(())
}

pub fn decode_v16shortindex_animated(
    array: &mut PixelArray,
    reference: &PixelArray,
    header: &QmageHeader,
    buffer: &[u8],
) -> Result<(), QmageDecodeError> {
    let header_size = header.header_size;
    let size = match header.get_record_size(false) {
        Some(s) => s,
        None => return Err(QmageDecodeError::NoMultiframeData),
    };
    let bitstream_start = header_size + 8;
    let mut bitstream = BitReader::new(&buffer[bitstream_start..size]);
    let byte_stream_start = u32::from_le_bytes([
        buffer[header_size],
        buffer[header_size + 1],
        buffer[header_size + 2],
        buffer[header_size + 3],
    ]) as usize;
    let mut byte_stream = buffer[byte_stream_start..size].iter();
    let width = header.width;
    let height = header.height;
    let ori_delta = QMAGE_ORIGINAL_DELTA[(header.version != QMAGE_MAGIC_VERSION) as usize];
    for y in (0..height).step_by(16) {
        for x in (0..width).step_by(16) {
            if x <= (width - 16) && y <= (height - 16) {
                decode_mb_animated(
                    array,
                    Some(reference),
                    header,
                    &mut bitstream,
                    &mut byte_stream,
                    x as usize,
                    y as usize,
                    &ori_delta,
                )?;
            } else {
                decode_mbedge_animated(
                    array,
                    header,
                    &mut bitstream,
                    &mut byte_stream,
                    x as usize,
                    y as usize,
                    &ori_delta,
                )?;
            }
        }
    }
    Ok(())
}

pub fn decode_v16shortindex_first_frame(
    array: &mut PixelArray,
    header: &QmageHeader,
    buffer: &[u8],
) -> Result<(), QmageDecodeError> {
    if header.codec != QmageCodec::V16ShortIndex {
        return Err(QmageDecodeError::IncorrectCodec(
            QmageCodec::V16ShortIndex,
            header.codec,
        ));
    }
    let header_size = header.header_size;
    let size = header.get_record_size(true).unwrap_or(buffer.len());
    let bitstream1_start = header_size + 8;
    let mut bitstream1 = BitReader::new(&buffer[bitstream1_start..size]);

    let bitstream2_start = u32::from_le_bytes([
        buffer[header_size],
        buffer[header_size + 1],
        buffer[header_size + 2],
        buffer[header_size + 3],
    ]) as usize;
    let mut bitstream2 = BitReader::new(&buffer[bitstream2_start..size]);

    let byte_pointer = u32::from_le_bytes([
        buffer[header_size + 4],
        buffer[header_size + 5],
        buffer[header_size + 6],
        buffer[header_size + 7],
    ]) as usize;
    let mut byte_stream = (&buffer[byte_pointer..size]).iter();

    let mut ori_delta_local = [0u16; 512];

    let ori_delta = if header.custom_lookup_table {
        let mut signs = [0u8; 512];
        let mut a;
        let mut b;
        let mut c;

        for i in 0..512 {
            signs[i] = *byte_stream.next().unwrap_or(&0);
        }

        for i in 0..512 {
            a = *byte_stream.next().unwrap_or(&0);
            b = *byte_stream.next().unwrap_or(&0);
            c = ((b as u16) << 8) | (a as u16);
            ori_delta_local[i] = if signs[i] > 0 {
                c
            } else {
                0u16.overflowing_sub(c).0
            };
        }

        &ori_delta_local[1..512]
    } else {
        QMAGE_ORIGINAL_DELTA[(header.version != QMAGE_MAGIC_VERSION) as usize].as_slice()
    };

    if header.image_type != QmagePixelType::RGB565 {
        return Err(QmageDecodeError::NotImplemented("non-RGB565 images"));
    }

    for y in (0..header.height).step_by(4) {
        for x in (0..header.width).step_by(4) {
            let mode = bitstream1.read_bits8(2) as usize;
            if mode < 3 {
                let byte_a = byte_stream.next().unwrap();
                let byte_b = byte_stream.next().unwrap();
                let cbp = ((*byte_b as u16) << 8) | (*byte_a as u16);
                let mut k = 0;

                for j in 0..4 {
                    for i in 0..4 {
                        if (x + i) < header.width && (y + j) < header.height {
                            decode_pixel_in_block(
                                array,
                                None,
                                &mut bitstream1,
                                Some(&mut bitstream2),
                                &mut byte_stream,
                                (x + i) as usize,
                                (y + j) as usize,
                                QMAGE_DIRS[mode].0,
                                QMAGE_DIRS[mode].1,
                                Some((cbp & (1 << k)) > 0),
                                &ori_delta,
                            );
                            k += 1
                        }
                    }
                }
            } else if x > 0 {
                copy_edge(array, None, x as usize, y as usize, 4, 4);
            }
        }
    }

    Ok(())
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Error)]
pub enum QmageDecodeError {
    #[error("No multiframe data provided when it's required")]
    NoMultiframeData,

    #[error("Expected {0:?} codec but got {1:?}")]
    IncorrectCodec(QmageCodec, QmageCodec),

    #[error("Ran out of bytes while reading")]
    RanOutOfBytes,

    #[error("Unsupported pixel type {0:?}")]
    UnsupportedPixelType(QmagePixelType),

    #[error("Unknown error")]
    UnknownError,

    #[error("Field {0} out of rande")]
    FieldOutOfRange(&'static str),

    #[error("Offscreen move instruction in image")]
    OffscreenMove,

    #[error("Support for {0} not implemented")]
    NotImplemented(&'static str),
}
