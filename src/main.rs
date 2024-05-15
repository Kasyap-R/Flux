use core::fmt;

use clap::Parser;

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

#[derive(Parser, Debug)]
#[command(version = "0.1", about = "A simple backup tool" , long_about = None)]
struct Args {
    /// The file to read from
    file_one: String,

    /// The file to write to
    file_two: String,
}

fn main() {
    let args = Args::parse();
    let file_type_one = FileType::from_file_name(&args.file_one);
    let file_type_two = FileType::from_file_name(&args.file_two);
    match file_type_one {
        Ok(x) => println!("File one type: {}", x),
        Err(s) => println!("Error: {}", s),
    }
    match file_type_two {
        Ok(x) => println!("File two type: {}", x),
        Err(s) => println!("Error: {}", s),
    }
}
