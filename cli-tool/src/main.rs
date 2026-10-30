mod licenses_and_credits;

use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
};

use image::{DynamicImage, ImageReader};
use licenses_and_credits::{print_credits, print_license_text, print_project_license};

use qmage::{
    decoder::{decode_v16shortindex_animated, decode_v16shortindex_first_frame},
    encoder::{
        EncoderError, EncoderResult, encode_v16shortindex_first_frame,
        encode_v16shortindex_identical_frame, write_result_to_buffer,
    },
    header::{QmageHeader, QmagePixelType},
    pixels::PixelArray,
};

use clap::{CommandFactory, Parser, Subcommand, error::ErrorKind};

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    subcommand: Subcommands,
}

#[derive(Subcommand)]
enum Subcommands {
    Credits,
    License {
        spdx: Option<String>,
    },
    Decode {
        qmage_file: String,
        output_file: String,
    },
    DecodeMultiframe {
        qmage_file: String,
        output_format: String,
    },
    EncodeBootImage {
        input_file: String,
        qmage_file: String,
        #[arg(long, default_value_t = 10)]
        frame_duration: u16,
        #[arg(long, default_value_t = 10)]
        total_frames: u16,
        #[arg(long)]
        use_version_11: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.subcommand {
        Subcommands::Credits => print_credits(true),
        Subcommands::License { spdx: None } => print_project_license(true),
        Subcommands::License { spdx: Some(s) } => {
            print_license_text(s.as_str());
        }
        Subcommands::Decode {
            qmage_file: q,
            output_file: o,
        } => decode(q, o),
        Subcommands::DecodeMultiframe {
            qmage_file: q,
            output_format: o,
        } => decode_multiframe(q, o),
        Subcommands::EncodeBootImage {
            input_file: i,
            qmage_file: q,
            frame_duration: f,
            total_frames: t,
            use_version_11: v11,
        } => encode_bootimage(i, q, f, t, v11),
    };
}

fn decode(qmage_file_path: String, output_file_path: String) {
    let mut qmage_file = File::open(qmage_file_path).unwrap();
    let mut output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_file_path)
        .unwrap();
    let mut buffer = Vec::new();
    qmage_file.read_to_end(&mut buffer).unwrap();
    let read_header = QmageHeader::read_header(&buffer).unwrap();
    let mut pixel_array = PixelArray::new(
        read_header.image_type,
        read_header.width,
        read_header.height,
    );
    decode_v16shortindex_first_frame(&mut pixel_array, &read_header, &buffer).unwrap();
    let pixel_array = pixel_array.convert_to_pixel_type(QmagePixelType::RGBA8888);
    let image = pixel_array.rgba8888_to_rgbaimage().unwrap();
    image
        .write_to(&mut output_file, image::ImageFormat::Png)
        .unwrap();
}

fn decode_multiframe(qmage_file_path: String, output_format: String) {
    if !output_format.contains('%') {
        panic!("No % in file path format");
    }
    let mut qmage_file = File::open(qmage_file_path).unwrap();

    let mut buffer = Vec::new();
    qmage_file.read_to_end(&mut buffer).unwrap();
    let read_header = QmageHeader::read_header(&buffer).unwrap();
    read_header
        .multiframe_data
        .as_ref()
        .expect("Not a multi-frame qmage file");
    let mut pixel_array = PixelArray::new(
        read_header.image_type,
        read_header.width,
        read_header.height,
    );
    let mut frame_header;
    let mut offset = 0;
    let mut frame_index = 0;
    let mut slice;
    let mut output_file;
    let mut image;
    let mut pixel_array_converted;
    let mut previous_frame;
    let mut multiframe_data;
    while offset < buffer.len() - 12 {
        slice = &buffer[offset..buffer.len()];
        frame_header = QmageHeader::read_header(slice).unwrap();
        multiframe_data = frame_header.multiframe_data.as_ref().unwrap();
        if offset == 0 {
            decode_v16shortindex_first_frame(&mut pixel_array, &read_header, &buffer).unwrap();
        } else {
            previous_frame = pixel_array.clone();
            decode_v16shortindex_animated(&mut pixel_array, &previous_frame, &frame_header, slice)
                .unwrap();
        }
        pixel_array_converted = pixel_array.convert_to_pixel_type(QmagePixelType::RGBA8888);
        output_file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output_format.replace("%", format!("{}", frame_index).as_str()))
            .unwrap();
        image = pixel_array_converted.rgba8888_to_rgbaimage().unwrap();
        image
            .write_to(&mut output_file, image::ImageFormat::Png)
            .unwrap();
        offset += (multiframe_data.total_length) as usize
            * if frame_header.version == qmage::header::QMAGE_MAGIC_VERSION {
                1
            } else {
                if frame_index == 0 { 4 } else { 1 }
            };
        frame_index += 1;
    }
}

fn unwrap_encoder_result(
    frame_index: usize,
    result: Result<EncoderResult, EncoderError>,
) -> EncoderResult {
    match result {
        Ok(r) => r,
        Err(EncoderError::RecordTooBig(s, m)) => {
            let mut cmd = Cli::command();
            cmd.error(ErrorKind::ValueValidation, format!("Record size too large for frame {}, size: {} (greater than the maximum {})\nNote: try using --use-version-11, which supports larger frame records", frame_index, s, m)).exit();
        }
        _ => result.unwrap(),
    }
}

fn encode_bootimage(
    input_file: String,
    qmage_file: String,
    frame_duration: u16,
    total_frames: u16,
    use_version_11: bool,
) {
    let image = ImageReader::open(input_file)
        .unwrap()
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap();

    let image = image.to_rgba8();
    let pixel_array =
        PixelArray::rgbaimage_to_rgba8888(image).convert_to_pixel_type(QmagePixelType::RGB565);

    let result = unwrap_encoder_result(
        1,
        encode_v16shortindex_first_frame(
            &pixel_array,
            frame_duration,
            total_frames as usize,
            if use_version_11 { 11 } else { 15 },
        ),
    );

    let mut buffer = Vec::new();
    let header = result.header.clone();
    let (o, s) = write_result_to_buffer(result, &mut buffer).unwrap();
    println!("Frame 1: offset: {}, length: {}", o, s.unwrap());

    for i in 2..=total_frames {
        let result = unwrap_encoder_result(
            i as usize,
            encode_v16shortindex_identical_frame(&pixel_array, &header, i as usize),
        );
        let (o, s) = write_result_to_buffer(result, &mut buffer).unwrap();
        println!("Frame {}: offset: {}, length: {}", i, o, s.unwrap());
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(qmage_file)
        .unwrap();

    file.write_all(&buffer).unwrap();
}
