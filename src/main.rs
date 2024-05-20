use clap::Parser;
use core::fmt;
use std::fs::File;
use std::hash::Hash;
use std::io::{self, Read, Write};

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

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
enum MarkdownState {
    BOLD,
    ITALIC,
    BoldAndItalic,
    InlineCode,
    CodeBlock,
    LINK,
    TEXT,
}

struct MDParserState {
    text: String,
    html: String,
    length: usize,
    index: usize,
    state: MarkdownState,
    list_level: usize,
}

impl MDParserState {
    fn md_init_parser(mut md_file: File) -> Self {
        let mut text = String::new();
        let html = String::new();
        md_file.read_to_string(&mut text).expect("Read Error");
        text = text
            .lines()
            .map(str::trim)
            .collect::<Vec<&str>>()
            .join("\n");
        let length = text.len();
        MDParserState {
            text,
            html,
            length,
            index: 0,
            state: MarkdownState::TEXT,
            list_level: 0,
        }
    }

    fn handle_header(&mut self) {
        let mut level = 0;
        let char = self.get_ith_char(self.index).unwrap();
        while self.index < self.length && char == '#' {
            level += 1;
            self.index += 1;
        }
        if char == ' ' {
            self.index += 1;
        }
        let header_text = self.parse_inline();
        self.html
            .push_str(&format!("<h{}>{}</h{}>\n", level, header_text, level));
    }

    fn handle_bold_italic(&mut self) {
        if &self.text[self.index..=self.index + 1] == "***" {
            self.state = MarkdownState::BoldAndItalic;
            self.index += 3;
            let bold_and_italicized_text = self.parse_inline();
            self.html.push_str(&format!(
                "<em><strong>{}</strong></em>\n",
                bold_and_italicized_text
            ));
        }
        if &self.text[self.index..=self.index + 1] == "**" {
            self.state = MarkdownState::BOLD;
            self.index += 2;
            let bold_text = self.parse_inline();
            self.html
                .push_str(&format!("<strong>{}</strong>\n", bold_text));
        } else if self.get_ith_char(self.index).unwrap() == '*' {
            self.state = MarkdownState::ITALIC;
            self.index += 1;
            let italic_text = self.parse_inline();
            self.html.push_str(&format!("<em>{}</em>\n", italic_text))
        }
        self.state = MarkdownState::TEXT;
    }

    fn handle_link(&mut self) {
        self.state = MarkdownState::LINK;
        self.index += 1;
        let mut link_text = "".to_string();
        while self.index < self.length && self.get_ith_char(self.index).unwrap() != ']' {
            link_text.push(self.get_ith_char(self.index).unwrap());
            self.index += 1;
        }
        self.index += 2; // Skip ']('
        let mut link_url = "".to_string();
        while self.index < self.length && self.get_ith_char(self.index).unwrap() != ')' {
            link_url.push(self.get_ith_char(self.index).unwrap());
            self.index += 1;
        }
        self.html
            .push_str(&format!("<a href={}>{}</a>\n", link_url, link_text));
        self.state = MarkdownState::TEXT;
    }

    fn handle_code(&mut self) {
        if &self.text[self.index..=self.index + 2] == "```" {
            self.state = MarkdownState::CodeBlock;
            self.index += 3;
            let mut code_block = "".to_string();
            while self.index < self.length && &self.text[self.index..=self.index + 2] != "```" {
                code_block.push(self.get_ith_char(self.index).unwrap());
                self.index += 1;
            }
            self.index += 3;
            self.html
                .push_str(&format!("<pre><code>{}</code></pre>\n", code_block));
        } else if self.get_ith_char(self.index).unwrap() == '`' {
            self.state = MarkdownState::InlineCode;
            self.index += 1;
            let mut code_text = "".to_string();
            while self.index < self.length && self.get_ith_char(self.index).unwrap() != '`' {
                code_text.push(self.get_ith_char(self.index).unwrap());
                self.index += 1;
            }
            self.index += 1;
            self.html.push_str(&format!("<code>{}</code>\n", code_text));
        }
        self.state = MarkdownState::TEXT;
    }

    fn handle_ordered_list(&mut self) {
        if self.list_level == 0 {
            self.html.push_str("<ol>\n");
        }
        self.list_level += 1;
        self.index += 1;
        let ordered_list_item = self.parse_inline();
        self.html
            .push_str(&format!("    <li>{}</li>\n", ordered_list_item.trim()));
        self.list_level -= 1;
        if self.list_level == 0 {
            self.html.push_str("</ol>\n");
        }
    }

    fn handle_list(&mut self) {
        if self.list_level == 0 {
            self.html.push_str("<ul>\n");
        }
        self.list_level += 1;
        self.index += 1;
        let list_item = self.parse_inline();
        self.html
            .push_str(&format!("    <li>{}</li>\n", list_item.trim()));
        self.list_level -= 1;
        if self.list_level == 0 {
            self.html.push_str("</ul>\n");
        }
    }

    fn parse_inline(&mut self) -> String {
        let mut inline_html = "".to_string();
        println!("{}", self.index);
        println!("{}", self.length);
        let mut char = self.get_ith_char(self.index).unwrap();
        while self.index < self.length && "\n#".contains(char) {
            char = self.get_ith_char(self.index).unwrap();
            match char {
                '*' => {
                    if &self.text[self.index..=self.index + 1] == "***" {
                        self.index += 3;
                        let bold_and_italicized_text = self.parse_inline();
                        inline_html.push_str(&format!(
                            "<em><strong>{}</strong></em>",
                            bold_and_italicized_text
                        ));
                    }
                    if &self.text[self.index..=self.index + 1] == "**" {
                        self.index += 2;
                        let bold_text = self.parse_inline();
                        inline_html.push_str(&format!("<strong>{}</strong>", bold_text));
                    } else {
                        self.index += 1;
                        let italic_text = self.parse_inline();
                        inline_html.push_str(&format!("<em>{}</em>", italic_text));
                    }
                }
                '[' => {
                    self.index += 1;
                    let mut link_text = "".to_string();
                    while self.index < self.length && self.get_ith_char(self.index).unwrap() != ']'
                    {
                        link_text.push(self.get_ith_char(self.index).unwrap());
                        self.index += 1;
                    }
                    self.index += 2; // Skip ']('
                    let mut link_url = "".to_string();
                    while self.index < self.length && self.get_ith_char(self.index).unwrap() != ')'
                    {
                        link_url.push(self.get_ith_char(self.index).unwrap());
                        self.index += 1;
                    }
                    inline_html.push_str(&format!("<a href={}>{}</a>\n", link_url, link_text));
                }
                '`' => {
                    self.index += 1;
                    let mut code_text = "".to_string();
                    while self.index < self.length && self.get_ith_char(self.index).unwrap() != '`'
                    {
                        code_text.push(self.get_ith_char(self.index).unwrap());
                        self.index += 1;
                    }
                    self.index += 1;
                    inline_html.push_str(&format!("<code>{}</code>", code_text));
                }
                _ => inline_html.push(char),
            }
            self.index += 1;
        }
        return inline_html;
    }

    fn get_ith_char(&self, index: usize) -> Option<char> {
        self.text.chars().nth(index)
    }
}

fn md_to_html(md_path: &str) -> Result<String, &'static str> {
    use MDParserState;
    use MarkdownState::*;
    let md_file = File::open(md_path).expect("IO Error");
    let mut parser = MDParserState::md_init_parser(md_file);
    println!("{}", &parser.text);

    while parser.index < parser.length {
        let i = parser.index;
        let char: char = parser.get_ith_char(i).unwrap();
        println!("{}", char);
        if parser.state == TEXT {
            match char {
                '#' => parser.handle_header(),
                '*' => {
                    parser.handle_bold_italic();
                    parser.index -= 1;
                }
                '[' => parser.handle_link(),
                '`' => parser.handle_code(),
                '\n' => parser.html.push_str("<br>\n"),
                '-' if parser.get_ith_char(i + 1).unwrap() == '.' => {
                    parser.handle_list();
                }
                _ if char.is_digit(10) && parser.get_ith_char(i + 1).unwrap() == '.' => {
                    parser.handle_ordered_list();
                }

                _ => parser.html.push(char),
            }
        }
        parser.index += 1;
    }

    println!("{}", parser.html);
    Ok(parser.html)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn validate_html() -> io::Result<()> {
        let mut real_file = File::open("test_files/baby.html")?;
        let mut real_contents = String::new();
        real_file.read_to_string(&mut real_contents)?;
        real_contents = real_contents.to_string();

        let mut test_file = File::open("test_files/test.html")?;
        let mut test_contents = String::new();
        test_file.read_to_string(&mut test_contents)?;
        test_contents = test_contents.trim().to_string();
        assert_eq!(real_contents, test_contents);
        Ok(())
    }
}
