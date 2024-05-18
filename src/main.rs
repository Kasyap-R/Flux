use clap::Parser;
use core::{fmt, num};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Write};

#[derive(Parser, Debug)]
#[command(version = "0.1", about = "A simple backup tool" , long_about = None)]
struct Args {
    /// The file to read from
    input_file: String,

    /// The file to write to
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

#[derive(Clone, Debug)]
enum MarkdownToken {
    Bold,
    Italic,
    BoldItalic,
    InlineCode,
    CodeBlock,
    Header1,
    Header2,
    Header3,
    BreakLine,
    Text(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn validate_html() -> io::Result<()> {
        let mut real_file = File::open("test_files/baby.html")?;
        let mut real_contents = String::new();
        real_file.read_to_string(&mut real_contents)?;
        real_contents = real_contents.trim().to_string();

        let mut test_file = File::open("test_files/test.html")?;
        let mut test_contents = String::new();
        test_file.read_to_string(&mut test_contents)?;
        test_contents = test_contents.trim().to_string();
        assert_eq!(real_contents, test_contents);
        Ok(())
    }
}

fn md_tags_to_tokens() -> HashMap<String, MarkdownToken> {
    use MarkdownToken::*;
    let mut map = HashMap::new();
    map.insert("#".to_string(), Header1);
    map.insert("##".to_string(), Header2);
    map.insert("###".to_string(), Header3);
    map.insert("***".to_string(), BoldItalic);
    map.insert("**".to_string(), Bold);
    map.insert("`".to_string(), InlineCode);
    map.insert("```".to_string(), CodeBlock);
    map
}

fn closers_md_to_html_map() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("#".to_string(), "<h1>".to_string());
    map
}

fn md_openers_to_closers() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("#".to_string(), "<h1>".to_string());
    map
}

fn validate_md(md_contents: &str) -> bool {
    // for each char in md
    //      if opener:
    //          write openers equivalent to file
    //          if opener has a closer:
    //              push corresponding coser to stack
    //      else if closer:
    //          if closer is top element of stack
    //              pop
    //              write closers equivalent to file
    //          else:
    //              return an error
    //      else:
    //          write char to file as is

    false
}

// Goal: Create a Markdown Tokenizer given a string of valid markdown using a state machine
// What is a markdown token? A markdown tag such as "*" or "###" is a markdownt token
// a character is also a valid token, given it isn't a tog or part of a tag
// the state you're in changes depending on the tag most recently processed
// For example, if you processed "**", you are in a bold state until you find a corresponding "**"
// while in bold state you can still add raw characters, which is what you do until you find an end
// tag
// Not all markdown tags have a closing tag, but all HTML tags do, so for some states, a newline
// will denote the end of that state
// No nesting of tags so when we find a tag, we continuously add chars until we find its end

fn handle_header1(md_contents: &str, i: usize) -> i32 {
    return 3;
}

fn tokenize_md(md_contents: &str) -> Vec<MarkdownToken> {
    use MarkdownToken::*;
    println!("{}", md_contents);
    let len = md_contents.len();
    let tags_to_tokens = md_tags_to_tokens();

    let mut tokens: Vec<MarkdownToken> = Vec::new();
    let mut i = 0;

    let valid_tags: Vec<String> = vec!["***".to_string(), "###".to_string(), "```".to_string()];
    while i < len {
        for str in &valid_tags {
            if !md_contents[i..].starts_with(str) {
                continue;
            }
            let token_len = str.len();
            let token;
            if let Some(x) = md_tags_to_tokens().get(str) {
                token = x.clone();
                tokens.push(token);
                if let Some(end) = md_contents[i..token_len].find(str) {
                    let adjusted_end = i + token_len + end;
                    tokens.push(Text(md_contents[i + 3..adjusted_end].to_string()));
                    tokens.push(BoldItalic);
                    i = adjusted_end + token_len;
                }
            }
        }
        if md_contents[i..].starts_with("***") {
            tokens.push(BoldItalic);
            if let Some(end) = md_contents[i + 3..].find("***") {
                let adjusted_end = i + 3 + end;
                tokens.push(Text(md_contents[i + 3..adjusted_end].to_string()));
                tokens.push(BoldItalic);
                i = adjusted_end + 3;
            } else {
                panic!();
            }
        } else if md_contents[i..].starts_with("###") {
            tokens.push(Header3);
            if let Some(end) = md_contents[i + 3..].find("\n") {
                let adjusted_end = i + 3 + end;
                tokens.push(Text(md_contents[i + 3..adjusted_end].to_string()));
                tokens.push(Header3);
                i = adjusted_end + 1;
            } else {
                panic!();
            }
        } else if md_contents[i..].starts_with("```") {
            tokens.push(CodeBlock);
            if let Some(end) = md_contents[i + 3..].find("```") {
                let adjusted_end = i + 3 + end;
                tokens.push(Text(md_contents[i + 3..adjusted_end].to_string()));
                tokens.push(CodeBlock);
                i = adjusted_end + 3;
            } else {
                panic!();
            }
        } else {
            tokens.push(Text(md_contents[i..i + 1].to_string()));
            i += 1;
        }
    }

    tokens
}

// Gonna have to create my own Errors later, one of which can hold IO errors
// Other issue, just because a markdown element doesn't have closing tags, doesn't mean that the
// HTML element also doesn't
// I'm Thinking I write some function to tokenize the data and that is where I can do the stack
// logic to determine faultiness and then this function just calls that and at that point can
// easily write the neccesary components to the file
fn md_to_html(md_path: &str) -> Result<String, &'static str> {
    // Open fileone
    // 3 maps needed
    //      opener_md to opener_html
    //      closer_md to closer_html
    //      opener_md to closer_md
    let mut converted_string = String::new();
    let mut md_file = File::open(md_path).expect("IO Error");
    let mut md_file_contents = String::new();
    md_file
        .read_to_string(&mut md_file_contents)
        .expect("Read Error");
    md_file_contents = md_file_contents
        .lines()
        .map(str::trim)
        .collect::<Vec<&str>>()
        .join("\n");
    // This last new line is needed for tokenization
    md_file_contents.push('\n');
    let tokens = tokenize_md(&md_file_contents);
    println!("{:#?}", tokens);

    Ok(converted_string)
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let input_file_type: FileType =
        FileType::from_file_name(&args.input_file).expect("Unsupported File Type");
    let output_file_type: FileType =
        FileType::from_file_name(&args.output_file).expect("Unsupported File Type");

    let converted_data = match (input_file_type, output_file_type) {
        (FileType::MD, FileType::HTML) => md_to_html(&args.input_file).expect("error"),
        (_, _) => "Unsupported Conversion".to_string(),
    };

    /* let mut output_file = File::create(&args.file_two)?;
    output_file.write_all(converted_data.as_bytes())?;
    output_file.flush()?; */
    Ok(())
}
