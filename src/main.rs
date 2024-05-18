use clap::Parser;
use core::{fmt, num};
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
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

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
enum MarkdownToken {
    Bold,
    Italic,
    BoldItalic,
    InlineCode,
    CodeBlock,
    Header1,
    Header2,
    Header3,
    UnorderedList,
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
    map.insert("*".to_string(), Italic);
    map.insert("`".to_string(), InlineCode);
    map.insert("```".to_string(), CodeBlock);
    map.insert("-".to_string(), UnorderedList);
    map
}

fn md_tokens_to_html_openers() -> HashMap<MarkdownToken, String> {
    use MarkdownToken::*;
    let mut map = HashMap::new();
    map.insert(Header1, "<h1>".to_string());
    map.insert(Header2, "<h2>".to_string());
    map.insert(Header3, "<h3>".to_string());
    map.insert(BoldItalic, "<bold><em>".to_string());
    map.insert(Bold, "<bold>".to_string());
    map.insert(Italic, "<em>".to_string());
    map.insert(InlineCode, "<code>".to_string());
    map.insert(CodeBlock, "<code>".to_string());
    map.insert(UnorderedList, "<ul>".to_string());
    map
}
fn md_tokens_to_html_closers() -> HashMap<MarkdownToken, String> {
    use MarkdownToken::*;
    let mut map = HashMap::new();
    map.insert(Header1, "</h1>".to_string());
    map.insert(Header2, "</h2>".to_string());
    map.insert(Header3, "</h3>".to_string());
    map.insert(BoldItalic, "</bold></em>".to_string());
    map.insert(Bold, "</bold>".to_string());
    map.insert(Italic, "</em>".to_string());
    map.insert(InlineCode, "</code>".to_string());
    map.insert(CodeBlock, "</code>".to_string());
    map.insert(UnorderedList, "</ul>".to_string());
    map
}

fn md_openers_to_closers() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("#".to_string(), "\n".to_string());
    map.insert("##".to_string(), "\n".to_string());
    map.insert("###".to_string(), "\n".to_string());
    map.insert("***".to_string(), "***".to_string());
    map.insert("**".to_string(), "**".to_string());
    map.insert("*".to_string(), "*".to_string());
    map.insert("```".to_string(), "```".to_string());
    map.insert("`".to_string(), "`".to_string());
    map.insert("-".to_string(), "\n".to_string());

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

fn tokenize_md(md_contents: &str) -> Vec<MarkdownToken> {
    use MarkdownToken::*;
    println!("{}", md_contents);
    let len = md_contents.len();
    let tags_to_tokens = md_tags_to_tokens();
    let openers_to_closers = md_openers_to_closers();

    let mut tokens: Vec<MarkdownToken> = Vec::new();
    let mut i = 0;

    let valid_tags: Vec<String> = vec![
        "***".to_string(),
        "**".to_string(),
        "*".to_string(),
        "###".to_string(),
        "##".to_string(),
        "#".to_string(),
        "```".to_string(),
        "`".to_string(),
        "-".to_string(),
    ];
    while i < len {
        let mut is_tag = false;
        for opener_tag in &valid_tags {
            if !md_contents[i..].starts_with(opener_tag) {
                continue;
            }
            is_tag = true;
            let token = match tags_to_tokens.get(opener_tag) {
                Some(t) => t.clone(),
                None => panic!(),
            };
            let closer_tag: String = match openers_to_closers.get(opener_tag) {
                Some(tag) => tag.clone(),
                None => panic!(),
            };
            let closer_len = closer_tag.len();
            let token_len = opener_tag.len();
            tokens.push(token.clone());
            if let Some(end) = md_contents[i + token_len..].find(&closer_tag) {
                let adjusted_end = i + token_len + end;
                tokens.push(Text(md_contents[i + token_len..adjusted_end].to_string()));
                tokens.push(token);
                i = adjusted_end + closer_len;
            } else {
                panic!();
            }
            break;
        }
        if !is_tag {
            tokens.push(Text(md_contents[i..i + 1].to_string()));
            i += 1;
        }
    }

    tokens
}

fn token_to_html(token: &MarkdownToken, map: &HashMap<MarkdownToken, String>) -> String {
    match token {
        MarkdownToken::Text(s) => s.replace('\n', "<br>\n"),
        _ => map
            .get(token)
            .cloned()
            .unwrap_or_else(|| panic!("No matching HTML opener for token")),
    }
}

fn md_to_html(md_path: &str) -> Result<String, &'static str> {
    use MarkdownToken::*;
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
    let openers = md_tokens_to_html_openers();
    let closers = md_tokens_to_html_closers();
    println!("{:#?}", tokens);
    let mut in_tag = false;
    // Will have to use a stack to keep track of most recent tag if I allow nested tag in future
    for token in tokens {
        let mut html: String;
        if !in_tag {
            match token {
                Text(_) => (),
                _ => in_tag = true,
            }
            html = token_to_html(&token, &openers);
        } else {
            html = token_to_html(&token, &closers);
            match token {
                Text(_) => (),
                _ => {
                    in_tag = false;
                    html.push('\n');
                }
            }
        }
        converted_string.push_str(&html);
    }
    println!("{}", converted_string);

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
