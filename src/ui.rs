use crate::model::{KanbanColumns, Ticket};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table},
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
    let layout = determine_layout(area.width);
    
    match layout {
        LayoutType::FourColumn => draw_four_column_board(frame, area, columns),
        LayoutType::TwoColumn => draw_two_column_board(frame, area, columns),
        LayoutType::Vertical => draw_vertical_board(frame, area, columns),
    }
}

#[derive(Debug)]
enum LayoutType {
    Vertical,
    TwoColumn,
    FourColumn,
}

fn determine_layout(terminal_width: u16) -> LayoutType {
    match terminal_width {
        0..=79 => LayoutType::Vertical,
        80..=119 => LayoutType::TwoColumn,
        _ => LayoutType::FourColumn,
    }
}

fn draw_four_column_board(frame: &mut Frame, area: Rect, columns: &KanbanColumns) {
    let col_width = (area.width - 3) / 4;
    let widths = vec![
        Constraint::Length(col_width),
        Constraint::Length(col_width),
        Constraint::Length(col_width),
        Constraint::Length(col_width),
    ];
    
    let max_rows = columns.max_ticket_count();
    let mut rows = Vec::new();
    
    let header = Row::new(vec![
        Cell::from("TO DO").style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Cell::from("IN PROGRESS").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Cell::from("REVIEW").style(Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Cell::from("DONE").style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
    ])
    .height(1)
    .bottom_margin(1);
    
    for i in 0..max_rows {
        let todo_cell = columns.todo.get(i)
            .map(|t| format_ticket(t, col_width))
            .unwrap_or_else(|| Cell::from(""));
        
        let progress_cell = columns.in_progress.get(i)
            .map(|t| format_ticket(t, col_width))
            .unwrap_or_else(|| Cell::from(""));
        
        let review_cell = columns.review.get(i)
            .map(|t| format_ticket(t, col_width))
            .unwrap_or_else(|| Cell::from(""));
        
        let done_cell = columns.done.get(i)
            .map(|t| format_ticket(t, col_width))
            .unwrap_or_else(|| Cell::from(""));
        
        rows.push(Row::new(vec![todo_cell, progress_cell, review_cell, done_cell]).height(3));
    }
    
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .title("🦀 KANBARS - JIRA Board (press 'q' to quit)"));
    
    frame.render_widget(table, area);
}

fn draw_two_column_board(frame: &mut Frame, area: Rect, columns: &KanbanColumns) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);
    
    let left_columns = KanbanColumns {
        todo: columns.todo.clone(),
        in_progress: columns.in_progress.clone(),
        review: Vec::new(),
        done: Vec::new(),
    };
    
    let right_columns = KanbanColumns {
        todo: Vec::new(),
        in_progress: Vec::new(),
        review: columns.review.clone(),
        done: columns.done.clone(),
    };
    
    draw_two_column_section(frame, chunks[0], &left_columns, true);
    draw_two_column_section(frame, chunks[1], &right_columns, false);
}

fn draw_two_column_section(frame: &mut Frame, area: Rect, columns: &KanbanColumns, is_left: bool) {
    let col_width = (area.width - 3) / 2;
    let widths = vec![
        Constraint::Length(col_width),
        Constraint::Length(col_width),
    ];
    
    let (col1, col2, header1, header2, color1, color2) = if is_left {
        (
            &columns.todo,
            &columns.in_progress,
            "TO DO",
            "IN PROGRESS",
            Color::Cyan,
            Color::Yellow,
        )
    } else {
        (
            &columns.review,
            &columns.done,
            "REVIEW",
            "DONE",
            Color::Magenta,
            Color::Green,
        )
    };
    
    let max_rows = col1.len().max(col2.len());
    let mut rows = Vec::new();
    
    let header = Row::new(vec![
        Cell::from(header1).style(Style::default().fg(color1).add_modifier(Modifier::BOLD)),
        Cell::from(header2).style(Style::default().fg(color2).add_modifier(Modifier::BOLD)),
    ])
    .height(1)
    .bottom_margin(1);
    
    for i in 0..max_rows {
        let cell1 = col1.get(i)
            .map(|t| format_ticket(t, col_width))
            .unwrap_or_else(|| Cell::from(""));
        
        let cell2 = col2.get(i)
            .map(|t| format_ticket(t, col_width))
            .unwrap_or_else(|| Cell::from(""));
        
        rows.push(Row::new(vec![cell1, cell2]).height(3));
    }
    
    let title = if is_left { "TODO / IN PROGRESS" } else { "REVIEW / DONE" };
    
    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title));
    
    frame.render_widget(table, area);
}

fn draw_vertical_board(frame: &mut Frame, area: Rect, columns: &KanbanColumns) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);
    
    draw_column_section(frame, chunks[0], &columns.todo, "TO DO", Color::Cyan);
    draw_column_section(frame, chunks[1], &columns.in_progress, "IN PROGRESS", Color::Yellow);
    draw_column_section(frame, chunks[2], &columns.review, "REVIEW", Color::Magenta);
    draw_column_section(frame, chunks[3], &columns.done, "DONE", Color::Green);
}

fn draw_column_section(frame: &mut Frame, area: Rect, tickets: &[Ticket], title: &str, color: Color) {
    let width = area.width - 2;
    let mut rows = Vec::new();
    
    for ticket in tickets.iter().take(((area.height - 3) / 2) as usize) {
        rows.push(Row::new(vec![format_ticket(ticket, width)]).height(2));
    }
    
    let table = Table::new(rows, vec![Constraint::Percentage(100)])
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_style(Style::default().fg(color).add_modifier(Modifier::BOLD)));
    
    frame.render_widget(table, area);
}

fn format_ticket(ticket: &Ticket, width: u16) -> Cell<'static> {
    let emoji = ticket.ticket_type.emoji().to_string();
    
    let available_width = (width as usize).saturating_sub(2);
    let summary = if ticket.summary.len() > available_width {
        format!("{}...", &ticket.summary[..available_width.saturating_sub(3)])
    } else {
        ticket.summary.clone()
    };
    
    let lines = vec![
        Line::from(vec![
            Span::raw(emoji.clone()),
            Span::raw(" "),
            Span::styled(ticket.key.clone(), Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(summary),
    ];
    
    Cell::from(lines)
}