# 🦀 KANBARS

Lightning-fast terminal kanban board for JIRA. No more 300MB browser tabs.

<img width="1516" height="788" alt="wee" src="https://github.com/user-attachments/assets/18de15c5-c420-49d1-b5e3-aeec7b79790e" />

## Features

- **Horizontal swim lanes** - only shows lanes with tickets
- **Smart text wrapping** - maximizes use of terminal width  
- **Type indicators** - 🐛 Bug | 📖 Story | ✓ Task | 🎯 Epic
- **Instant** - no loading spinners, just your tickets

## Quick Start

```bash
# Install (simplest method)
cargo install kanbars

# Set credentials (same as ACLI)
export JIRA_SITE="yourcompany.atlassian.net"
export JIRA_USER="your.email@company.com"
export JIRA_API_TOKEN="your-api-token"  # Get from https://id.atlassian.com/manage/api-tokens

# Run
kanbars
```

### Alternative Installation

```bash
# From source
git clone https://github.com/yourusername/kanbars.git
cd kanbars
cargo install --path .
```

## Usage

```bash
kanbars                                           # Your tickets (developer = currentUser())
kanbars --jql "assignee = currentUser()"         # Only assigned to you
kanbars --jql "sprint in openSprints()"          # Current sprint
kanbars --assignee "teammate@company.com"        # Someone else's tickets
kanbars --init                                   # Create config file
```

Press `q` to quit.

## Default Query

Shows tickets where you are the **Developer** (not just assignee):
```
developer = currentUser() AND status NOT IN (Done, Shipped, Discontinued, Closed, Hibernate)
```

Perfect for teams where tickets get reassigned to QA/Product during review.

## Config File (Optional)

Create `~/.config/kanbars/config.toml` to avoid environment variables:
```toml
[jira]
url = "https://yourcompany.atlassian.net"
email = "your.email@company.com"
api_token = "your-api-token"

[query]
jql = "your custom default query"
```

## License

MIT
