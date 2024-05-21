use crate::parsers::md_parser;
use clap::Parser;
use core::fmt;
use std::fs::File;
use std::io::{self, Write};

pub mod parsers;

#[derive(Parser, Debug)]
#[command(version = "0.1", about = "A tool to convert between file types" , long_about = None)]
struct Args {
    /// The file to read from
    #[arg(short, long, default_value = "test_files/baby.md")]
    input_file: String,

    /// The file to write to
    #[arg(short, long, default_value = "test_files/test.html")]
    output_file: String,
}

enum FileType {
    HTML,
    MD,
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let file_type_str = match self {
            FileType::HTML => "HTML",
            FileType::MD => "MD",
        };
        write!(f, "{}", file_type_str)
    }
}

impl FileType {
    fn from_file_name(file_name: &str) -> Result<FileType, &'static str> {
        match file_name.split_once('.').unwrap().1 {
            "html" => Ok(FileType::HTML),
            "md" => Ok(FileType::MD),
            _ => Err("Invalid File Type"),
        }
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let input_file_type: FileType =
        FileType::from_file_name(&args.input_file).expect("Unsupported File Type");
    let output_file_type: FileType =
        FileType::from_file_name(&args.output_file).expect("Unsupported File Type");

    let converted_data = match (input_file_type, output_file_type) {
        (FileType::MD, FileType::HTML) => md_parser::md_to_html(&args.input_file).expect("error"),
        (_, _) => "Unsupported Conversion".to_string(),
    };

    let mut output_file = File::create(&args.output_file)?;
    output_file.write_all(converted_data.as_bytes())?;
    output_file.flush()?;
    Ok(())
}
