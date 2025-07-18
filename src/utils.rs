use unicode_width::UnicodeWidthChar;

use url::{ParseError, Url};

/// Safely truncate a string, ensuring it is not truncated in the middle of multi-byte characters
///
/// This function will:
/// 1. Correctly handle Unicode characters (including Chinese, emoji, etc.)
/// 2. Add ellipsis when maximum length is reached
/// 3. Ensure the output string's display width does not exceed the specified length
#[allow(dead_code)]
pub fn truncate_str(s: &str, max_width: usize) -> String {
    use unicode_width::UnicodeWidthStr;

    if s.width() <= max_width {
        return s.to_string();
    }

    let mut result = String::new();
    let mut current_width = 0;

    for c in s.chars() {
        let char_width = c.width().unwrap_or(1);

        if current_width + char_width + 3 > max_width {
            break;
        }

        result.push(c);
        current_width += char_width;
    }

    result.push_str("...");
    result
}

pub fn pickup_host_from_url(url: &str) -> Result<String, ParseError> {
    let parsed_url = Url::parse(url)?;
    let scheme = parsed_url.scheme();
    let host = parsed_url.host_str().ok_or(url::ParseError::EmptyHost)?;

    let port = parsed_url
        .port()
        .map(|x| format!(":{x}"))
        .or_else(|| Some("".to_string()))
        .unwrap();

    Ok(format!("{scheme}://{host}{port}/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_str() {
        assert_eq!(truncate_str("Hello, world!", 10), "Hello, ...");
        assert_eq!(truncate_str("你好，世界！", 8), "你好...");
        assert_eq!(truncate_str("Hello 你好！", 10), "Hello ...");
        assert_eq!(truncate_str("Hi!", 10), "Hi!");
    }
}
