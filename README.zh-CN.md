# 🔍 RepoLens

> 本地 Git 仓库搜索引擎 — 瞬间找到你的项目。

**RepoLens** 是一个极速的、对 AI Agent 友好的 CLI 工具，用于索引本地 Git 仓库，让你（或 AI 编程助手）通过一条命令找到任何项目。

不再"那个项目放哪儿了？" — RepoLens 扫描你的目录，提取丰富元数据（语言、框架、README 摘要、Git 活跃度），通过简洁的 CLI 提供 JSON 输出支持。

## ✨ 特性

- **🔍 仓库级搜索** — 按名称、README 内容、框架或路径查找仓库
- **📊 丰富指纹** — 自动检测语言、框架、Git 分支/最近提交
- **🤖 Agent 原生** — 所有命令支持 `--json`，专为 AI 助手集成设计
- **⚡ 极速** — SQLite FTS5 全文索引，增量扫描
- **🧩 零配置** — 指向项目目录即可开始
- **🏗️ 单二进制** — 纯 Rust，无运行时依赖

## 🚀 快速开始

### 安装

```bash
# macOS / Linux — 一键安装
curl -fsSL https://raw.githubusercontent.com/xinzhuang/repolens/main/scripts/install.sh | bash
```

<details>
<summary>其他安装方式</summary>

**Homebrew：**
```bash
brew tap xinzhuang/tap
brew install repolens
```

**从源码构建：**
```bash
git clone https://github.com/xinzhuang/repolens.git
cd repolens
cargo install --path .
```

也可以从 [GitHub Releases](https://github.com/xinzhuang/repolens/releases) 直接下载预编译二进制文件。
</details>

### 使用

```bash
# 添加项目目录
repolens config --add-path ~/projects
repolens config --add-path ~/work

# 扫描并索引
repolens scan

# 搜索
repolens find "企业微信"
repolens find "api" --lang rust --recent 30d

# 列出所有仓库
repolens list
repolens list --lang python --sort recent

# 查看仓库详情
repolens show my-project

# JSON 输出（适合脚本和 AI Agent）
repolens find "bot" --json
repolens show my-project --json
```

## 📖 命令

| 命令 | 别名 | 说明 |
|---------|-------|-------------|
| `repolens scan` | `s` | 扫描配置目录并索引仓库 |
| `repolens find <query>` | `f` | 按关键词搜索仓库 |
| `repolens list` | `ls` | 列出所有已索引仓库 |
| `repolens show <repo>` | `sh` | 显示仓库详细信息 |
| `repolens config` | — | 管理扫描路径和设置 |

### 搜索过滤

```bash
repolens find "api" --lang rust           # 按语言过滤
repolens find "bot" --framework fastapi    # 按框架过滤
repolens find "web" --recent 7d           # 最近活跃（7d / 1m / 1y）
repolens find "service" --size +100m      # 大小过滤（+/- 前缀）
repolens find "tool" --path ~/projects    # 限定目录
repolens find "lib" --limit 5             # 限制结果数量
```

## 🤖 AI Agent 集成

RepoLens 专为 AI Agent 工作流设计：

```bash
# Agent 一次交互找到项目
$ repolens find "wechat bot" --json
{"results": [{"name": "wechat-bot", "path": "/home/user/projects/wechat-bot", 
              "languages": ["Python"], "frameworks": ["FastAPI"]}], "total": 1}

# Agent 获取完整上下文
$ repolens show wechat-bot --json
{"name": "wechat-bot", "branch": "main", "last_commit": "2026-04-20", ...}
```

**两条命令，一次对话 — Agent 精准定位项目。**

## 🏗️ 架构

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

## 📁 配置

配置文件：`~/.config/repolens/config.toml`
数据文件：`~/.local/share/repolens/repolens.db`

```bash
# 管理扫描路径
repolens config --add-path ~/projects
repolens config --remove-path ~/old-projects
repolens config                  # 查看当前配置
```

## 📋 路线图

- [x] **MVP** — CLI：scan、find、list、show
- [x] **丰富指纹** — 语言检测、框架识别
- [x] **Agent 友好** — 所有命令支持 JSON 输出
- [ ] **增量更新** — 基于 mtime 的增量扫描
- [ ] **MCP Server** — Model Context Protocol 集成
- [ ] **内容搜索** — 搜索仓库内的文件
- [ ] **语义搜索** — 基于嵌入的检索

## 📄 许可证

MIT
