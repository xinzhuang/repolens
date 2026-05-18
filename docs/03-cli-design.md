# RepoLens — CLI 命令设计

## 设计原则

1. **默认人类可读，`-j/--json` 切机器模式**
2. **子命令风格**（像 `git` / `gh`）— 每个动作语义清晰
3. **组合式过滤** — flag 自由组合，不用记太多子命令
4. **极短命令别名** — 高频操作 3 个字母搞定

## 命令总览

```
repolens <command> [options]

Commands:
  scan    (s)    扫描并索引仓库
  find    (f)    模糊搜索仓库
  list    (ls)   列出仓库
  show    (sh)   展示仓库详情
  config         配置管理

Global Options:
  -j, --json     JSON 输出（Agent 用）
  -q, --quiet    静默模式，只输出数据
  -h, --help     帮助
  -V, --version  版本
```

---

## `repolens scan` / `repolens s`

扫描配置的根目录，发现并索引仓库。

### 用法

```bash
# 首次扫描 / 增量更新
repolens scan

# 强制全量重建
repolens scan --rebuild

# 添加新的扫描路径（同时扫描）
repolens scan --add-path ~/projects

# 静默模式（适合 cron / Agent 调用）
repolens scan --quiet
```

### 选项

| Flag | 说明 |
|------|------|
| `--rebuild` | 强制全量重建索引 |
| `--add-path <PATH>` | 添加扫描路径并立即扫描 |
| `-q, --quiet` | 静默模式，只输出统计 JSON |

### 输出

人类可读：
```
✓ Scanned 3 root directories
  87 repos found (12 new, 3 removed, 72 unchanged)
  Indexed in 2.3s
```

JSON (`--json` / `--quiet`)：
```json
{
  "roots_scanned": 3,
  "repos_total": 87,
  "repos_new": 12,
  "repos_removed": 3,
  "repos_unchanged": 72,
  "duration_ms": 2300
}
```

---

## `repolens find` / `repolens f`

模糊搜索仓库。匹配仓库名 + README + 路径 + 框架。

### 用法

```bash
# 模糊搜索
repolens find "企业微信"
repolens find "openhuman"
repolens find "bot"

# 按语言过滤
repolens find --lang rust
repolens find --lang python --lang typescript   # 多选

# 按框架过滤
repolens find --framework fastapi

# 按时间过滤
repolens find --recent 7d        # 7天内活跃
repolens find --recent 1m        # 1个月内活跃

# 按大小过滤
repolens find --size +100m       # 大于 100MB
repolens find --size -10m        # 小于 10MB

# 限定目录
repolens find --path ~/projects

# 限制结果数
repolens find "api" --limit 5

# 组合
repolens find "api" --lang rust --recent 30d --limit 5
```

### 选项

| Flag | 说明 |
|------|------|
| `--lang <LANG>` | 按语言过滤（可多次使用） |
| `--framework <FW>` | 按框架过滤（可多次使用） |
| `--recent <DURATION>` | 最近活跃，如 `7d` / `1m` / `1y` |
| `--size <SIZE>` | 按大小过滤，如 `+100m` / `-10m` |
| `--path <PATH>` | 限定目录范围 |
| `--limit <N>` | 限制结果数（默认 20） |
| `-j, --json` | JSON 输出 |

### 输出

人类可读：
```
REPO                      LANG         LAST ACTIVE    SIZE   README
bots/wechat-bot           Python       2026-04-20     12MB   企业微信消息机器人...
frank/wechat-api          TypeScript   2026-03-10     3MB    企业微信API封装...

(2 results)
```

JSON：
```json
{
  "results": [
    {
      "name": "wechat-bot",
      "path": "/home/frank/projects/bots/wechat-bot",
      "languages": ["Python"],
      "frameworks": ["FastAPI"],
      "last_commit": "2026-04-20",
      "size_mb": 12.3,
      "file_count": 89,
      "readme_summary": "企业微信消息机器人，支持关键词回复、定时推送..."
    }
  ],
  "total": 2
}
```

---

## `repolens list` / `repolens ls`

列出所有仓库。

### 用法

```bash
# 列出全部
repolens list

# 按语言过滤
repolens list --lang rust

# 按语言分组显示
repolens list --group-by lang

# 排序
repolens list --sort recent     # 最近活跃（默认）
repolens list --sort name       # 按名称
repolens list --sort size       # 按大小
```

### 选项

| Flag | 说明 |
|------|------|
| `--lang <LANG>` | 过滤语言 |
| `--framework <FW>` | 过滤框架 |
| `--sort <FIELD>` | 排序：recent / name / size |
| `--group-by <FIELD>` | 分组：lang / path |
| `--limit <N>` | 限制数量 |
| `-j, --json` | JSON 输出 |

### 输出

人类可读（默认按 recent 排序）：
```
REPO                              LANG         LAST ACTIVE    SIZE
frank/openhuman-fork              Rust         2026-05-17     340MB
frank/xinzhuang-skills            TypeScript   2026-05-16     8MB
bots/wechat-bot                   Python       2026-04-20     12MB
old/legacy-service                Java         2025-11-03     45MB
...
(87 repos)
```

分组模式（`--group-by lang`）：
```
Python (12 repos):
  bots/wechat-bot         2026-04-20    12MB
  tools/data-pipeline     2026-03-15    5MB
  ...

Rust (8 repos):
  frank/openhuman-fork    2026-05-17    340MB
  ...
```

---

## `repolens show` / `repolens sh`

展示单个仓库的详细信息。

### 用法

```bash
# 按名称
repolens show wechat-bot

# 按路径
repolens show /home/frank/projects/bots/wechat-bot

# 模糊匹配
repolens show "openhuman"
```

### 输出

人类可读：
```
📦 wechat-bot
   Path:      /home/frank/projects/bots/wechat-bot
   Languages: Python (82%), YAML (12%), Markdown (6%)
   Framework: FastAPI
   Branch:    main
   Last commit: 2026-04-20 "fix: timeout handling"
   Size:      12.3 MB (89 files)

   README:
   企业微信消息机器人，支持关键词回复、定时推送、Webhook集成...
   (truncated, 500 chars)

   Top files:
   src/main.py          2.1KB   2026-04-20
   src/handlers.py      4.8KB   2026-04-18
   requirements.txt     0.3KB   2026-03-15
```

JSON：
```json
{
  "name": "wechat-bot",
  "path": "/home/frank/projects/bots/wechat-bot",
  "languages": {"Python": 82, "YAML": 12, "Markdown": 6},
  "frameworks": ["FastAPI"],
  "branch": "main",
  "last_commit": {
    "hash": "abc1234",
    "date": "2026-04-20",
    "message": "fix: timeout handling"
  },
  "size_bytes": 12897430,
  "file_count": 89,
  "readme_summary": "企业微信消息机器人，支持关键词回复、定时推送、Webhook集成...",
  "top_files": [
    {"path": "src/main.py", "size": 2148, "modified": "2026-04-20"},
    {"path": "src/handlers.py", "size": 4915, "modified": "2026-04-18"}
  ]
}
```

---

## `repolens config`

配置管理。

### 用法

```bash
# 查看当前配置
repolens config

# 添加扫描路径
repolens config --add-path ~/projects

# 移除扫描路径
repolens config --remove-path ~/old-projects

# 设置默认排序
repolens config --set default_sort=name
```

---

## Agent 调用模式

### 典型流程：找项目

```bash
# Step 1: 搜索
$ repolens find "企业微信" --json
{"results": [{"name": "wechat-bot", "path": "/home/frank/..."}], "total": 1}

# Step 2: 详情（如需要）
$ repolens show wechat-bot --json
{"name": "wechat-bot", "languages": {"Python": 82}, ...}
```

### 典型流程：列项目

```bash
# 按语言列
$ repolens list --lang rust --json
{"results": [...], "total": 8}

# 最近活跃的
$ repolens list --sort recent --limit 5 --json
{"results": [...], "total": 5}
```

### 退出码

| Code | 含义 |
|------|------|
| 0 | 成功，有结果 |
| 1 | 成功，无结果 |
| 2 | 参数错误 |
| 3 | 数据库错误 |
| 4 | 配置错误 |

无结果时退出码为 1（而非 0），方便 Agent 判断 `if ! repolens find ...`。
