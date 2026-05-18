# RepoLens — 技术架构

## 整体架构

```
┌──────────────────────────────────────┐
│           CLI (clap)                  │
│  scan / find / list / show / config   │
└──────────────┬───────────────────────┘
               │
               ▼
┌──────────────────────────────────────┐
│           Core Library                │
│                                       │
│  ┌───────────┐  ┌──────────────────┐  │
│  │ Scanner   │  │ Search Engine    │  │
│  │ (仓库发现) │  │ (SQLite FTS5)    │  │
│  └─────┬─────┘  └────────┬─────────┘  │
│        │                  │            │
│  ┌─────▼──────────────────▼─────────┐  │
│  │        Fingerprint Engine         │  │
│  │  README / 语言 / 框架 / Git信息   │  │
│  └──────────────┬───────────────────┘  │
│                 │                       │
│  ┌──────────────▼───────────────────┐  │
│  │         Storage (SQLite)          │  │
│  │  repos 表 + FTS5 虚拟表          │  │
│  └──────────────────────────────────┘  │
└──────────────────────────────────────┘
```

## 数据模型

### SQLite Schema

```sql
-- 仓库元数据
CREATE TABLE repos (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,           -- 仓库名（目录名）
    path        TEXT NOT NULL UNIQUE,    -- 绝对路径
    parent_path TEXT,                     -- 父目录
    languages   TEXT,                     -- JSON: ["Python", "YAML"]
    frameworks  TEXT,                     -- JSON: ["FastAPI"]
    branch      TEXT,                     -- 当前分支
    last_commit_hash   TEXT,
    last_commit_date   TEXT,              -- ISO 8601
    last_commit_msg    TEXT,
    file_count  INTEGER DEFAULT 0,
    size_bytes  INTEGER DEFAULT 0,
    readme_hash TEXT,                     -- README 内容 hash，用于变更检测
    readme_summary TEXT,                  -- README 前 500 字
    scanned_at  TEXT NOT NULL,            -- 最后扫描时间
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

-- FTS5 全文索引（搜索用）
CREATE VIRTUAL TABLE repos_fts USING fts5(
    name,
    path,
    readme_summary,
    frameworks,
    languages,
    content=repos,
    content_rowid=id
);

-- 扫描路径配置
CREATE TABLE scan_roots (
    id   INTEGER PRIMARY KEY AUTOINCREMENT,
    path TEXT NOT NULL UNIQUE
);
```

## 指纹提取

### 语言检测

按文件扩展名统计，取 Top 3：

```rust
fn detect_languages(path: &Path) -> Vec<LanguageInfo> {
    // 遍历文件（排除 .gitignore 规则）
    // 按扩展名分组计数
    // 映射扩展名 → 语言名
    // 返回按文件数排序的语言列表
}
```

扩展名映射（核心子集）：
- `.rs` → Rust
- `.py` → Python
- `.ts/.tsx` → TypeScript
- `.js/.jsx` → JavaScript
- `.go` → Go
- `.java` → Java
- `.md` → Markdown
- `.toml/.yaml/.json` → Config

### 框架识别

通过特征文件检测：

| 框架 | 特征文件 | 特征内容 |
|------|----------|----------|
| FastAPI | `requirements.txt` / `pyproject.toml` | 包含 `fastapi` |
| Django | `manage.py` | - |
| React | `package.json` | 包含 `react` |
| Tauri | `Cargo.toml` + `src-tauri/` | 包含 `tauri` |
| Axum | `Cargo.toml` | 包含 `axum` |
| Next.js | `package.json` | 包含 `next` |

### README 提取

```
1. 查找 README.md / README / README.txt / readme.md
2. 提取前 500 字符作为摘要
3. 计算 hash 用于变更检测
```

## 核心流程

### 扫描流程

```
scan(root_paths)
  ├── 遍历配置的所有根目录
  ├── 递归查找 .git 目录 → 识别仓库
  ├── 对每个仓库：
  │   ├── 检查 mtime / readme_hash → 是否需要更新
  │   ├── 如果需要：
  │   │   ├── 提取指纹（语言、框架、README）
  │   │   ├── 读取 Git 信息（分支、最后提交）
  │   │   ├── 计算 size / file_count
  │   │   └── UPSERT 到 SQLite
  │   └── 如果不需要：跳过
  ├── 检查已索引但目录不存在的 → 标记删除
  └── 返回扫描统计
```

### 搜索流程

```
find(query, filters)
  ├── 构建 FTS5 查询
  │   ├── query → FTS5 MATCH
  │   ├── --lang → JOIN / WHERE
  │   ├── --recent → WHERE last_commit_date >
  │   └── --path → WHERE path LIKE
  ├── 执行查询
  ├── 按相关度 + 活跃度排序
  └── 格式化输出（table / json）
```

## 项目结构

```
repolens/
├── Cargo.toml
├── docs/                   # 设计文档
├── src/
│   ├── main.rs             # CLI 入口，clap 定义
│   ├── config.rs           # 配置管理（scan_roots, settings）
│   ├── scanner.rs          # 仓库发现 + 遍历
│   ├── fingerprint.rs      # 指纹提取（语言、框架、README）
│   ├── db.rs               # SQLite 操作（建表、CRUD）
│   ├── search.rs           # FTS5 搜索逻辑
│   ├── display.rs          # 输出格式化（table / json）
│   └── models.rs           # 数据结构定义
└── tests/
    ├── integration_test.rs
    └── fixtures/            # 测试用 mock 仓库
```

## 核心依赖

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
rusqlite = { version = "0.31", features = ["bundled"] }
git2 = "0.19"
walkdir = "2"
ignore = "0.4"           # 尊重 .gitignore
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = "0.4"
sha2 = "0.10"            # README hash
toml = "0.8"             # 配置文件解析
```

## 配置

位置：`~/.config/repolens/config.toml`

```toml
[scan]
roots = []               # 通过 repolens config --add-path 添加
exclude_dirs = ["node_modules", "target", ".git", "build", "dist"]
interval_min = 5         # 自动扫描间隔（分钟，0 = 不自动）
follow_symlinks = false

[display]
default_sort = "recent"  # recent | name | size
max_results = 20
date_format = "%Y-%m-%d"
```

## 数据位置

```
~/.local/share/repolens/
├── repolens.db           # SQLite 数据库
└── config.toml → ~/.config/repolens/config.toml
```
