use crate::config::Config;
use crate::jira_api;
use std::error::Error;

pub fn fetch_tickets(config: &Config) -> Result<Vec<crate::model::Ticket>, Box<dyn Error>> {
    jira_api::fetch_tickets_api(config)
}