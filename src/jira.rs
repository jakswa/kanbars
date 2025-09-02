use crate::model::{Ticket, TicketType};
use std::error::Error;
use std::process::Command;

pub fn fetch_tickets() -> Result<Vec<Ticket>, Box<dyn Error>> {
    let output = Command::new("acli")
        .arg("jira")
        .arg("workitem")
        .arg("search")
        .arg("--jql")
        .arg("developer = 'Jake Swanson' AND status NOT IN ('Done', 'Shipped', 'Discontinued', 'Closed', 'Hibernate')")
        .arg("--paginate")
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to fetch JIRA tickets: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_acli_output(&stdout)
}

fn parse_acli_output(output: &str) -> Result<Vec<Ticket>, Box<dyn Error>> {
    let mut tickets = Vec::new();
    let lines: Vec<&str> = output.lines().collect();
    
    for (i, line) in lines.iter().enumerate() {
        if i == 0 && line.starts_with("Type") {
            continue;
        }
        if let Some(ticket) = parse_acli_line(line) {
            tickets.push(ticket);
        }
    }
    
    Ok(tickets)
}

fn parse_acli_line(line: &str) -> Option<Ticket> {
    if line.trim().is_empty() {
        return None;
    }
    
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 5 {
        return None;
    }
    
    let first_part = parts[0].to_lowercase();
    if !matches!(
        first_part.as_str(),
        "story" | "bug" | "task" | "epic"
    ) {
        return None;
    }
    
    let ticket_type = parts[0];
    let key = parts[1];
    
    let mut assignee_parts = Vec::new();
    let mut priority_idx = None;
    
    for (i, part) in parts.iter().enumerate().skip(2) {
        let part_lower = part.to_lowercase();
        if part_lower == "medium" || part_lower == "high" || part_lower == "low" || part_lower == "critical" {
            priority_idx = Some(i);
            break;
        }
        assignee_parts.push(*part);
    }
    
    let assignee = assignee_parts.join(" ");
    
    let (status, summary) = if let Some(idx) = priority_idx {
        if idx + 1 < parts.len() {
            let mut status_parts = Vec::new();
            let mut summary_start_idx = idx + 1;
            
            for (i, part) in parts.iter().enumerate().skip(idx + 1) {
                if !is_status_word(part) || status_parts.len() >= 3 {
                    summary_start_idx = i;
                    break;
                }
                status_parts.push(*part);
            }
            
            let status = status_parts.join(" ");
            let summary = parts[summary_start_idx..].join(" ");
            (status, summary)
        } else {
            (String::new(), String::new())
        }
    } else {
        (String::new(), String::new())
    };

    Some(Ticket {
        ticket_type: TicketType::from_str(ticket_type),
        key: key.to_string(),
        status,
        summary,
        assignee,
    })
}

fn find_status_end(parts: &[&str]) -> usize {
    for (i, part) in parts.iter().enumerate() {
        if !is_status_word(part) {
            return i;
        }
    }
    parts.len()
}

fn is_status_word(word: &str) -> bool {
    let status_words = [
        "to", "do", "in", "progress", "peer", "review", "code", 
        "qa", "product", "testing", "done", "shipped", "closed", 
        "resolved", "open", "ready", "for", "development", "backlog"
    ];
    status_words.contains(&word.to_lowercase().as_str())
}