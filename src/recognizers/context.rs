//! 上下文判断：正向上下文词看命中前的窗口；负向上下文词只看紧挨在命中前的那个词。

/// 正向上下文窗口：在命中位置之前最多看这么多个字符。
const WINDOW_CHARS: usize = 32;

/// 负向上下文：命中前允许跳过的分隔符。
const SEPARATORS: &[char] = &[' ', '\t', '=', ':', '：', '#', '_', '-', '"', '\''];

/// 命中前 `WINDOW_CHARS` 个字符内是否出现任一词（`words` 已小写）。
pub(super) fn window_has(text: &str, start: usize, words: &[String]) -> bool {
    if words.is_empty() {
        return false;
    }
    let prefix = &text[..start];
    let from = prefix
        .char_indices()
        .rev()
        .nth(WINDOW_CHARS - 1)
        .map_or(0, |(i, _)| i);
    let window = prefix[from..].to_lowercase();
    words.iter().any(|w| window.contains(w.as_str()))
}

/// 去掉分隔符后，命中前的文本是否以某个完整的词结尾（`words` 已小写）。
/// "完整"指词前面不是 ASCII 字母，所以 `server 10.0.0.1` 不会被 `ver` 命中。
pub(super) fn preceded_by(text: &str, start: usize, words: &[String]) -> bool {
    if words.is_empty() {
        return false;
    }
    let prefix = text[..start].trim_end_matches(SEPARATORS);
    let from = prefix.char_indices().rev().nth(15).map_or(0, |(i, _)| i);
    let tail = prefix[from..].to_lowercase();
    words.iter().any(|w| {
        tail.strip_suffix(w.as_str())
            .is_some_and(|before| !before.ends_with(|c: char| c.is_ascii_alphabetic()))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(w: &[&str]) -> Vec<String> {
        w.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn preceded_by_requires_whole_word() {
        let w = words(&["ver", "版本"]);
        assert!(preceded_by("ver=1.2.3.4", 4, &w));
        assert!(preceded_by("版本 1.2.3.4", 7, &w));
        assert!(!preceded_by("server 1.2.3.4", 7, &w));
        assert!(!preceded_by("ver is 1.2.3.4", 7, &w));
    }

    #[test]
    fn window_has_is_case_insensitive_substring() {
        assert!(window_has("My Card No: 4111", 12, &words(&["card"])));
        assert!(!window_has("nothing here 4111", 13, &words(&["card"])));
    }
}
