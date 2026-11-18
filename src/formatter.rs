pub struct Formatter;

impl Formatter {
    /// Formats a complete Oxide source code string into standard canonical format.
    pub fn format(source: &str) -> String {
        let lines: Vec<&str> = source.lines().collect();
        let mut formatted_lines: Vec<String> = Vec::new();
        let mut indent_level: usize = 0;
        let mut prev_was_empty = false;

        for raw_line in lines {
            let trimmed = raw_line.trim();

            if trimmed.is_empty() {
                if !prev_was_empty && !formatted_lines.is_empty() {
                    formatted_lines.push(String::new());
                    prev_was_empty = true;
                }
                continue;
            }
            prev_was_empty = false;

            // Count leading closing braces on this line to adjust indentation before printing
            let mut leading_closes = 0;
            let mut temp = trimmed;
            while temp.starts_with('}') || temp.starts_with(']') || temp.starts_with(')') {
                if temp.starts_with('}') {
                    leading_closes += 1;
                    temp = temp[1..].trim_start();
                    if temp.starts_with(';') {
                        temp = temp[1..].trim_start();
                    }
                } else {
                    break;
                }
            }

            if leading_closes > 0 && indent_level >= leading_closes {
                indent_level -= leading_closes;
            }

            // Format line contents (spacing around operators, colons, commas)
            let formatted_content = Self::format_line_content(trimmed);

            // Indent line
            let indent_spaces = "    ".repeat(indent_level);
            formatted_lines.push(format!("{}{}", indent_spaces, formatted_content));

            // Calculate indentation change for subsequent lines
            // We count net braces outside of strings and comments
            let (opens, closes) = Self::count_braces(trimmed);
            let remaining_closes = closes.saturating_sub(leading_closes);
            if remaining_closes > 0 && indent_level >= remaining_closes {
                indent_level -= remaining_closes;
            }
            indent_level += opens;
        }

        let mut result = formatted_lines.join("\n");
        if !result.is_empty() {
            result.push('\n');
        }
        result
    }

    /// Checks if source is already formatted.
    pub fn check(source: &str) -> bool {
        let formatted = Self::format(source);
        formatted == source
    }

    /// Counts opening and closing curly braces outside of string literals and comments.
    fn count_braces(line: &str) -> (usize, usize) {
        let mut opens = 0;
        let mut closes = 0;
        let mut in_str = false;
        let mut escaped = false;
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            if in_str {
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_str = false;
                }
                i += 1;
                continue;
            }

            // Check comments
            if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
                break; // rest of line is comment
            }

            if c == '"' {
                in_str = true;
            } else if c == '{' {
                opens += 1;
            } else if c == '}' {
                closes += 1;
            }

            i += 1;
        }

        (opens, closes)
    }

    #[allow(dead_code)]
    fn net_brace_delta(line: &str) -> isize {
        let (opens, closes) = Self::count_braces(line);
        opens as isize - closes as isize
    }

    /// Formats spacing inside a single line, preserving strings and comments intact.
    fn format_line_content(line: &str) -> String {
        // If the whole line is a comment or doc comment, keep it as is
        if line.starts_with("//") {
            return line.to_string();
        }

        let chars: Vec<char> = line.chars().collect();
        let mut out = String::new();
        let mut i = 0;
        let mut in_str = false;
        let mut escaped = false;

        while i < chars.len() {
            let c = chars[i];

            if in_str {
                out.push(c);
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == '"' {
                    in_str = false;
                }
                i += 1;
                continue;
            }

            // Detect start of comment
            if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
                // Ensure a space before comment if not at the start
                if !out.is_empty() && !out.ends_with(' ') {
                    out.push(' ');
                }
                let rest: String = chars[i..].iter().collect();
                out.push_str(&rest);
                break;
            }

            if c == '"' {
                in_str = true;
                out.push(c);
                i += 1;
                continue;
            }

            // Check 2-character tokens: '->', '=>', '==', '!=', '<=', '>=', '&&', '||', '::', '..'
            if i + 1 < chars.len() {
                let next = chars[i + 1];

                if (c == '-' || c == '=') && next == '>' {
                    // '->' or '=>'
                    Self::push_spaced(&mut out, &format!("{}{}", c, next));
                    i += 2;
                    continue;
                }

                if (c == '=' && next == '=')
                    || (c == '!' && next == '=')
                    || (c == '<' && next == '=')
                    || (c == '>' && next == '=')
                    || (c == '&' && next == '&')
                    || (c == '|' && next == '|')
                {
                    Self::push_spaced(&mut out, &format!("{}{}", c, next));
                    i += 2;
                    continue;
                }

                if c == ':' && next == ':' {
                    // '::' has no spaces around it (e.g. Option::Some)
                    Self::trim_trailing_space(&mut out);
                    out.push_str("::");
                    i += 2;
                    continue;
                }

                if c == '.' && next == '.' {
                    // '..' range has no spaces around it (e.g. 0..10)
                    Self::trim_trailing_space(&mut out);
                    out.push_str("..");
                    i += 2;
                    continue;
                }
            }

            // Single characters
            match c {
                ',' => {
                    Self::trim_trailing_space(&mut out);
                    out.push(',');
                    out.push(' ');
                    i += 1;
                }
                ';' => {
                    Self::trim_trailing_space(&mut out);
                    out.push(';');
                    i += 1;
                }
                ':' => {
                    // In type annotations or map pairs: 'x: int'
                    Self::trim_trailing_space(&mut out);
                    out.push(':');
                    out.push(' ');
                    i += 1;
                }
                '=' | '+' | '*' | '/' | '%' => {
                    // Math & Assignment operators
                    Self::push_spaced(&mut out, &c.to_string());
                    i += 1;
                }
                '-' => {
                    // Could be unary minus or subtraction
                    // If preceding character is an operator or open paren, keep it tight (e.g. -5, (-x))
                    let prev_is_op = out.trim_end().ends_with(|ch: char| {
                        ch == '(' || ch == '[' || ch == '=' || ch == '+' || ch == '*' || ch == '/' || ch == ','
                    });
                    if prev_is_op || out.trim().is_empty() {
                        out.push('-');
                    } else {
                        Self::push_spaced(&mut out, "-");
                    }
                    i += 1;
                }
                '<' | '>' => {
                    // Could be comparison (a < b) or generic (Box<int>)
                    // If preceded by identifier without space, likely generic: e.g. "Box<" or "Option<"
                    let last_word_is_type = out.ends_with(|ch: char| ch.is_alphanumeric() || ch == '_');
                    if last_word_is_type && c == '<' {
                        out.push('<');
                    } else if c == '>' && (out.contains('<') && !out.ends_with(' ')) {
                        out.push('>');
                    } else {
                        Self::push_spaced(&mut out, &c.to_string());
                    }
                    i += 1;
                }
                '{' => {
                    if !out.is_empty() && !out.ends_with(' ') {
                        out.push(' ');
                    }
                    out.push('{');
                    i += 1;
                }
                '}' => {
                    Self::trim_trailing_space(&mut out);
                    out.push('}');
                    i += 1;
                    if i < chars.len()
                        && !chars[i].is_whitespace()
                        && chars[i] != ';'
                        && chars[i] != ','
                        && chars[i] != ')'
                        && chars[i] != ']'
                    {
                        out.push(' ');
                    }
                    continue;
                }
                '(' | '[' => {
                    out.push(c);
                    i += 1;
                }
                ')' | ']' => {
                    Self::trim_trailing_space(&mut out);
                    out.push(c);
                    i += 1;
                }
                '.' => {
                    Self::trim_trailing_space(&mut out);
                    out.push('.');
                    i += 1;
                }
                ' ' | '\t' => {
                    // Normalize multiple consecutive spaces outside strings
                    if !out.ends_with(' ') && !out.ends_with('(') && !out.ends_with('[') {
                        out.push(' ');
                    }
                    i += 1;
                }
                _ => {
                    out.push(c);
                    i += 1;
                }
            }
        }

        // Clean up redundant spaces around parentheses: "( " -> "(", " )" -> ")"
        let mut cleaned = out.replace("( ", "(").replace(" )", ")").replace("[ ", "[").replace(" ]", "]");
        // Clean double spaces if any
        while cleaned.contains("  ") {
            cleaned = cleaned.replace("  ", " ");
        }

        cleaned.trim_end().to_string()
    }

    fn push_spaced(out: &mut String, token: &str) {
        Self::trim_trailing_space(out);
        if !out.is_empty() && !out.ends_with('(') && !out.ends_with('[') {
            out.push(' ');
        }
        out.push_str(token);
        out.push(' ');
    }

    fn trim_trailing_space(out: &mut String) {
        while out.ends_with(' ') {
            out.pop();
        }
    }
}
