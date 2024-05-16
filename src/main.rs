use clap::Parser;
use core::fmt;
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
    Header1,
    Header2,
    Header3,
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
    map.insert("**".to_string(), Bold);
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

fn tokenize_md(md_contents: &str) -> Vec<MarkdownToken> {
    use MarkdownToken::*;
    println!("{}", md_contents);
    let mut token_list: Vec<MarkdownToken> = Vec::new();
    let tags_to_tokens = md_tags_to_tokens();
    let mut curr_sequence: String = "".to_string();
    let chars: Vec<char> = md_contents.chars().collect();
    // For each char
    //  add it to the curr_sequence string
    //  if curr_sequence combined with the next char is a valid token, continue
    //  else if curr_sequence is a token, add that token to token list,
    //  else if curr_char is a token, add curr_sequence (minus last) as a string and set curr_sequence to just
    //  this char
    for (index, c) in chars.iter().enumerate() {
        curr_sequence.push(*c);
        // check if adding the next token creates a token
        // NOTE: doesn't really work with non-uniform tokens of length > 3 if the first two chars
        // make a token and the doesn't but the fourth does. Don't know if this will ever be a
        // problem
        let mut sequence_plus_next = curr_sequence.clone();

        if index < chars.len() - 1 {
            sequence_plus_next.push(chars[index + 1]);
            if let Some(_x) = tags_to_tokens.get(&sequence_plus_next) {
                continue;
            }
        }
        if let Some(x) = tags_to_tokens.get(&curr_sequence) {
            token_list.push(x.clone());
            curr_sequence.clear();
        } else if let Some(x) = tags_to_tokens.get(&c.to_string()) {
            // If this is a single token that's made it this far, tokenize
            if curr_sequence.len() == 1 {
                token_list.push(x.clone());
                curr_sequence.clear();
                continue;
            }
            // Otherwise Add all previous chars excluding the current as a text token
            let text = curr_sequence[..curr_sequence.len() - 1].to_string();
            token_list.push(Text(text));

            // if this character isn't part of a multi char sequence, tokenize it as well
            if index >= chars.len() - 1 {
                continue;
            }
            let next_sequence_len = sequence_plus_next.len();
            match tags_to_tokens
                .get(&sequence_plus_next[next_sequence_len - 2..=next_sequence_len - 1])
            {
                Some(_x) => {
                    curr_sequence.clear();
                    curr_sequence.push(*c);
                }
                None => {
                    curr_sequence.clear();
                    token_list.push(x.clone());
                }
            }
        }
    }

    // Add remaining chars as text
    if !curr_sequence.is_empty() {
        token_list.push(Text(curr_sequence));
    }

    token_list
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
