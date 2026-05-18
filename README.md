# 🔍 RepoLens

> Local git repository search engine — find your projects instantly.

**RepoLens** is a blazing-fast, agent-friendly CLI tool that indexes your local git repositories and lets you (or your AI coding assistant) find any project in a single query.

No more "where did I put that project?" — RepoLens scans your directories, extracts rich metadata (languages, frameworks, README summaries, git activity), and serves it through a simple CLI with full JSON output support.

## ✨ Features

- **🔍 Repo-level search** — Find repos by name, README content, framework, or path
- **📊 Rich fingerprints** — Auto-detects languages, frameworks, git branch/last commit
- **🤖 Agent-native** — `--json` flag on every command, designed for AI assistant integration
- **⚡ Blazing fast** — SQLite FTS5 full-text index, incremental scanning
- **🧩 Zero config** — Point at your projects directory and go
- **🏗️ Single binary** — Pure Rust, no runtime dependencies

## 🚀 Quick Start

```bash
# Install
cargo install --path .

# Add your project directories
repolens config --add-path ~/projects
repolens config --add-path ~/work

# Scan and index
repolens scan

# Search
repolens find "enterprise wechat"
repolens find "api" --lang rust --recent 30d

# List all repos
repolens list
repolens list --lang python --sort recent

# Show repo details
repolens show my-project

# JSON output (for scripts and AI agents)
repolens find "bot" --json
repolens show my-project --json
```

## 📖 Commands

| Command | Alias | Description |
|---------|-------|-------------|
| `repolens scan` | `s` | Scan configured directories and index repos |
| `repolens find <query>` | `f` | Search repos by keyword |
| `repolens list` | `ls` | List all indexed repos |
| `repolens show <repo>` | `sh` | Show detailed repo info |
| `repolens config` | — | Manage scan paths and settings |

### Search Filters

```bash
repolens find "api" --lang rust           # Filter by language
repolens find "bot" --framework fastapi    # Filter by framework
repolens find "web" --recent 7d           # Recently active (7d / 1m / 1y)
repolens find "service" --size +100m      # Size filter (+/- prefix)
repolens find "tool" --path ~/projects    # Limit to directory
repolens find "lib" --limit 5             # Limit results
```

## 🤖 AI Agent Integration

RepoLens is designed for seamless AI agent workflows:

```bash
# Agent finds a project in one interaction
$ repolens find "wechat bot" --json
{"results": [{"name": "wechat-bot", "path": "/home/user/projects/wechat-bot", 
              "languages": ["Python"], "frameworks": ["FastAPI"]}], "total": 1}

# Agent gets full context
$ repolens show wechat-bot --json
{"name": "wechat-bot", "branch": "main", "last_commit": "2026-04-20", ...}
```

**Two commands, one conversation — your agent knows exactly where the project is.**

## 🏗️ Architecture

```
┌─────────────────────────────────┐
│         CLI (clap)              │
└──────────────┬──────────────────┘
               ▼
┌─────────────────────────────────┐
│  Scanner → Fingerprint → DB    │
│  (walkdir)  (lang/fw)  (SQLite) │
└──────────────┬──────────────────┘
               ▼
┌─────────────────────────────────┐
│  FTS5 Search → Display          │
│  (full-text)  (table / json)    │
└─────────────────────────────────┘
```

## 🛠️ Building

```bash
# Prerequisites: Rust toolchain (rustup.rs)
git clone https://github.com/ZhangXinZhuang/repolens.git
cd repolens
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

## 📁 Configuration

Config: `~/.config/repolens/config.toml`
Data: `~/.local/share/repolens/repolens.db`

```bash
# Manage scan paths
repolens config --add-path ~/projects
repolens config --remove-path ~/old-projects
repolens config                  # View current config
```

## 📋 Roadmap

- [x] **MVP** — CLI with scan, find, list, show
- [x] **Rich fingerprints** — Language detection, framework identification
- [x] **Agent-friendly** — JSON output on all commands
- [ ] **Incremental updates** — mtime-based delta scanning
- [ ] **MCP Server** — Model Context Protocol integration
- [ ] **Content search** — Search inside repo files
- [ ] **Semantic search** — Embedding-based retrieval

## License

MIT
