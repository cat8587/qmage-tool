mod licenses_and_credits;

use std::{
    fs::{File, OpenOptions},
    io::Read,
};

use licenses_and_credits::{print_credits, print_license_text, print_project_license};

use qmage::{decoder::decode_image, header::QmageHeader, header::QmagePixelType};

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
    let pixel_array = decode_image(read_header, &buffer);
    let pixel_array = pixel_array.convert_to_pixel_type(QmagePixelType::RGBA8888);
    let image = pixel_array.rgba8888_to_rgbaimage().unwrap();
    image
        .write_to(&mut output_file, image::ImageFormat::Png)
        .unwrap();
}
