pub mod pager;

pub use pager::Pager;

use indicatif::{ProgressBar, ProgressStyle};
use terminal_size::{Height, Width, terminal_size};

pub fn term_width() -> usize {
    terminal_size().map_or(100, |(Width(w), _)| w as usize)
}

pub fn term_height() -> usize {
    terminal_size().map_or(24, |(_, Height(h))| h as usize)
}

/// How many list items fit in the terminal, accounting for line-wrap.
/// `chrome_lines` = lines consumed by prompt/header/footer outside the item list.
/// Each item that is wider than the terminal wraps and occupies multiple visual rows.
pub fn max_visible_items(items: &[String], chrome_lines: usize) -> usize {
    let width = term_width();
    let available = term_height().saturating_sub(chrome_lines);
    let mut visual_rows = 0;
    let mut count = 0;

    for item in items {
        // FuzzySelect prepends a selector char + space (~4 cols of chrome per item)
        let item_width = console::measure_text_width(item) + 4;
        let rows = if width > 0 {
            item_width.div_ceil(width).max(1)
        } else {
            1
        };
        if visual_rows + rows > available {
            break;
        }
        visual_rows += rows;
        count += 1;
    }

    count.max(1)
}

pub fn truncate(s: &str, max: usize) -> String {
    let char_count: usize = s.chars().count();
    if char_count <= max {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max.saturating_sub(1)).collect();
        format!("{truncated}…")
    }
}

pub fn spinner(msg: &str) -> ProgressBar {
    if crate::is_agent() {
        return ProgressBar::hidden();
    }
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .expect("valid template"),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(80));
    pb
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_ascii_within_limit() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_ascii_at_limit() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn truncate_ascii_over_limit() {
        assert_eq!(truncate("hello world", 5), "hell…");
    }

    #[test]
    fn truncate_multibyte_no_panic() {
        assert_eq!(truncate("Andrés Valero", 5), "Andr…");
    }

    #[test]
    fn truncate_multibyte_within_limit() {
        assert_eq!(truncate("Andrés", 10), "Andrés");
    }

    #[test]
    fn truncate_multibyte_at_boundary() {
        assert_eq!(truncate("Andrés", 6), "Andrés");
    }

    #[test]
    fn truncate_multibyte_just_over() {
        assert_eq!(truncate("Andrés", 5), "Andr…");
    }

    #[test]
    fn truncate_emoji() {
        assert_eq!(truncate("🇳🇴 Norway", 3), "🇳🇴…");
    }

    #[test]
    fn truncate_empty() {
        assert_eq!(truncate("", 5), "");
    }

    #[test]
    fn truncate_zero_max() {
        assert_eq!(truncate("hello", 0), "…");
    }

    #[test]
    fn truncate_max_one() {
        assert_eq!(truncate("hello", 1), "…");
    }

    #[test]
    fn truncate_norwegian_chars() {
        assert_eq!(truncate("Friheten i Koden: Ære", 15), "Friheten i Kod…");
    }
}
