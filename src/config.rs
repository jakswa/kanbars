use dirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub jira: JiraConfig,
    pub query: QueryConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraConfig {
    pub url: Option<String>,
    pub email: Option<String>,
    pub api_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryConfig {
    pub jql: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            jira: JiraConfig {
                url: None,
                email: None,
                api_token: None,
            },
            query: QueryConfig {
                jql: "developer = currentUser() AND status NOT IN ('Done', 'Shipped', 'Discontinued', 'Closed', 'Hibernate')".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::config_path();
        
        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)
                .unwrap_or_else(|_| String::new());
            toml::from_str(&contents).unwrap_or_else(|_| Self::default())
        } else {
            // Check environment variables as fallback
            let mut config = Self::default();
            
            // Support both JIRA_URL and JIRA_SITE (ACLI style)
            if let Ok(url) = std::env::var("JIRA_URL") {
                config.jira.url = Some(url);
            } else if let Ok(site) = std::env::var("JIRA_SITE") {
                // Convert site to full URL if it's just the domain
                let url = if site.starts_with("http://") || site.starts_with("https://") {
                    site
                } else {
                    format!("https://{}", site)
                };
                config.jira.url = Some(url);
            }
            
            if let Ok(user) = std::env::var("JIRA_USER") {
                config.jira.email = Some(user);
            } else if let Ok(email) = std::env::var("JIRA_EMAIL") {
                config.jira.email = Some(email);
            }
            
            if let Ok(token) = std::env::var("JIRA_API_TOKEN") {
                config.jira.api_token = Some(token);
            }
            
            config
        }
    }
    
    pub fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .expect("Could not find config directory")
            .join("kanbars");
        
        // Create directory if it doesn't exist
        let _ = fs::create_dir_all(&config_dir);
        
        config_dir.join("config.toml")
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path();
        let toml_string = toml::to_string_pretty(self)?;
        fs::write(config_path, toml_string)?;
        Ok(())
    }
}