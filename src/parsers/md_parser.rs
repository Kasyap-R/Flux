use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::Read;

use clap::Parser;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
enum MarkdownState {
    BOLD,
    ITALIC,
    BoldAndItalic,
    InlineCode,
    CodeBlock,
    HEADER,
    LINK,
    OrderedList,
    UnorderedList,
    TEXT,
    PARAGRAPH,
    STRIKETHROUGH,
    QUOTE,
}

enum ListType {
    Ordered,
    Unordered,
    NOTHING,
}

struct MDParser {
    text: String,
    html: String,
    length: usize,
    index: usize,
    list_level: Option<usize>,
    max_list_level: usize,
    curr_list_type: ListType,
    indent_to_list_level: BTreeMap<usize, usize>,
    indentation_level: usize,
    states: Vec<MarkdownState>,
}

impl MDParser {
    fn md_init_parser(mut md_file: File) -> Self {
        let mut text = String::new();
        let html = String::new();
        md_file.read_to_string(&mut text).expect("Read Error");
        text = MDParser::preprocess_md(text);
        let length = text.len();
        let mut list_map = BTreeMap::new();
        list_map.insert(0, 1);
        MDParser {
            text,
            html,
            length,
            index: 0,
            list_level: Some(1),
            max_list_level: 1,
            curr_list_type: ListType::NOTHING,
            indent_to_list_level: list_map,
            indentation_level: 0,
            states: vec![MarkdownState::TEXT],
        }
    }

    fn preprocess_md(mut md_contents: String) -> String {
        let lines = md_contents.lines();
        let mut new_lines: Vec<&str> = Vec::new();
        let mut in_code_block = false;
        for line in lines {
            if line.trim().is_empty() {
                continue;
            }
            if line.starts_with("```") {
                in_code_block = !in_code_block;
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

    fn get_list_level_from_indent(&mut self, indent_level: usize) -> (usize, usize) {
        match self.indent_to_list_level.get(&indent_level) {
            Some(&x) => (indent_level, x.clone()),
            None => {
                // If there exists a higher index level we've already dealt with, then
                // we try to find the closest indent level that is less than
                // the current indent level
                // if nothing is less than it, then we add a new level

                let mut higher_indent = self.indent_to_list_level.range(indent_level + 1..);
                if higher_indent.next().is_some() {
                    let closest_key = self.indent_to_list_level.range(..indent_level).next_back();
                    match closest_key {
                        Some((&k, &v)) => (k.clone(), v.clone()),
                        None => {
                            panic!("Curr indent is not highest, but nothing lower exists");
                        }
                    }
                } else {
                    self.indent_to_list_level
                        .insert(indent_level, self.max_list_level);
                    (indent_level, self.max_list_level)
                }
            }
        }
    }

    // Each of the bottom two handle functions will record what list level they were associated
    // with. If our list level is valid (i.e. the next line contains a list) and equivalent to theirs, they know to keep going. Every
    // time our indentation level is lower than it was previously, we will decrease the size of the
    // callstack until we get to a function that was called to handle that level
    fn handle_list_items(&mut self) {
        println!("List Level: {}", self.list_level.unwrap());
        println!("Curr_indent: {}", self.indentation_level);
        let spaces = " ".repeat(self.list_level.unwrap() * 4);
        self.html.push_str(&format!("{}<li>", spaces));
        self.parse_inline();
        self.html.push_str("</li>\n");
        self.list_level = None;
        let dist_to_char = self.find_distance_to_non_whitespace(self.index);
        let indent_difference: i32 = dist_to_char as i32 - self.indentation_level as i32;
        if indent_difference > 5 {
            return;
        }
        // If the curr indent level exists in the map and there is a swap in list type, then return
        // But that doesn't mean we stop parsing at this indent level
        let curr_char = match self.get_ith_char(self.index + dist_to_char) {
            Some(x) => x,
            None => ' ',
        };
        // Find the appropriate list level based off of the indentation level
        let (curr_indent, curr_list_level) = self.get_list_level_from_indent(dist_to_char);
        let current_state: MarkdownState = self.get_current_state();
        if curr_char == '-' {
            self.list_level = Some(curr_list_level);
            self.index += dist_to_char;
            self.indentation_level = curr_indent;
            if indent_difference < 0 {
                return;
            }
            if current_state == MarkdownState::UnorderedList && indent_difference == 0 {
                self.index += 1;
                self.handle_list_items();
            } else {
                self.handle_unordered_list();
            }
        } else if curr_char.is_digit(10)
            && self.check_next_chars(self.index + dist_to_char + 1, ".")
        {
            self.list_level = Some(curr_list_level);
            self.index += dist_to_char;
            self.indentation_level = curr_indent;
            if indent_difference < 0 || indent_difference > 5 {
                return;
            }
            if current_state == MarkdownState::OrderedList && indent_difference == 0 {
                self.index += 2;
                self.handle_list_items();
            } else {
                self.handle_ordered_list();
            }
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

    fn find_distance_to_non_whitespace(&mut self, mut start_index: usize) -> usize {
        let mut indentation: usize = 0;
        while let Some(x) = self.get_ith_char(start_index) {
            match x {
                ' ' => indentation += 1,
                _ => return indentation,
            }
            start_index += 1;
        }
        0
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

    fn handle_ordered_list(&mut self) {
        self.push_state(MarkdownState::OrderedList);
        let spaces = " ".repeat((self.list_level.unwrap() - 1) * 4);
        self.html.push_str(&format!("{}<ol>\n", spaces));
        self.index += 2;
        self.handle_list_items();
        self.html.push_str(&format!("{}</ol>\n", spaces));
        self.pop_state();
    }

    fn handle_unordered_list(&mut self) {
        self.push_state(MarkdownState::UnorderedList);
        let list_level_snapshot = self.list_level.unwrap();
        let indent_level_snapshot = self.indentation_level;
        let spaces = " ".repeat((list_level_snapshot - 1) * 4);
        self.html.push_str(&format!("{}<ul>\n", spaces));
        while let Some(x) = self.list_level {
            if x == list_level_snapshot {
                self.index += 1;
                self.max_list_level += 1;
                self.handle_list_items();
                self.max_list_level -= 1;
            } else {
                println!(
                    "Removing list level: {}",
                    self.indent_to_list_level
                        .get(&indent_level_snapshot)
                        .unwrap()
                );
                self.indent_to_list_level.remove(&indent_level_snapshot);
                break;
            }
        }
        self.html.push_str(&format!("{}</ul>\n", spaces));
        self.pop_state();
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
                '>' => parser.handle_quotes(),
                '\n' => parser.index += 1,
                '-' => {
                    parser.handle_unordered_list();
                    parser.list_level = Some(1);
                    parser.max_list_level = 1;
                    // Clear the BTreeMap of everyting but the 0,0 pair
                    parser.indent_to_list_level.retain(|&k, _| k == 0);
                    parser.indentation_level = 0;
                }
                _ if char.is_digit(10) && parser.get_ith_char(i + 1).unwrap() == '.' => {
                    parser.handle_ordered_list();
                    parser.list_level = Some(1);
                    parser.max_list_level = 1;
                    parser.indent_to_list_level.retain(|&k, _| k == 0);
                    parser.indentation_level = 0;
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
