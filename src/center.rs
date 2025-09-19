use crossterm::terminal;

#[derive(Debug, Clone, Copy, Default)]
pub struct CenteringOffsets {
    pub top: u16,
    pub left: u16,
}

pub fn calculate_centering_offsets(input_lines: &[String]) -> Result<CenteringOffsets, Box<dyn std::error::Error>> {
    let (terminal_width, terminal_height) = terminal::size()?;
    
    if input_lines.is_empty() {
        return Ok(CenteringOffsets::default());
    }
    
    // Calculate content dimensions without ANSI codes
    let content_height = input_lines.len() as u16;
    let max_width = input_lines
        .iter()
        .map(|line| strip_ansi_codes(line).chars().count())
        .max()
        .unwrap_or(0) as u16;
    
    // Calculate centering offsets
    let top = if terminal_height > content_height {
        (terminal_height - content_height) / 2
    } else {
        0
    };
    
    let left = if terminal_width > max_width {
        (terminal_width - max_width) / 2
    } else {
        0
    };
    
    Ok(CenteringOffsets { top, left })
}

pub fn strip_ansi_codes(input: &str) -> String {
    let mut result = String::new();
    let mut in_escape = false;
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            in_escape = true;
            chars.next(); // consume '['
            continue;
        }

        if in_escape {
            if ch.is_ascii_alphabetic() {
                in_escape = false;
            }
            continue;
        }

        result.push(ch);
    }

    result
}