mod licenses_and_credits;

use std::{
    fs::{File, OpenOptions},
    io::Read,
};

use licenses_and_credits::{print_credits, print_license_text, print_project_license};

use qmage::{
    decoder::{decode_v16shortindex_animated, decode_v16shortindex_first_frame},
    header::{QmageHeader, QmagePixelType},
    pixels::PixelArray,
};

use clap::{Parser, Subcommand};

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
    };
}

fn decode(qmage_file_path: String, output_file_path: String) {
    let mut qmage_file = File::open(qmage_file_path).unwrap();
    let mut output_file = OpenOptions::new()
        .write(true)
        .create(true)
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
    while offset < buffer.len() {
        slice = &buffer[offset..buffer.len()];
        frame_header = QmageHeader::read_header(slice).unwrap();
        multiframe_data = frame_header.multiframe_data.as_ref().unwrap();
        println!("Header {}: {}", frame_index, multiframe_data.current_frame,); // DEBUG
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
            .open(output_format.replace("%", format!("{}", frame_index).as_str()))
            .unwrap();
        image = pixel_array_converted.rgba8888_to_rgbaimage().unwrap();
        image
            .write_to(&mut output_file, image::ImageFormat::Png)
            .unwrap();
        offset += (multiframe_data.total_length
            * if frame_header.version == qmage::header::QMAGE_MAGIC_VERSION {
                1
            } else {
                if frame_index == 0 { 4 } else { 1 }
            }) as usize;
        frame_index += 1;
        println!("Offset: {}", offset); // DEBUG
    }
}
