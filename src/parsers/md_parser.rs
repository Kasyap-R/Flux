use std::fs::File;
use std::io::Read;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
enum MarkdownState {
    BOLD,
    ITALIC,
    BoldAndItalic,
    InlineCode,
    CodeBlock,
    HEADER,
    LINK,
    LIST,
    TEXT,
    PARAGRAPH,
    STRIKETHROUGH,
    QUOTE,
}

struct MDParser {
    text: String,
    html: String,
    length: usize,
    index: usize,
    list_level: usize,
    states: Vec<MarkdownState>,
}

impl MDParser {
    fn md_init_parser(mut md_file: File) -> Self {
        let mut text = String::new();
        let html = String::new();
        md_file.read_to_string(&mut text).expect("Read Error");
        text = MDParser::preprocess_md(text);
        let length = text.len();
        MDParser {
            text,
            html,
            length,
            index: 0,
            list_level: 0,
            states: vec![MarkdownState::TEXT],
        }
    }

    fn preprocess_md(mut md_contents: String) -> String {
        let lines = md_contents.lines();
        let mut new_lines: Vec<&str> = Vec::new();
        let mut in_code_block = false;
        for mut line in lines {
            if line.starts_with("```") {
                in_code_block = !in_code_block;
            }
            if !in_code_block {
                line = line.trim_start();
            }
            new_lines.push(line);
        }
        md_contents = new_lines.into_iter().collect::<Vec<&str>>().join("\n");
        md_contents
    }

    fn handle_header(&mut self) {
        let mut level = 0;
        let mut char = self.get_ith_char(self.index).unwrap();
        while self.check_next_chars(self.index, "#") {
            level += 1;
            self.index += 1;
            char = self.get_ith_char(self.index).unwrap();
        }
        // Skip whitespace following hashtags if present
        if char == ' ' {
            self.index += 1;
        }
        self.push_state(MarkdownState::HEADER);
        self.html.push_str(&format!("<h{}>", level));
        self.parse_inline();
        self.html.push_str(&format!("</h{}>", level));
        self.pop_state();
        self.optionally_push_newline();
    }

    fn handle_italic(&mut self, state: MarkdownState) {
        self.index += 1;
        match state {
            MarkdownState::ITALIC => {
                self.pop_state();
            }
            _ => {
                self.push_state(MarkdownState::ITALIC);
                self.html.push_str("<em>");
                self.parse_inline();
                self.html.push_str("</em>");
                self.optionally_push_newline();
            }
        }
    }

    fn handle_bold(&mut self, state: MarkdownState) {
        self.index += 2;
        match state {
            MarkdownState::BOLD => {
                self.pop_state();
            }
            _ => {
                self.push_state(MarkdownState::BOLD);
                self.html.push_str("<strong>");
                self.parse_inline();
                self.html.push_str("</strong>");
                self.optionally_push_newline();
            }
        }
    }

    fn handle_bold_italic(&mut self, state: MarkdownState) {
        self.index += 3;
        match state {
            MarkdownState::BoldAndItalic => {
                self.pop_state();
            }
            _ => {
                self.push_state(MarkdownState::BoldAndItalic);
                self.html.push_str("<em><strong>");
                self.parse_inline();
                self.html.push_str("</em></strong>");
                self.optionally_push_newline();
            }
        }
    }

    fn handle_asterisks_inline(&mut self) {
        let state = self.get_current_state();
        match state {
            MarkdownState::BoldAndItalic if self.check_next_chars(self.index, "***") => {
                self.handle_bold_italic(state);
            }
            MarkdownState::BOLD if self.check_next_chars(self.index, "**") => {
                self.handle_bold(state);
            }
            MarkdownState::ITALIC if self.check_next_chars(self.index, "*") => {
                self.handle_italic(state);
            }
            _ => self.handle_asterisks(),
        }
    }

    fn handle_asterisks(&mut self) {
        let state = self.get_current_state();

        if self.check_next_chars(self.index, "***") {
            self.handle_bold_italic(state);
        } else if self.check_next_chars(self.index, "**") {
            self.handle_bold(state);
        } else if self.check_next_chars(self.index, "*") {
            self.handle_italic(state);
        }
    }

    fn handle_link(&mut self) {
        self.push_state(MarkdownState::LINK);
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
        self.index += 1;
        self.html
            .push_str(&format!("<a href={}>{}</a>", link_url, link_text));
        self.pop_state();
        self.optionally_push_newline();
    }

    fn handle_code(&mut self) {
        if self.check_next_chars(self.index, "```") {
            self.index += 3;
            let state = self.get_current_state();
            match state {
                MarkdownState::TEXT => {
                    self.push_state(MarkdownState::CodeBlock);
                    let mut code_block = "".to_string();
                    while self.index < self.length
                        && &self.text[self.index..=self.index + 2] != "```"
                    {
                        code_block.push(self.get_ith_char(self.index).unwrap());
                        self.index += 1;
                    }
                    self.index += 3;
                    self.html
                        .push_str(&format!("<pre><code>{}</code></pre>", code_block));
                    self.pop_state();
                    self.optionally_push_newline();
                }
                _ => {
                    self.html.push_str("```");
                }
            }
        } else if self.check_next_chars(self.index, "`") {
            self.push_state(MarkdownState::InlineCode);
            self.index += 1;
            let mut code_text = "".to_string();
            while !self.check_next_chars(self.index, "`") {
                code_text.push(self.get_ith_char(self.index).unwrap());
                self.index += 1;
            }
            self.index += 1;
            self.html.push_str(&format!("<code>{}</code>", code_text));
            self.pop_state();
            self.optionally_push_newline();
        }
    }

    // NOTE: Currently don't support nested lists
    fn handle_ordered_list(&mut self) {
        self.push_state(MarkdownState::LIST);
        self.html.push_str("    <li>");
        self.parse_inline();
        self.html.push_str("</li>");
        self.pop_state();
        self.optionally_push_newline();

        if self.index + 1 < self.length
            && self.get_ith_char(self.index).unwrap().is_digit(10)
            && self.get_ith_char(self.index + 1).unwrap() == '.'
        {
            self.index += 2;
            self.handle_ordered_list();
        }
    }

    fn handle_list(&mut self) {
        self.push_state(MarkdownState::LIST);
        self.html.push_str("    <li>");
        self.parse_inline();
        self.html.push_str("</li>");
        self.pop_state();
        self.optionally_push_newline();
        if self.index < self.length && self.get_ith_char(self.index).unwrap() == '-' {
            self.index += 1;
            self.handle_list();
        }
    }

    fn handle_paragraph(&mut self) {
        self.push_state(MarkdownState::PARAGRAPH);
        self.html.push_str("<p>");
        while self.index < self.length {
            // Use parse inline to parse till the end of the line and then do a check if there is a
            // tag immediately following a newline
            self.parse_inline();
            let char = match self.get_ith_char(self.index) {
                Some(x) => x,
                None => break,
            };
            if "#>-".contains(char)
                || self.check_next_chars(self.index, "```")
                || (char.is_digit(10) && self.get_ith_char(self.index + 1).unwrap() == '.')
            {
                break;
            }
        }
        self.html.push_str("</p>\n");
        self.pop_state();
    }

    fn handle_quotes(&mut self) {
        self.push_state(MarkdownState::QUOTE);
        self.html.push_str("<quoteblock>\n");
        while self.check_next_chars(self.index, ">") {
            self.index += 1;
            self.parse_inline();
            self.html.push('\n');
        }
        self.html.push_str("</quoteblock>\n");
        self.pop_state();
    }

    fn handle_strikethrough(&mut self) {
        if !self.check_next_chars(self.index, "~~") {
            return;
        }
        self.index += 2;
        let state = self.get_current_state();
        match state {
            MarkdownState::STRIKETHROUGH => {
                self.pop_state();
            }
            _ => {
                self.push_state(MarkdownState::STRIKETHROUGH);
                self.html.push_str("<s>");
                self.parse_inline();
                self.html.push_str("</s>");
                self.optionally_push_newline();
            }
        }
    }

    fn parse_inline(&mut self) {
        // We keep track of the length of the stack, if it changes, meaning we have fulfilled the
        // purpose of this inline, we break;
        let stack_size: usize = self.states.len();
        while self.index < self.length {
            let new_stack_size = self.states.len();
            if new_stack_size != stack_size {
                break;
            }
            let char = self.get_ith_char(self.index).unwrap();
            if char == '\n' {
                self.html.push(' ');
                self.index += 1;
                break;
            }

            match char {
                '*' => {
                    self.handle_asterisks_inline();
                }
                '[' => {
                    self.handle_link();
                }
                '`' => {
                    self.handle_code();
                }
                '~' => {
                    self.handle_strikethrough();
                }
                ' ' if self.get_current_state() == MarkdownState::PARAGRAPH
                    && self.check_next_chars(self.index, "  \n") =>
                {
                    self.html.push_str("<br>\n");
                    self.index += 3;
                    break;
                }
                _ => {
                    self.html.push(char);
                    self.index += 1;
                }
            }
        }
    }

    fn get_ith_char(&self, index: usize) -> Option<char> {
        if index >= self.length {
            return None;
        }
        self.text.chars().nth(index)
    }

    fn push_state(&mut self, state: MarkdownState) {
        self.states.push(state);
    }

    fn pop_state(&mut self) {
        if self.states.len() <= 1 {
            panic!("Removing the bottom TEXT state");
        }
        self.states.pop();
    }

    fn get_current_state(&self) -> MarkdownState {
        self.states.last().unwrap().clone()
    }

    fn optionally_push_newline(&mut self) {
        let index = self.html.len() - 1;
        let mut chars_since_newline = 0;
        for c in self.html[..index].chars().rev() {
            if c == '\n' {
                break;
            }
            chars_since_newline += 1;
        }

        if self.get_current_state() == MarkdownState::TEXT || chars_since_newline >= 80 {
            self.html.push('\n');
        }
    }

    fn check_next_chars(&self, index: usize, substring: &str) -> bool {
        let substring_length = substring.len();
        if index + substring_length <= self.text.len() {
            return &self.text[index..=index + substring_length - 1] == substring;
        }
        false
    }
}

pub fn md_to_html(md_path: &str) -> Result<String, &'static str> {
    use MDParser;
    use MarkdownState::*;
    let md_file = File::open(md_path).expect("IO Error");
    let mut parser = MDParser::md_init_parser(md_file);
    println!("====================================\nMarkdown Contents:\n====================================\n {}\n=====================================", &parser.text);

    while parser.index < parser.length {
        let i = parser.index;
        let char: char = parser.get_ith_char(i).unwrap();
        if parser.get_current_state() == TEXT {
            match char {
                '#' => parser.handle_header(),
                '*' => {
                    parser.handle_asterisks();
                }
                '[' => parser.handle_link(),
                '`' => parser.handle_code(),
                '~' => parser.handle_strikethrough(),
                '-' => {
                    parser.html.push_str("<ul>\n");
                    parser.index += 1;
                    parser.list_level += 1;
                    parser.handle_list();
                    parser.html.push_str("</ul>\n");
                    parser.list_level -= 1;
                    parser.index += 1;
                }
                '>' => parser.handle_quotes(),
                '\n' => parser.index += 1,
                _ if char.is_digit(10) && parser.get_ith_char(i + 1).unwrap() == '.' => {
                    parser.html.push_str("<ol>\n");
                    parser.index += 2;
                    parser.handle_ordered_list();
                    parser.html.push_str("</ol>\n");
                }

                _ => {
                    parser.handle_paragraph();
                }
            }
        }
    }

    println!("HTML Contents:\n====================================\n {}\n=====================================", &parser.html);
    Ok(parser.html)
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
