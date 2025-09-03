use crate::model::{KanbanColumns, Ticket};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

#[derive(Debug, Clone)]
pub enum UiMode {
    Board,
    Detail,
}

#[derive(Debug)]
pub struct AppState {
    pub mode: UiMode,
    pub selected_index: usize,  // Global index across all tickets
    pub detail_ticket: Option<Ticket>,
    pub detail_scroll: usize,
    pub detail_max_scroll: usize,  // Track the max valid scroll position
}

pub fn draw_ui(
    frame: &mut Frame, 
    columns: &KanbanColumns,
    last_update: Option<&chrono::DateTime<chrono::Local>>,
    paused: bool,
    refresh_seconds: u64,
    app_state: &mut AppState,
) {
    let size = frame.area();
    
    match app_state.mode {
        UiMode::Board => {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Min(0)])
                .split(size);
            draw_kanban_board(frame, chunks[0], columns, last_update, paused, refresh_seconds, app_state);
        }
        UiMode::Detail => {
            if app_state.detail_ticket.is_some() {
                draw_ticket_detail(frame, size, app_state);
            }
        }
    }
}

fn draw_kanban_board(
    frame: &mut Frame, 
    area: Rect, 
    columns: &KanbanColumns,
    last_update: Option<&chrono::DateTime<chrono::Local>>,
    paused: bool,
    refresh_seconds: u64,
    app_state: &AppState,
) {
    // Always use horizontal lanes for better space utilization
    draw_horizontal_lanes(frame, area, columns, last_update, paused, refresh_seconds, app_state);
}

fn draw_horizontal_lanes(
    frame: &mut Frame, 
    area: Rect, 
    columns: &KanbanColumns,
    last_update: Option<&chrono::DateTime<chrono::Local>>,
    paused: bool,
    refresh_seconds: u64,
    app_state: &AppState,
) {
    // Count non-empty lanes
    let mut active_lanes = Vec::new();
    if !columns.todo.is_empty() {
        active_lanes.push(("TO DO", &columns.todo, Color::Cyan));
    }
    if !columns.in_progress.is_empty() {
        active_lanes.push(("IN PROGRESS", &columns.in_progress, Color::Yellow));
    }
    if !columns.review.is_empty() {
        active_lanes.push(("REVIEW", &columns.review, Color::Magenta));
    }
    if !columns.done.is_empty() {
        active_lanes.push(("DONE", &columns.done, Color::Green));
    }
    
    // If no tickets at all, show a message
    if active_lanes.is_empty() {
        let message = Paragraph::new("No tickets found! 🎉")
            .block(Block::default()
                .borders(Borders::ALL)
                .title("🦀 KANBARS"))
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(message, area);
        return;
    }
    
    // Split into title and active lanes
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),     // Title bar
            Constraint::Min(0),        // Rest for lanes
        ])
        .split(area);
    
    // Split the rest into equal lanes for active categories only
    let lane_count = active_lanes.len();
    let lane_constraints: Vec<Constraint> = (0..lane_count)
        .map(|_| Constraint::Ratio(1, lane_count as u32))
        .collect();
    
    let lane_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(lane_constraints)
        .split(main_chunks[1]);
    
    // Title with status information
    let mut title_str = String::from("🦀 KANBARS");
    
    // Add last update time
    if let Some(update_time) = last_update {
        title_str.push_str(&format!(" | Updated: {}", update_time.format("%H:%M:%S")));
    }
    
    // Add refresh status
    if paused {
        title_str.push_str(" | ⏸ PAUSED");
    } else {
        title_str.push_str(&format!(" | ↻ {}s", refresh_seconds));
    }
    
    // Add controls hint
    title_str.push_str(" | q:quit r:refresh p:pause ↑↓/jk:navigate Enter:detail");
    
    let title = Block::default()
        .borders(Borders::BOTTOM)
        .title(title_str);
    frame.render_widget(title, main_chunks[0]);
    
    // Render only non-empty lanes with proper selection tracking
    let mut global_ticket_index = 0;
    for (i, (title, tickets, color)) in active_lanes.iter().enumerate() {
        // Calculate which ticket in this lane is selected (if any)
        let selected_ticket = if app_state.selected_index >= global_ticket_index && 
                                 app_state.selected_index < global_ticket_index + tickets.len() {
            Some(app_state.selected_index - global_ticket_index)
        } else {
            None
        };
        
        draw_lane(frame, lane_chunks[i], tickets, title, *color, selected_ticket);
        global_ticket_index += tickets.len();
    }
}

fn draw_lane(frame: &mut Frame, area: Rect, tickets: &[Ticket], title: &str, color: Color, selected_ticket: Option<usize>) {
    // Split lane into label and content
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(12),  // Lane label
            Constraint::Min(0),      // Tickets
        ])
        .split(area);
    
    // Lane label with colored border
    let label = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(color))
        .title(title)
        .title_style(Style::default().fg(color).add_modifier(Modifier::BOLD));
    frame.render_widget(label, chunks[0]);
    
    // Build ticket lines
    let mut lines: Vec<Line> = Vec::new();
    let content_width = chunks[1].width as usize;
    
    for (i, ticket) in tickets.iter().enumerate() {
        let is_selected = selected_ticket == Some(i);
        
        if i > 0 && lines.len() < area.height as usize - 1 {
            // Add subtle separator between tickets
            lines.push(Line::from(""));
        }
        
        // Format ticket on 1-2 lines
        let emoji = ticket.ticket_type.emoji();
        let key = &ticket.key;
        let summary = &ticket.summary;
        
        // Extract assignee username (before @ if email, otherwise full string)
        let assignee = ticket.assignee
            .split('@')
            .next()
            .unwrap_or(&ticket.assignee)
            .trim();
        
        // First line: emoji + key + assignee + as much summary as fits
        let prefix = if !assignee.is_empty() && assignee != "unassigned" {
            format!("{} {} @{} ", emoji, key, assignee)
        } else {
            format!("{} {} ", emoji, key)
        };
        let prefix_len = prefix.len() + 3; // +3 for " • "
        
        let available_for_summary = content_width.saturating_sub(prefix_len);
        
        // Build the main ticket line
        let key_style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD).add_modifier(Modifier::UNDERLINED)
        } else {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        };
        
        let mut main_line_spans = vec![];
        
        // Add selection indicator
        if is_selected {
            main_line_spans.push(Span::styled("▶ ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
        } else {
            main_line_spans.push(Span::raw("  "));
        }
        
        main_line_spans.extend(vec![
            Span::raw(format!("{} ", emoji)),
            Span::styled(key.clone(), key_style),
        ]);
        
        // Add assignee if present
        if !assignee.is_empty() && assignee != "unassigned" {
            main_line_spans.push(Span::styled(
                format!(" @{}", assignee),
                Style::default().fg(Color::Blue),
            ));
        }
        
        main_line_spans.push(Span::styled(" • ", Style::default().fg(Color::DarkGray)));
        
        // Add summary text and handle wrapping
        if summary.len() <= available_for_summary {
            // Simple case: everything fits on one line
            main_line_spans.push(Span::raw(summary.clone()));
            lines.push(Line::from(main_line_spans));
        } else {
            // Need to wrap to second line
            let words: Vec<&str> = summary.split_whitespace().collect();
            let mut first_line = String::new();
            let mut second_line = String::new();
            let mut current_len = 0;
            
            for word in &words {
                if current_len + word.len() + 1 <= available_for_summary {
                    if !first_line.is_empty() {
                        first_line.push(' ');
                        current_len += 1;
                    }
                    first_line.push_str(word);
                    current_len += word.len();
                } else if second_line.is_empty() || second_line.len() + word.len() + 1 <= content_width - 4 {
                    if !second_line.is_empty() {
                        second_line.push(' ');
                    }
                    second_line.push_str(word);
                }
            }
            
            main_line_spans.push(Span::raw(first_line));
            lines.push(Line::from(main_line_spans));
            
            // Add continuation line if we have more text
            if !second_line.is_empty() {
                lines.push(Line::from(vec![
                    Span::raw("    "), // Indent
                    Span::styled(second_line, Style::default().fg(Color::Gray)),
                ]));
            }
        }
        
        // Stop if we're running out of vertical space
        if lines.len() >= area.height as usize - 1 {
            break;
        }
    }
    
    // Add overflow indicator if needed
    if tickets.len() > tickets.iter().take_while(|_| lines.len() < area.height as usize - 1).count() {
        let remaining = tickets.len() - tickets.iter().take_while(|_| lines.len() < area.height as usize - 1).count();
        lines.push(Line::from(Span::styled(
            format!("  ...and {} more", remaining),
            Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
        )));
    }
    
    let content = Paragraph::new(lines)
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default());
    
    frame.render_widget(content, chunks[1]);
}

fn draw_ticket_detail(frame: &mut Frame, area: Rect, app_state: &mut AppState) {
    let ticket = match &app_state.detail_ticket {
        Some(t) => t,
        None => return,
    };
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),    // Header
            Constraint::Min(0),       // Content
            Constraint::Length(1),    // Footer
        ])
        .split(area);
    
    // Header with ticket key and type
    let header = Block::default()
        .borders(Borders::BOTTOM)
        .title(format!("{} {} - {}", 
            ticket.ticket_type.emoji(),
            ticket.key,
            ticket.summary
        ))
        .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    frame.render_widget(header, chunks[0]);
    
    // Build content lines
    let mut lines = Vec::new();
    
    // Status and assignee
    lines.push(Line::from(vec![
        Span::styled("Status: ", Style::default().fg(Color::Gray)),
        Span::styled(&ticket.status, Style::default().fg(Color::Yellow)),
        Span::raw("  "),
        Span::styled("Assignee: ", Style::default().fg(Color::Gray)),
        Span::styled(&ticket.assignee, Style::default().fg(Color::Blue)),
    ]));
    lines.push(Line::from(""));
    
    // Priority if available
    if let Some(ref priority) = ticket.priority {
        lines.push(Line::from(vec![
            Span::styled("Priority: ", Style::default().fg(Color::Gray)),
            Span::styled(priority, Style::default().fg(Color::Magenta)),
        ]));
    }
    
    // Reporter if available
    if let Some(ref reporter) = ticket.reporter {
        lines.push(Line::from(vec![
            Span::styled("Reporter: ", Style::default().fg(Color::Gray)),
            Span::styled(reporter, Style::default().fg(Color::Blue)),
        ]));
    }
    
    // Created/Updated dates
    if ticket.created.is_some() || ticket.updated.is_some() {
        let mut date_spans = Vec::new();
        if let Some(ref created) = ticket.created {
            date_spans.push(Span::styled("Created: ", Style::default().fg(Color::Gray)));
            date_spans.push(Span::styled(created, Style::default().fg(Color::DarkGray)));
        }
        if let Some(ref updated) = ticket.updated {
            if !date_spans.is_empty() {
                date_spans.push(Span::raw("  "));
            }
            date_spans.push(Span::styled("Updated: ", Style::default().fg(Color::Gray)));
            date_spans.push(Span::styled(updated, Style::default().fg(Color::DarkGray)));
        }
        lines.push(Line::from(date_spans));
    }
    
    // Labels if available
    if let Some(ref labels) = ticket.labels {
        if !labels.is_empty() {
            let mut label_spans = vec![
                Span::styled("Labels: ", Style::default().fg(Color::Gray)),
            ];
            for (i, label) in labels.iter().enumerate() {
                if i > 0 {
                    label_spans.push(Span::raw(", "));
                }
                label_spans.push(Span::styled(label, Style::default().fg(Color::Cyan)));
            }
            lines.push(Line::from(label_spans));
        }
    }
    
    lines.push(Line::from(""));
    
    // Description
    lines.push(Line::from(Span::styled("Description:", Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD))));
    
    if let Some(ref desc) = ticket.description {
        // Split description into lines
        for line in desc.lines() {
            lines.push(Line::from(line.to_string()));
        }
    } else {
        lines.push(Line::from(Span::styled("(No description available)", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))));
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Note: Full details may not be available. Check JIRA API config.", Style::default().fg(Color::DarkGray))));
    }
    
    // Comments
    if let Some(ref comments) = ticket.comments {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(format!("Comments ({})", comments.len()), Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD))));
        for comment in comments {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(&comment.author, Style::default().fg(Color::Blue)),
                Span::raw(" - "),
                Span::styled(&comment.created, Style::default().fg(Color::DarkGray)),
            ]));
            lines.push(Line::from(&comment.body[..]));
        }
    }
    
    // Apply scroll offset - ensure we don't scroll past the end
    let visible_lines = chunks[1].height as usize;
    let total_lines = lines.len();
    let max_scroll = total_lines.saturating_sub(visible_lines);
    
    // Update the max scroll in app state
    app_state.detail_max_scroll = max_scroll;
    
    // Clamp the current scroll to valid range
    app_state.detail_scroll = app_state.detail_scroll.min(max_scroll);
    let actual_scroll = app_state.detail_scroll;
    
    let visible_content: Vec<Line> = lines.into_iter()
        .skip(actual_scroll)
        .take(visible_lines)
        .collect();
    
    let content = Paragraph::new(visible_content)
        .block(Block::default().borders(Borders::NONE))
        .wrap(Wrap { trim: true });
    frame.render_widget(content, chunks[1]);
    
    // Footer with controls and scroll position
    let scroll_info = if total_lines > visible_lines {
        format!(" [{}-{}/{}]", 
            actual_scroll + 1, 
            (actual_scroll + visible_lines).min(total_lines),
            total_lines
        )
    } else {
        String::new()
    };
    
    let footer_text = format!("ESC/q: Back  ↑↓/jk: Scroll  PgUp/PgDn: Page{}", scroll_info);
    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::TOP));
    frame.render_widget(footer, chunks[2]);
}