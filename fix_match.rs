                        KeyCode::Char('K') | KeyCode::PageUp | KeyCode::Char('[') => {
                            thread_scroll = thread_scroll.saturating_sub(5);
                        }
                        KeyCode::Char('J') | KeyCode::PageDown | KeyCode::Char(']') => {
                            thread_scroll = thread_scroll.saturating_add(5);
                        }
