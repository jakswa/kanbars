use crate::config::Config;
use crate::model::{Ticket, TicketType, Comment};
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
                description: None,
                priority: None,
                reporter: None,
                created: None,
                updated: None,
                labels: None,
                comments: None,
            }
        })
        .collect();
    
    Ok(tickets)
}

// We use raw JSON parsing for ticket details to handle different JIRA configurations

pub fn fetch_ticket_details(config: &Config, ticket_key: &str) -> Result<Ticket, Box<dyn Error>> {
    let url = config.jira.url.as_ref()
        .ok_or("JIRA URL not configured")?;
    let email = config.jira.email.as_ref()
        .ok_or("JIRA email not configured")?;
    let token = config.jira.api_token.as_ref()
        .ok_or("JIRA API token not configured")?;
    
    let client = Client::new();
    
    // Create basic auth header
    let auth = format!("{}:{}", email, token);
    let encoded = general_purpose::STANDARD.encode(auth.as_bytes());
    
    // Fetch detailed issue information
    let api_url = format!("{}/rest/api/3/issue/{}", 
        url.trim_end_matches('/'), ticket_key);
    
    let response = client
        .get(&api_url)
        .header("Authorization", format!("Basic {}", encoded))
        .header("Accept", "application/json")
        .send()?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().unwrap_or_else(|_| "Could not read response body".to_string());
        return Err(format!(
            "Failed to fetch ticket details: {}\nResponse: {}",
            status,
            body
        ).into());
    }
    
    // Parse as raw JSON for flexibility
    let json: serde_json::Value = response.json()?;
    
    // Extract fields safely
    let fields = json.get("fields").ok_or("No fields in response")?;
    let key = json.get("key")
        .and_then(|k| k.as_str())
        .ok_or("No key in response")?
        .to_string();
    
    // Extract basic fields
    let summary = fields.get("summary")
        .and_then(|s| s.as_str())
        .unwrap_or("No summary")
        .to_string();
    
    let status = fields.get("status")
        .and_then(|s| s.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("Unknown")
        .to_string();
    
    let issue_type = fields.get("issuetype")
        .and_then(|t| t.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("Task")
        .to_string();
    
    let assignee = fields.get("assignee")
        .and_then(|a| {
            a.get("displayName").and_then(|d| d.as_str())
                .or_else(|| a.get("emailAddress").and_then(|e| e.as_str()))
        })
        .unwrap_or("unassigned")
        .to_string();
    
    let reporter = fields.get("reporter")
        .and_then(|r| {
            r.get("displayName").and_then(|d| d.as_str())
                .or_else(|| r.get("emailAddress").and_then(|e| e.as_str()))
        })
        .map(|s| s.to_string());
    
    let priority = fields.get("priority")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string());
    
    let created = fields.get("created")
        .and_then(|c| c.as_str())
        .map(|s| s.to_string());
    
    let updated = fields.get("updated")
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());
    
    let labels = fields.get("labels")
        .and_then(|l| l.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .map(|s| s.to_string())
                .collect()
        });
    
    // Parse description - can be string, null, or ADF object
    let description = fields.get("description").and_then(|desc| {
        match desc {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Object(_) => extract_text_from_adf(desc),
            serde_json::Value::Null => None,
            _ => None,
        }
    });
    
    // Parse comments
    let comments = fields.get("comment")
        .and_then(|c| c.get("comments"))
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter().filter_map(|comment| {
                let author = comment.get("author")
                    .and_then(|a| {
                        a.get("displayName").and_then(|d| d.as_str())
                            .or_else(|| a.get("emailAddress").and_then(|e| e.as_str()))
                    })
                    .unwrap_or("Unknown")
                    .to_string();
                
                let created = comment.get("created")
                    .and_then(|c| c.as_str())
                    .unwrap_or("")
                    .to_string();
                
                let body = comment.get("body")
                    .and_then(|b| {
                        if b.is_string() {
                            b.as_str().map(|s| s.to_string())
                        } else {
                            extract_text_from_adf(b)
                        }
                    })
                    .unwrap_or_else(|| "".to_string());
                
                Some(Comment { author, created, body })
            }).collect()
        });
    
    Ok(Ticket {
        key,
        ticket_type: TicketType::from_str(&issue_type),
        summary,
        status,
        assignee,
        description,
        priority,
        reporter,
        created,
        updated,
        labels,
        comments,
    })
}

// Extract plain text from Atlassian Document Format
fn extract_text_from_adf(adf: &serde_json::Value) -> Option<String> {
    let mut text = String::new();
    
    if let Some(content) = adf.get("content").and_then(|c| c.as_array()) {
        for node in content {
            extract_node_text(node, &mut text);
        }
    }
    
    if text.is_empty() {
        None
    } else {
        Some(text.trim().to_string())
    }
}

fn extract_node_text(node: &serde_json::Value, text: &mut String) {
    if let Some(node_type) = node.get("type").and_then(|t| t.as_str()) {
        match node_type {
            "text" => {
                if let Some(t) = node.get("text").and_then(|t| t.as_str()) {
                    text.push_str(t);
                }
            }
            "paragraph" | "heading" | "blockquote" | "codeBlock" | 
            "bulletList" | "orderedList" | "listItem" | "panel" => {
                if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
                    for child in content {
                        extract_node_text(child, text);
                    }
                    text.push('\n');
                }
            }
            "hardBreak" => text.push('\n'),
            _ => {
                // Try to extract content from unknown nodes
                if let Some(content) = node.get("content").and_then(|c| c.as_array()) {
                    for child in content {
                        extract_node_text(child, text);
                    }
                }
            }
        }
    }
}