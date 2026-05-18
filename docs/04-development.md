# RepoLens — 开发计划

## 里程碑

### M1: 骨架 + 仓库发现（Day 1）

**目标：** `repolens scan` 能跑通，发现所有仓库并入库。

| 任务 | 预估 | 说明 |
|------|------|------|
| 项目初始化 | 0.5h | `cargo init`、依赖配置、目录结构 |
| config 模块 | 1h | TOML 配置读写、scan_roots 管理 |
| scanner 模块 | 2h | 递归扫描 `.git`、walkdir + ignore |
| db 模块 | 1.5h | SQLite 建表、CRUD、FTS5 虚拟表 |
| CLI scan 子命令 | 0.5h | clap 定义、串联以上模块 |
| 测试 | 1h | fixtures mock 仓库、集成测试 |

**合计：~6.5h**

---

### M2: 指纹提取 + 搜索（Day 2）

**目标：** `repolens find` 能搜到仓库，带指纹信息。

| 任务 | 预估 | 说明 |
|------|------|------|
| fingerprint 模块 | 3h | 语言检测、框架识别、README 提取 |
| search 模块 | 2h | FTS5 查询构建、过滤条件、排序 |
| CLI find/list 子命令 | 1h | 参数解析、输出格式化 |
| display 模块 | 1h | table 格式 + JSON 格式 |
| 测试 | 1h | 搜索准确性、过滤逻辑 |

**合计：~8h**

---

### M3: 详情 + 配置 + 收尾（Day 3）

**目标：** 所有 MVP 命令可用，整体打磨。

| 任务 | 预估 | 说明 |
|------|------|------|
| CLI show 子命令 | 1.5h | 仓库详情 + top files |
| CLI config 子命令 | 1h | add-path / remove-path / 查看 |
| git 信息读取 | 1h | git2 读取分支、最后提交 |
| 增量扫描优化 | 1h | mtime 检测、只更新变化的仓库 |
| 错误处理 | 1h | 友好错误信息、退出码 |
| 集成测试 + 修复 | 1.5h | 端到端测试、边界情况 |

**合计：~7h**

---

## 总计

| 里程碑 | 时间 | 核心产出 |
|--------|------|----------|
| M1 | Day 1 (6.5h) | scan 命令可用 |
| M2 | Day 2 (8h) | find / list 命令可用 |
| M3 | Day 3 (7h) | show / config 可用，MVP 完成 |
| **总计** | **3天 (21.5h)** | **MVP 交付** |

## V1 后续（可选）

| 任务 | 时间 | 版本 |
|------|------|------|
| 增量更新（mtime + hash） | 已含在 M3 | - |
| MCP Server | 1-2天 | V1 |
| 文件内容搜索 | 3-5天 | V2 |
| 语义搜索（embedding） | 3-5天 | V2 |
| Web UI | 5-10天 | V2 |

## 开发环境

```bash
# 前置要求
rustup stable
# SQLite 由 rusqlite bundled 自带，无需额外安装

# 开发
cd repolens
cargo build
cargo test

# 运行
cargo run -- scan
cargo run -- find "关键词"
```

## 验收标准

- [ ] `repolens scan` 能扫描配置目录下所有 git 仓库
- [ ] `repolens find "关键词"` 返回匹配的仓库
- [ ] `repolens list` 列出所有仓库，支持排序和过滤
- [ ] `repolens show <repo>` 展示仓库详情
- [ ] `repolens config --add-path` 管理扫描路径
- [ ] 所有命令 `--json` 输出有效 JSON
- [ ] Agent 通过 CLI 两步找到任意仓库
