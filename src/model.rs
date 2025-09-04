use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct Ticket {
    pub key: String,
    pub ticket_type: TicketType,
    pub summary: String,
    pub status: String,
    pub assignee: String,
    // Extended fields (fetched on demand)
    pub description: Option<String>,
    pub priority: Option<String>,
    pub reporter: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub labels: Option<Vec<String>>,
    pub comments: Option<Vec<Comment>>,
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub author: String,
    pub created: String,
    pub body: String,
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
pub struct StatusGroups {
    pub groups: BTreeMap<String, Vec<Ticket>>,
}

impl StatusGroups {
    pub fn new() -> Self {
        StatusGroups {
            groups: BTreeMap::new(),
        }
    }
    
    pub fn total_tickets(&self) -> usize {
        self.groups.values().map(|v| v.len()).sum()
    }
    
    pub fn get_ticket_by_index(&self, global_index: usize) -> Option<&Ticket> {
        let mut current_index = 0;
        
        for (_status, tickets) in self.groups.iter() {
            if global_index < current_index + tickets.len() {
                return tickets.get(global_index - current_index);
            }
            current_index += tickets.len();
        }
        
        None
    }
    
    pub fn from_tickets(mut tickets: Vec<Ticket>) -> Self {
        let mut groups = StatusGroups::new();
        
        // Sort tickets by status priority first
        tickets.sort_by(|a, b| {
            let a_priority = get_status_priority(&a.status);
            let b_priority = get_status_priority(&b.status);
            a_priority.cmp(&b_priority)
        });
        
        // Group tickets by their actual status
        for ticket in tickets {
            groups.groups
                .entry(ticket.status.clone())
                .or_insert_with(Vec::new)
                .push(ticket);
        }
        
        groups
    }
    
    pub fn print_simple(&self) {
        if self.groups.is_empty() {
            println!("No tickets found! 🎉");
            return;
        }
        
        // Print each status group
        for (status, tickets) in &self.groups {
            if !tickets.is_empty() {
                let emoji = get_status_emoji(status);
                println!("{} {} ({})", emoji, status.to_uppercase(), tickets.len());
                
                for ticket in tickets {
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
        }
    }
}


// Get a priority value for sorting statuses in logical workflow order
fn get_status_priority(status: &str) -> u8 {
    let status_lower = status.to_lowercase();
    
    // Priority 0-3: Todo-like statuses (leftmost)
    if status_lower.contains("backlog") { return 0; }
    if status_lower.contains("todo") || status_lower == "to do" { return 1; }
    if status_lower.contains("open") || status_lower.contains("new") { return 2; }
    if status_lower.contains("ready for development") || status_lower.contains("ready to start") { return 3; }
    
    // Priority 10-19: In Progress-like statuses
    if status_lower.contains("in progress") || status_lower.contains("in-progress") { return 10; }
    if status_lower.contains("development") || status_lower.contains("in dev") { return 11; }
    if status_lower.contains("coding") || status_lower.contains("implementing") { return 12; }
    if status_lower.contains("ready to ship") || status_lower.contains("ready for deploy") { return 15; }
    
    // Priority 20-29: Review-like statuses
    if status_lower.contains("review") || status_lower.contains("pr") { return 20; }
    if status_lower.contains("testing") || status_lower.contains("qa") { return 21; }
    if status_lower.contains("verification") || status_lower.contains("approval") { return 22; }
    if status_lower.contains("staging") { return 23; }
    
    // Priority 30-39: Done-like statuses (rightmost)
    if status_lower.contains("done") { return 30; }
    if status_lower.contains("closed") { return 31; }
    if status_lower.contains("resolved") { return 32; }
    if status_lower.contains("shipped") || status_lower.contains("deployed") { return 33; }
    if status_lower.contains("complete") { return 34; }
    
    // Unknown statuses go in the middle
    return 15;
}

// Get an appropriate emoji for a status
fn get_status_emoji(status: &str) -> &str {
    let status_lower = status.to_lowercase();
    
    if status_lower.contains("done") || status_lower.contains("closed") || 
       status_lower.contains("resolved") || status_lower.contains("complete") {
        return "✅";
    }
    if status_lower.contains("progress") || status_lower.contains("development") || 
       status_lower.contains("coding") || status_lower.contains("ship") {
        return "🚀";
    }
    if status_lower.contains("review") || status_lower.contains("testing") || 
       status_lower.contains("qa") || status_lower.contains("verification") {
        return "🔍";
    }
    if status_lower.contains("todo") || status_lower.contains("backlog") || 
       status_lower == "to do" || status_lower.contains("open") {
        return "📋";
    }
    
    // Default emoji for unknown statuses
    "📌"
}

// Get color for UI rendering
pub fn get_status_color(status: &str) -> ratatui::style::Color {
    use ratatui::style::Color;
    let status_lower = status.to_lowercase();
    
    if status_lower.contains("done") || status_lower.contains("closed") || 
       status_lower.contains("resolved") || status_lower.contains("complete") {
        return Color::Green;
    }
    if status_lower.contains("progress") || status_lower.contains("development") || 
       status_lower.contains("coding") || status_lower.contains("ship") {
        return Color::Yellow;
    }
    if status_lower.contains("review") || status_lower.contains("testing") || 
       status_lower.contains("qa") || status_lower.contains("verification") {
        return Color::Magenta;
    }
    if status_lower.contains("todo") || status_lower.contains("backlog") || 
       status_lower == "to do" || status_lower.contains("open") {
        return Color::Cyan;
    }
    
    // Default color for unknown statuses
    Color::Blue
}