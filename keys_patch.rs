                        KeyCode::Char('u') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            thread_scroll = thread_scroll.saturating_sub(10);
                        }
                        KeyCode::Char('d') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            thread_scroll = thread_scroll.saturating_add(10);
                        }
                        KeyCode::Char('b') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            thread_scroll = thread_scroll.saturating_sub(20);
                        }
                        KeyCode::Char('f') if key.modifiers.contains(event::KeyModifiers::CONTROL) => {
                            thread_scroll = thread_scroll.saturating_add(20);
                        }
                        KeyCode::Char('K') => {
                            thread_scroll = thread_scroll.saturating_sub(5);
                        }
                        KeyCode::Char('J') => {
                            thread_scroll = thread_scroll.saturating_add(5);
                        }
                        KeyCode::PageDown | KeyCode::Char(']') => {
                            thread_scroll = thread_scroll.saturating_add(5);
                        }
                        KeyCode::PageUp | KeyCode::Char('[') => {
                            thread_scroll = thread_scroll.saturating_sub(5);
                        }
