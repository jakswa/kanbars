use crate::config::Config;
use crate::model::{Ticket, TicketType};
use base64::{Engine as _, engine::general_purpose};
use reqwest::blocking::Client;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct JiraResponse {
    issues: Vec<JiraIssue>,
}

#[derive(Debug, Deserialize)]
struct JiraIssue {
    key: String,
    fields: JiraFields,
}

#[derive(Debug, Deserialize)]
struct JiraFields {
    summary: String,
    status: JiraStatus,
    issuetype: JiraIssueType,
    assignee: Option<JiraUser>,
}

#[derive(Debug, Deserialize)]
struct JiraStatus {
    name: String,
}

#[derive(Debug, Deserialize)]
struct JiraIssueType {
    name: String,
}

#[derive(Debug, Deserialize)]
struct JiraUser {
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    #[serde(rename = "emailAddress")]
    email_address: Option<String>,
}

pub fn fetch_tickets_api(config: &Config) -> Result<Vec<Ticket>, Box<dyn Error>> {
    let url = config.jira.url.as_ref()
        .ok_or("JIRA URL not configured. Set JIRA_URL or JIRA_SITE environment variable")?;
    let email = config.jira.email.as_ref()
        .ok_or("JIRA email not configured. Set JIRA_USER or JIRA_EMAIL environment variable")?;
    let token = config.jira.api_token.as_ref()
        .ok_or("JIRA API token not configured. Set JIRA_API_TOKEN environment variable")?;
    
    let client = Client::new();
    
    // Create basic auth header
    let auth = format!("{}:{}", email, token);
    let encoded = general_purpose::STANDARD.encode(auth.as_bytes());
    
    // Use the new v3 JQL search endpoint
    let api_url = format!("{}/rest/api/3/search/jql", url.trim_end_matches('/'));
    
    let response = client
        .get(&api_url)
        .header("Authorization", format!("Basic {}", encoded))
        .header("Accept", "application/json")
        .query(&[
            ("jql", config.query.jql.as_str()),
            ("maxResults", "100"),
            ("fields", "key,summary,status,issuetype,assignee"),
        ])
        .send()?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_else(|_| "Could not read response body".to_string());
        return Err(format!(
            "JIRA API request failed with status: {}\nResponse: {}",
            status,
            body
        ).into());
    }
    
    let jira_response: JiraResponse = response.json()?;
    
    let tickets: Vec<Ticket> = jira_response.issues
        .into_iter()
        .map(|issue| {
            let assignee = issue.fields.assignee
                .and_then(|u| u.display_name.or(u.email_address))
                .unwrap_or_else(|| "unassigned".to_string());
            
            Ticket {
                key: issue.key,
                ticket_type: TicketType::from_str(&issue.fields.issuetype.name),
                summary: issue.fields.summary,
                status: issue.fields.status.name,
                assignee,
            }
        })
        .collect();
    
    Ok(tickets)
}