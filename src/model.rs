#[derive(Debug, Clone)]
pub struct Ticket {
    pub key: String,
    pub ticket_type: TicketType,
    pub summary: String,
    pub status: String,
    pub assignee: String,
}

#[derive(Debug, Clone)]
pub enum TicketType {
    Story,
    Bug,
    Task,
    Epic,
}

impl TicketType {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "story" => TicketType::Story,
            "bug" => TicketType::Bug,
            "task" => TicketType::Task,
            "epic" => TicketType::Epic,
            _ => TicketType::Task,
        }
    }

    pub fn emoji(&self) -> &str {
        match self {
            TicketType::Bug => "🐛",
            TicketType::Story => "📖",
            TicketType::Task => "✓",
            TicketType::Epic => "🎯",
        }
    }
}

#[derive(Debug)]
pub struct KanbanColumns {
    pub todo: Vec<Ticket>,
    pub in_progress: Vec<Ticket>,
    pub review: Vec<Ticket>,
    pub done: Vec<Ticket>,
}

impl KanbanColumns {
    pub fn new() -> Self {
        KanbanColumns {
            todo: Vec::new(),
            in_progress: Vec::new(),
            review: Vec::new(),
            done: Vec::new(),
        }
    }

    pub fn from_tickets(tickets: Vec<Ticket>) -> Self {
        let mut columns = KanbanColumns::new();
        
        for ticket in tickets {
            match categorize_status(&ticket.status) {
                Column::Todo => columns.todo.push(ticket),
                Column::InProgress => columns.in_progress.push(ticket),
                Column::Review => columns.review.push(ticket),
                Column::Done => columns.done.push(ticket),
            }
        }
        
        columns
    }

    pub fn print_simple(&self) {
        // Print simple text output for --once mode
        if !self.todo.is_empty() {
            println!("📋 TO DO ({})", self.todo.len());
            for ticket in &self.todo {
                let assignee = if !ticket.assignee.is_empty() && ticket.assignee != "unassigned" {
                    format!(" @{}", ticket.assignee.split('@').next().unwrap_or(&ticket.assignee))
                } else {
                    String::new()
                };
                println!("  {} {}{} - {}", 
                    ticket.ticket_type.emoji(), 
                    ticket.key, 
                    assignee,
                    ticket.summary
                );
            }
            println!();
        }
        
        if !self.in_progress.is_empty() {
            println!("🚀 IN PROGRESS ({})", self.in_progress.len());
            for ticket in &self.in_progress {
                let assignee = if !ticket.assignee.is_empty() && ticket.assignee != "unassigned" {
                    format!(" @{}", ticket.assignee.split('@').next().unwrap_or(&ticket.assignee))
                } else {
                    String::new()
                };
                println!("  {} {}{} - {}", 
                    ticket.ticket_type.emoji(), 
                    ticket.key, 
                    assignee,
                    ticket.summary
                );
            }
            println!();
        }
        
        if !self.review.is_empty() {
            println!("🔍 REVIEW ({})", self.review.len());
            for ticket in &self.review {
                let assignee = if !ticket.assignee.is_empty() && ticket.assignee != "unassigned" {
                    format!(" @{}", ticket.assignee.split('@').next().unwrap_or(&ticket.assignee))
                } else {
                    String::new()
                };
                println!("  {} {}{} - {}", 
                    ticket.ticket_type.emoji(), 
                    ticket.key, 
                    assignee,
                    ticket.summary
                );
            }
            println!();
        }
        
        if !self.done.is_empty() {
            println!("✅ DONE ({})", self.done.len());
            for ticket in &self.done {
                let assignee = if !ticket.assignee.is_empty() && ticket.assignee != "unassigned" {
                    format!(" @{}", ticket.assignee.split('@').next().unwrap_or(&ticket.assignee))
                } else {
                    String::new()
                };
                println!("  {} {}{} - {}", 
                    ticket.ticket_type.emoji(), 
                    ticket.key, 
                    assignee,
                    ticket.summary
                );
            }
            println!();
        }
        
        if self.todo.is_empty() && self.in_progress.is_empty() && self.review.is_empty() && self.done.is_empty() {
            println!("No tickets found! 🎉");
        }
    }
}

#[derive(Debug)]
enum Column {
    Todo,
    InProgress,
    Review,
    Done,
}

fn categorize_status(status: &str) -> Column {
    let status_lower = status.to_lowercase();
    match status_lower.as_str() {
        "to do" | "open" | "ready for development" | "backlog" => Column::Todo,
        "in progress" | "development" => Column::InProgress,
        "peer review" | "code review" | "qa review" | "product review" | "testing" => {
            Column::Review
        }
        "done" | "shipped" | "closed" | "resolved" => Column::Done,
        _ => Column::Todo,
    }
}