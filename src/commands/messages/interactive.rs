use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap},
};
use tokio::sync::mpsc;

use super::args::InboxView;
use crate::client::TrpcClient;
use crate::types::{ConversationRow, GetConversationResult};

enum AppEvent {
    Input(event::KeyEvent),
    Tick,
    ThreadLoaded(String, Result<(GetConversationResult, Option<crate::types::Proposal>)>),
    ConversationsLoaded(InboxView, Result<Vec<ConversationRow>>),
}

const TABS: &[InboxView] = &[
    InboxView::Active,
    InboxView::NeedsReply,
    InboxView::Mine,
    InboxView::Resolved,
    InboxView::Archived,
];

pub async fn run(client: &TrpcClient, initial_conversations: Vec<ConversationRow>) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let client_arc = Arc::new(client.clone());
    let res = run_app(&mut terminal, client_arc, initial_conversations).await;

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;

    res
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    client: Arc<TrpcClient>,
    initial_conversations: Vec<ConversationRow>,
) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();
    let tick_rate = Duration::from_millis(200);

    // Event loop task
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        loop {
            if crossterm::event::poll(tick_rate).unwrap_or(false) {
                if let Ok(Event::Key(key)) = event::read() {
                    let _ = tx_clone.send(AppEvent::Input(key));
                }
            }
            let _ = tx_clone.send(AppEvent::Tick);
        }
    });

    let mut tab_index = 0;
    let mut conversations = initial_conversations;
    let mut list_state = ListState::default();
    if !conversations.is_empty() {
        list_state.select(Some(0));
    }

    let mut loading_list = false;
    let mut loading_thread = false;
    let mut active_thread: Option<(GetConversationResult, Option<crate::types::Proposal>)> = None;
    let mut active_thread_id: Option<String> = None;

    let mut active_thread_error: Option<String> = None;

    // Load initial thread if available
    if let Some(first) = conversations.first() {
        active_thread_id = Some(first.id.clone());
        loading_thread = true;
        let c = client.clone();
        let id = first.id.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let res = fetch_thread(&c, &id).await;
            let _ = tx.send(AppEvent::ThreadLoaded(id, res));
        });
    }

    loop {
        terminal.draw(|f| {
            let size = f.area();

            // Layout: Tabs at top, then split screen (left list, right thread)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
                .split(size);

            let tab_titles: Vec<Line> = TABS
                .iter()
                .map(|t| Line::from(format!("{:?}", t)))
                .collect();

            let tabs = Tabs::new(tab_titles)
                .block(Block::default().borders(Borders::ALL).title(" Inbox Tabs "))
                .select(tab_index)
                .style(Style::default().fg(Color::Cyan))
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                );
            f.render_widget(tabs, chunks[0]);

            let bottom_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(33), Constraint::Percentage(67)].as_ref())
                .split(chunks[1]);

            // LEFT PANE: List
            let items: Vec<ListItem> = conversations
                .iter()
                .map(|c| {
                    let subject = c.subject.as_deref().unwrap_or("No subject");
                    let cp = c
                        .counterpart
                        .as_ref()
                        .map(|cc| cc.name.as_str())
                        .unwrap_or("Unknown");

                    let mut style = Style::default();
                    if c.unread_count > 0 {
                        style = style.add_modifier(Modifier::BOLD).fg(Color::Red);
                    } else if c.needs_reply {
                        style = style.fg(Color::Yellow);
                    } else if c.status == "resolved" {
                        style = style.fg(Color::DarkGray);
                    }

                    let line1 =
                        Line::from(vec![Span::styled(crate::ui::truncate(subject, 30), style)]);
                    let line2 = Line::from(vec![
                        Span::raw(format!("{} | ", cp)),
                        Span::raw(if c.unread_count > 0 {
                            format!("{} unread", c.unread_count)
                        } else {
                            "".to_string()
                        }),
                    ]);

                    ListItem::new(vec![line1, line2])
                })
                .collect();

            let list_title = if loading_list {
                " Loading... "
            } else {
                " Conversations "
            };
            let list_widget = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(list_title))
                .highlight_style(
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");
            f.render_stateful_widget(list_widget, bottom_chunks[0], &mut list_state);

            // RIGHT PANE: Thread
            let thread_title = if loading_thread {
                " Thread (Loading...) ".to_string()
            } else {
                " Thread ".to_string()
            };

            let right_block = Block::default().borders(Borders::ALL).title(thread_title);

            if let Some((thread, proposal)) = &active_thread {
                let mut text = vec![];

                // Subject header
                text.push(Line::from(Span::styled(
                    thread
                        .conversation
                        .subject
                        .as_deref()
                        .unwrap_or("No subject"),
                    Style::default()
                        .add_modifier(Modifier::BOLD)
                        .fg(Color::Cyan),
                )));
                text.push(Line::from(Span::styled(
                    format!("ID: {}", thread.conversation.id),
                    Style::default().fg(Color::DarkGray),
                )));
                text.push(Line::from(""));

                // Proposal info if any
                if let Some(p) = proposal {
                    text.push(Line::from(vec![
                        Span::styled("Proposal: ", Style::default().fg(Color::Magenta)),
                        Span::raw(&p.title),
                        Span::styled(format!(" [{:?}]", p.status), Style::default().fg(Color::Yellow)),
                    ]));
                    text.push(Line::from(""));
                }

                // Participants
                let mut p_str = String::new();
                for p in &thread.participants {
                    p_str.push_str(p.name.as_deref().unwrap_or("Unknown"));
                    p_str.push_str(", ");
                }
                text.push(Line::from(Span::styled(
                    format!("Participants: {}", p_str),
                    Style::default().fg(Color::Yellow),
                )));
                text.push(Line::from("─".repeat(bottom_chunks[1].width as usize - 2)));
                text.push(Line::from(""));

                // Messages
                for msg in thread.messages.iter().rev() {
                    let author = msg.author_name.as_deref().unwrap_or("Unknown");
                    text.push(Line::from(vec![
                        Span::styled(
                            author,
                            Style::default()
                                .add_modifier(Modifier::BOLD)
                                .fg(Color::Blue),
                        ),
                        Span::styled(
                            format!(" [{}]", msg.created_at),
                            Style::default().fg(Color::DarkGray),
                        ),
                    ]));

                    for line in msg.body.lines() {
                        text.push(Line::from(line.to_string()));
                    }
                    text.push(Line::from(""));
                }

                // Keybinds at bottom
                text.push(Line::from("─".repeat(bottom_chunks[1].width as usize - 2)));
                text.push(Line::from(Span::styled("Press 'r' to reply", Style::default().fg(Color::DarkGray))));

                let p = Paragraph::new(Text::from(text))
                    .block(right_block)
                    .wrap(Wrap { trim: false });
                f.render_widget(p, bottom_chunks[1]);
            } else if let Some(e) = &active_thread_error {
                let p = Paragraph::new(Text::from(vec![
                    Line::from(Span::styled("Failed to load thread:", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
                    Line::from(""),
                    Line::from(Span::styled(e, Style::default().fg(Color::Red)))
                ])).block(right_block).wrap(Wrap { trim: false });
                f.render_widget(p, bottom_chunks[1]);
            } else {
                let p = Paragraph::new("Select a conversation...").block(right_block);
                f.render_widget(p, bottom_chunks[1]);
            }
        })?;

        if let Some(event) = rx.recv().await {
            match event {
                AppEvent::Input(key) => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Char('r') => {
                        if let Some(id) = &active_thread_id {
                            let _ = disable_raw_mode();
                            let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
                            
                            println!("Replying to thread: {}", id);
                            let reply_opt = dialoguer::Editor::new()
                                .require_save(false)
                                .edit("")
                                .unwrap_or(None);
                                
                            let _ = enable_raw_mode();
                            let _ = execute!(terminal.backend_mut(), EnterAlternateScreen);
                            let _ = terminal.clear();
                            
                            if let Some(text) = reply_opt {
                                if !text.trim().is_empty() {
                                    loading_thread = true;
                                    let c = client.clone();
                                    let id_clone = id.clone();
                                    let tx = tx.clone();
                                    tokio::spawn(async move {
                                        let _ = c.mutate::<serde_json::Value>(
                                            "message.send",
                                            &serde_json::json!({
                                                "conversationId": id_clone,
                                                "body": text,
                                            }),
                                        ).await;
                                        let res = fetch_thread(&c, &id_clone).await;
                                        let _ = tx.send(AppEvent::ThreadLoaded(id_clone, res));
                                    });
                                }
                            }
                        }
                    }
                    KeyCode::Right | KeyCode::Tab => {
                        tab_index = (tab_index + 1) % TABS.len();
                        loading_list = true;
                        let view = TABS[tab_index];
                        let c = client.clone();
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            let res = fetch_list(&c, view).await;
                            let _ = tx.send(AppEvent::ConversationsLoaded(view, res));
                        });
                    }
                    KeyCode::Left | KeyCode::BackTab => {
                        tab_index = (tab_index + TABS.len() - 1) % TABS.len();
                        loading_list = true;
                        let view = TABS[tab_index];
                        let c = client.clone();
                        let tx = tx.clone();
                        tokio::spawn(async move {
                            let res = fetch_list(&c, view).await;
                            let _ = tx.send(AppEvent::ConversationsLoaded(view, res));
                        });
                    }
                    KeyCode::Down | KeyCode::Char('j') => {
                        if !conversations.is_empty() {
                            let i = match list_state.selected() {
                                Some(i) => {
                                    if i >= conversations.len() - 1 {
                                        0
                                    } else {
                                        i + 1
                                    }
                                }
                                None => 0,
                            };
                            list_state.select(Some(i));

                            // Load thread
                            let id = conversations[i].id.clone();
                            active_thread_id = Some(id.clone());
                            active_thread_error = None;
                            loading_thread = true;
                            let c = client.clone();
                            let tx = tx.clone();
                            tokio::spawn(async move {
                                let res = fetch_thread(&c, &id).await;
                                let _ = tx.send(AppEvent::ThreadLoaded(id, res));
                            });
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        if !conversations.is_empty() {
                            let i = match list_state.selected() {
                                Some(i) => {
                                    if i == 0 {
                                        conversations.len() - 1
                                    } else {
                                        i - 1
                                    }
                                }
                                None => 0,
                            };
                            list_state.select(Some(i));

                            // Load thread
                            let id = conversations[i].id.clone();
                            active_thread_id = Some(id.clone());
                            active_thread_error = None;
                            loading_thread = true;
                            let c = client.clone();
                            let tx = tx.clone();
                            tokio::spawn(async move {
                                let res = fetch_thread(&c, &id).await;
                                let _ = tx.send(AppEvent::ThreadLoaded(id, res));
                            });
                        }
                    }
                    _ => {}
                },
                AppEvent::ConversationsLoaded(view, res) => {
                    if TABS[tab_index] == view {
                        loading_list = false;
                        match res {
                            Ok(list) => {
                                conversations = list;
                                if !conversations.is_empty() {
                                    list_state.select(Some(0));
                                    // Load first thread
                                    let id = conversations[0].id.clone();
                                    active_thread_id = Some(id.clone());
                                    active_thread_error = None;
                                    loading_thread = true;
                                    let c = client.clone();
                                    let tx = tx.clone();
                                    tokio::spawn(async move {
                                        let r = fetch_thread(&c, &id).await;
                                        let _ = tx.send(AppEvent::ThreadLoaded(id, r));
                                    });
                                } else {
                                    list_state.select(None);
                                    active_thread = None;
                                    active_thread_id = None;
                                }
                            }
                            Err(_) => {
                                conversations = vec![];
                                list_state.select(None);
                            }
                        }
                    }
                }
                AppEvent::ThreadLoaded(id, res) => {
                    if Some(id) == active_thread_id {
                        loading_thread = false;
                        match res {
                            Ok(thread) => {
                                active_thread = Some(thread);
                                active_thread_error = None;
                            }
                            Err(e) => {
                                active_thread = None;
                                active_thread_error = Some(e.to_string());
                            }
                        }
                    }
                }
                AppEvent::Tick => {}
            }
        }
    }
}

async fn fetch_list(client: &TrpcClient, view: InboxView) -> Result<Vec<ConversationRow>> {
    let args = super::ListArgs {
        view,
        ..Default::default()
    };
    let res: serde_json::Value = client
        .query(
            "message.listConversations",
            Some(&serde_json::to_value(&args)?),
        )
        .await?;

    if let Some(arr) = res.as_array() {
        Ok(serde_json::from_value(serde_json::Value::Array(
            arr.clone(),
        ))?)
    } else if let Some(arr) = res.get("conversations").and_then(|v| v.as_array()) {
        Ok(serde_json::from_value(serde_json::Value::Array(
            arr.clone(),
        ))?)
    } else {
        Ok(Vec::new())
    }
}

async fn fetch_thread(client: &TrpcClient, id: &str) -> Result<(GetConversationResult, Option<crate::types::Proposal>)> {
    let convo: GetConversationResult = client
        .query("message.getConversation", Some(&serde_json::json!({ "id": id })))
        .await?;
        
    let messages: Vec<crate::types::ConversationMessage> = client
        .query("message.listMessages", Some(&serde_json::json!({ "conversationId": id })))
        .await?;

    let mut c = convo;
    c.messages = messages;
    
    let mut proposal = None;
    if let Some(pid) = &c.conversation.proposal_id {
        if let Ok(p) = client.query::<crate::types::Proposal>("proposal.admin.getById", Some(&serde_json::json!({ "id": pid }))).await {
            proposal = Some(p);
        }
    }
    
    Ok((c, proposal))
}

// We still keep the format_item if used elsewhere
pub fn format_item(convo: &ConversationRow) -> String {
    let subject = convo.subject.as_deref().unwrap_or("No subject");
    let counterpart = convo
        .counterpart
        .as_ref()
        .map(|c| c.name.as_str())
        .unwrap_or("Unknown");

    let status = if convo.status == "resolved" {
        "Resolved".to_string()
    } else if convo.needs_reply {
        "Needs Reply".to_string()
    } else {
        "Open".to_string()
    };

    let unread = if convo.unread_count > 0 {
        format!("{} unread", convo.unread_count)
    } else {
        "".to_string()
    };

    format!(
        "{:<25} | {:<50} | {:<20} | {:<12} {}",
        convo.id,
        crate::ui::truncate(subject, 50),
        crate::ui::truncate(counterpart, 20),
        status,
        unread
    )
}
