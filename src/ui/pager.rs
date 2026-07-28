use console::{Key, Term};

/// Action returned by the pager's key handler.
pub enum Action {
    /// Continue rendering the current page (scroll changed).
    Redraw,
    /// A custom key was pressed that the caller should handle.
    Custom(Key),
}

/// Terminal-aware scrollable pager.
///
/// Renders pre-built content inside a viewport that accounts for
/// line-wrapping, with header/footer chrome reserved automatically.
pub struct Pager {
    lines: Vec<String>,
    vis_rows: Vec<usize>,
    term: Term,
    viewport: usize,
    max_scroll: usize,
    scroll: usize,
    scrollable: bool,
}

impl Pager {
    /// Build a pager for `content`. `footer_hint` is the navigation text
    /// shown at the bottom — its wrapped height is measured to compute
    /// the available viewport.
    pub fn new(content: &str, footer_hint: &str) -> Self {
        let term = Term::stderr();
        let (term_rows, term_cols) = term.size();
        let term_w = term_cols as usize;

        let lines: Vec<String> = content.lines().map(String::from).collect();

        let vis_rows: Vec<usize> = lines.iter().map(|l| vis_row_count(l, term_w)).collect();

        let hint_rows = vis_row_count(footer_hint, term_w);
        // chrome = header(1) + blank before hints(1) + hint rows + cursor(1)
        let chrome = 1 + 1 + hint_rows + 1;
        let viewport = (term_rows as usize).saturating_sub(chrome);

        let max_scroll = calc_max_scroll(&vis_rows, viewport);
        let scrollable = max_scroll > 0;

        Self {
            lines,
            vis_rows,
            term,
            viewport,
            max_scroll,
            scroll: 0,
            scrollable,
        }
    }

    /// Whether the content overflows the viewport.
    pub fn is_scrollable(&self) -> bool {
        self.scrollable
    }

    /// Clear the screen and render the current viewport with header and footer.
    pub fn render(&self, header: &str, footer: &str) -> std::io::Result<()> {
        self.term.clear_screen()?;

        println!("{header}");

        let mut rows_used = 0;
        let mut i = self.scroll;
        while i < self.lines.len() {
            if rows_used + self.vis_rows[i] > self.viewport {
                break;
            }
            println!("{}", self.lines[i]);
            rows_used += self.vis_rows[i];
            i += 1;
        }

        println!("\n{footer}");
        Ok(())
    }

    /// Current scroll offset (logical line index).
    pub fn scroll_offset(&self) -> usize {
        self.scroll
    }

    /// Total number of logical lines.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Wait for a keypress and handle scroll keys. Returns `Action::Redraw`
    /// for scroll events, or `Action::Custom(key)` for everything else.
    pub fn handle_key(&mut self) -> std::io::Result<Action> {
        let key = self.term.read_key()?;
        match key {
            Key::ArrowUp | Key::Char('k') => {
                self.scroll = self.scroll.saturating_sub(1);
                Ok(Action::Redraw)
            }
            Key::ArrowDown | Key::Char('j') => {
                if self.scroll < self.max_scroll {
                    self.scroll += 1;
                }
                Ok(Action::Redraw)
            }
            Key::Char('\x15') | Key::PageUp => {
                let half = self.viewport / 2;
                let mut rows = 0;
                while self.scroll > 0 && rows < half {
                    self.scroll -= 1;
                    rows += self.vis_rows[self.scroll];
                }
                Ok(Action::Redraw)
            }
            Key::Char('\x04') | Key::PageDown => {
                let half = self.viewport / 2;
                let mut rows = 0;
                while self.scroll < self.max_scroll && rows < half {
                    rows += self.vis_rows[self.scroll];
                    self.scroll += 1;
                }
                Ok(Action::Redraw)
            }
            other => Ok(Action::Custom(other)),
        }
    }

    /// Clear the screen (useful before returning to the caller).
    pub fn clear(&self) -> std::io::Result<()> {
        self.term.clear_screen()
    }
}

fn vis_row_count(text: &str, term_w: usize) -> usize {
    let w = console::measure_text_width(text);
    if w == 0 || term_w == 0 {
        1
    } else {
        w.div_ceil(term_w)
    }
}

fn calc_max_scroll(vis_rows: &[usize], viewport: usize) -> usize {
    let mut max_scroll = vis_rows.len();
    let mut tail = 0;
    while max_scroll > 0 {
        let needed = vis_rows[max_scroll - 1];
        if tail + needed > viewport {
            break;
        }
        tail += needed;
        max_scroll -= 1;
    }
    max_scroll
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vis_row_count_empty() {
        assert_eq!(vis_row_count("", 80), 1);
    }

    #[test]
    fn vis_row_count_fits() {
        assert_eq!(vis_row_count("hello", 80), 1);
    }

    #[test]
    fn vis_row_count_wraps() {
        let long = "a".repeat(200);
        assert_eq!(vis_row_count(&long, 80), 3);
    }

    #[test]
    fn vis_row_count_zero_width_term() {
        assert_eq!(vis_row_count("hello", 0), 1);
    }

    #[test]
    fn calc_max_scroll_all_fits() {
        let vis = vec![1, 1, 1];
        assert_eq!(calc_max_scroll(&vis, 10), 0);
    }

    #[test]
    fn calc_max_scroll_needs_scroll() {
        let vis = vec![1, 1, 1, 1, 1];
        assert_eq!(calc_max_scroll(&vis, 3), 2);
    }

    #[test]
    fn calc_max_scroll_wrapping_lines() {
        let vis = vec![3, 2, 1]; // 6 visual rows total
        assert_eq!(calc_max_scroll(&vis, 4), 1); // can show lines 1+2 (3 rows) but not 0+1+2 (6)
    }
}
