use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "kanbars")]
#[command(about = "🦀 Lightweight Terminal Kanban for JIRA", long_about = None)]
pub struct Args {
    /// Custom JQL query
    #[arg(long)]
    pub jql: Option<String>,
    
    /// Filter by epic
    #[arg(long)]
    pub epic: Option<String>,
    
    /// Show tickets for a specific assignee
    #[arg(long)]
    pub assignee: Option<String>,
    
    /// JIRA instance URL (overrides config)
    #[arg(long)]
    pub url: Option<String>,
    
    /// Generate a sample config file
    #[arg(long)]
    pub init: bool,
    
    /// Auto-refresh interval in seconds (default: 60)
    #[arg(short = 'r', long = "refresh", default_value = "60")]
    pub refresh: u64,
    
    /// Display once and exit (useful with watch command)
    #[arg(long = "once")]
    pub once: bool,
}

impl Args {
    pub fn build_jql(&self, default_jql: &str) -> String {
        if let Some(ref jql) = self.jql {
            return jql.clone();
        }
        
        let mut jql = default_jql.to_string();
        
        if let Some(ref epic) = self.epic {
            jql = format!("\"Epic Link\" = {} AND {}", epic, jql);
        }
        
        if let Some(ref assignee) = self.assignee {
            // Replace currentUser() or any assignee clause
            if jql.contains("assignee") {
                jql = jql.replace("assignee = currentUser()", &format!("assignee = '{}'", assignee));
                jql = jql.replace("developer = 'Jake Swanson'", &format!("assignee = '{}'", assignee));
            } else {
                jql = format!("assignee = '{}' AND {}", assignee, jql);
            }
        }
        
        jql
    }
}