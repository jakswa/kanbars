# KANBARS 🦀 - CLI Kanban Board for JIRA

A lightweight, terminal-based kanban board for JIRA tickets, built with Rust and ratatui.

## Overview

Kanbars replaces the heavy JIRA browser tab (300MB RAM) with a fast, responsive CLI tool that displays tickets in a proper kanban layout.

## Technical Stack

- **Language**: Rust
- **TUI Framework**: ratatui (with crossterm backend)
- **Data Source**: `acli jira` commands
- **Output**: Single compiled binary at `/home/jake/work/.claude/kanbars`

## Architecture

### 1. Project Structure
```
/home/jake/sandbox/kanbars/
├── Cargo.toml
├── src/
│   ├── main.rs         # Entry point & terminal setup
│   ├── jira.rs         # ACLI integration & parsing
│   ├── model.rs        # Data structures (Ticket, Status enums)
│   └── ui.rs           # Ratatui table rendering
```

### 2. Dependencies (Cargo.toml)
```toml
[package]
name = "kanbars"
version = "0.1.0"
edition = "2021"

[dependencies]
ratatui = "0.28"
crossterm = "0.28"
```

### 3. Data Flow

```
acli jira workitem search (JQL query)
    ↓ (stdout - table format)
parse_acli_output()
    ↓ (Vec<Ticket>)
categorize_by_status()
    ↓ (KanbanColumns struct)
render_table()
    ↓ (ratatui Table widget)
Terminal Display
```

### 4. Core Data Structures

```rust
struct Ticket {
    key: String,
    ticket_type: TicketType,
    summary: String,
    status: String,
    assignee: String,
}

enum TicketType {
    Story,
    Bug,
    Task,
    Epic,
}

struct KanbanColumns {
    todo: Vec<Ticket>,
    in_progress: Vec<Ticket>,
    review: Vec<Ticket>,
    done: Vec<Ticket>,
}
```

### 5. ACLI Parsing Strategy

The ACLI output uses fixed-width columns:
- Type: columns 1-20
- Key: columns 21-40
- Assignee: columns 41-69
- Priority: columns 70-89
- Status: columns 90-108
- Summary: columns 109+

```rust
fn parse_acli_line(line: &str) -> Option<Ticket> {
    // Skip header and empty lines
    if !line.starts_with("Story") && !line.starts_with("Bug") && 
       !line.starts_with("Task") && !line.starts_with("Epic") {
        return None;
    }
    
    Some(Ticket {
        ticket_type: parse_type(&line[0..20]),
        key: line[20..40].trim().to_string(),
        status: line[89..108].trim().to_string(),
        summary: line[108..].trim().to_string(),
        assignee: line[40..69].trim().to_string(),
    })
}
```

### 6. Status Categorization

```rust
fn categorize_status(status: &str) -> Column {
    match status {
        "To Do" | "Open" | "Ready for Development" | "Backlog" => Column::Todo,
        "In Progress" | "Development" => Column::InProgress,
        "Peer Review" | "Code Review" | "QA Review" | 
        "Product Review" | "Testing" => Column::Review,
        "Done" | "Shipped" | "Closed" | "Resolved" => Column::Done,
        _ => Column::Todo,  // Default fallback
    }
}
```

### 7. Responsive Layout

```rust
fn determine_layout(terminal_width: u16) -> Layout {
    match terminal_width {
        0..=79 => Layout::Vertical,      // List view
        80..=119 => Layout::TwoColumn,   // TODO+PROGRESS | REVIEW+DONE
        _ => Layout::FourColumn,          // Full kanban
    }
}
```

### 8. Table Rendering

```rust
fn create_kanban_table(columns: &KanbanColumns, width: u16) -> Table {
    // Calculate column widths
    let col_width = (width - 3) / 4;  // -3 for borders
    let widths = vec![Constraint::Length(col_width); 4];
    
    // Build rows - align tickets horizontally
    let max_rows = columns.max_ticket_count();
    let mut rows = Vec::new();
    
    for i in 0..max_rows {
        let todo_cell = columns.todo.get(i)
            .map(|t| format_ticket(t))
            .unwrap_or_default();
        let progress_cell = columns.in_progress.get(i)
            .map(|t| format_ticket(t))
            .unwrap_or_default();
        // ... etc for review and done
        
        rows.push(Row::new(vec![todo_cell, progress_cell, review_cell, done_cell]));
    }
    
    Table::new(rows, widths)
        .header(Row::new(vec![
            Cell::from("TO DO").style(Style::default().fg(Color::Cyan)),
            Cell::from("IN PROGRESS").style(Style::default().fg(Color::Yellow)),
            Cell::from("REVIEW").style(Style::default().fg(Color::Magenta)),
            Cell::from("DONE").style(Style::default().fg(Color::Green)),
        ]))
        .block(Block::default().borders(Borders::ALL).title("KANBARS"))
}
```

### 9. Ticket Formatting

```rust
fn format_ticket(ticket: &Ticket) -> String {
    let type_indicator = match ticket.ticket_type {
        TicketType::Bug => "🐛",
        TicketType::Story => "📖",
        TicketType::Task => "✓",
        TicketType::Epic => "🎯",
    };
    
    let summary = if ticket.summary.len() > 20 {
        format!("{}...", &ticket.summary[..17])
    } else {
        ticket.summary.clone()
    };
    
    format!("{} {}\n{}", type_indicator, ticket.key, summary)
}
```

## Build & Deployment

```bash
# Build the project
cd /home/jake/sandbox/kanbars
cargo build --release

# Copy binary to accessible location
cp target/release/kanbars /home/jake/work/.claude/kanbars

# Make it runnable if needed
chmod +x /home/jake/work/.claude/kanbars
```

## Testing Strategy

1. **Parse sample ACLI output** - Verify ticket extraction
2. **Test with varying terminal widths** - Ensure responsive layout
3. **Handle edge cases**:
   - No tickets in a column
   - Very long ticket summaries
   - Unknown status values
   - Terminal resize during display

## Future Enhancements

### Phase 2 - Interactivity
- Arrow keys to navigate tickets
- Enter to view ticket details (call workitem-view.sh)
- 'r' to refresh data
- 'q' to quit

### Phase 3 - Advanced Features
- Cache JIRA data with TTL
- Filter by epic
- Show assignee avatars/initials
- Ticket age indicators
- Priority indicators

## Command Line Usage

```bash
# Basic usage - show all tickets where Jake is developer
kanbars

# Future: Filter by epic
kanbars --epic CR-12345

# Future: Show different developer
kanbars --developer "other@callrail.com"
```

## Performance Goals

- Startup time: < 100ms
- Memory usage: < 10MB
- Binary size: < 5MB

## Error Handling

- Gracefully handle ACLI failures
- Show clear error messages if JIRA is unreachable
- Fall back to cached data if available

## Testing Commands

```bash
# Test ACLI parsing
acli jira workitem search --jql "developer = 'Jake Swanson' AND status NOT IN ('Done', 'Shipped', 'Discontinued', 'Closed', 'Hibernate')" --paginate | head -10

# Test terminal width detection
echo "Terminal width: $(tput cols)"

# Run the tool
/home/jake/work/.claude/kanbars
```

---

This plan provides a solid foundation for implementing kanbars. The key insight is keeping it simple initially - just parse, categorize, and display. We can add interactivity and advanced features once the core rendering is solid.
