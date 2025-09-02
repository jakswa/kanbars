use crate::model::{KanbanColumns, Ticket};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub fn draw_ui(frame: &mut Frame, columns: &KanbanColumns) {
    let size = frame.area();
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0)])
        .split(size);

    draw_kanban_board(frame, chunks[0], columns);
}

fn draw_kanban_board(frame: &mut Frame, area: Rect, columns: &KanbanColumns) {
    // Always use horizontal lanes for better space utilization
    draw_horizontal_lanes(frame, area, columns);
}

fn draw_horizontal_lanes(frame: &mut Frame, area: Rect, columns: &KanbanColumns) {
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
    
    // Title
    let title = Block::default()
        .borders(Borders::BOTTOM)
        .title("🦀 KANBARS - JIRA Board (press 'q' to quit)");
    frame.render_widget(title, main_chunks[0]);
    
    // Render only non-empty lanes
    for (i, (title, tickets, color)) in active_lanes.iter().enumerate() {
        draw_lane(frame, lane_chunks[i], tickets, title, *color);
    }
}

fn draw_lane(frame: &mut Frame, area: Rect, tickets: &[Ticket], title: &str, color: Color) {
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
        let mut main_line_spans = vec![
            Span::raw(format!("{} ", emoji)),
            Span::styled(key.clone(), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ];
        
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