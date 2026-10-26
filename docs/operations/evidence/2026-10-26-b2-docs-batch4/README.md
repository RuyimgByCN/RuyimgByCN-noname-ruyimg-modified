# B2 批④：10-24 文档批入库 + 运行数据目录忽略（2026-10-26）

- 执行时间：2026-10-26 **21:45 → 21:49 +08:00**；`HEAD = da8c7ff0`（领取时实值）；执行者 ZCode A 道
- 工作包：[台帐 B2](../../../architecture/native-remaining-ledger.md#当前阻断2026-10-26-扫描按优先级) 第四批（最后一批）

## 1 内容与核对

| 文件 | 改动 | 与登记的对应 |
|---|---|---|
| `docs/architecture/native-progress-history.md` +68 | 新增 `delivery-2026-10-24-history` 锚与「2026-10-24 交付流水（17:00 时点移存）」 | [架构审计证据](../2026-10-24-architecture-doc-audit/README.md) 第 86/98 行：「10/24 多轮完整摘要移到既有历史页，原文保留」；已提交的进度主稿正文已链向该锚（此前干净 HEAD 上该锚不存在） |
| `docs/architecture/overview.md`、`architecture-evolution-plan.md`、`README.md`、`docs/operations/self-check-guide.md` | 导航重排、锚点、自检表改为 `check_local_links.py --all` 与主稿口径 | 同上审计证据的文档调整项 |
| `docs/adr/INDEX.md` +2 | 补 ADR 0784（PA09 草案）/0785（PA11 草案）索引行 | 两份 ADR 已在 HEAD（PA09/PA11 批提交），索引行漏提交 |
| `evidence/2026-10-07-n08-cardlist-parity/README.md`、`2026-10-18-blocked-fixes/README.md` | 把旧工作区行号固定为 GitHub 永久链接 / 快照行号注记 | 审计证据「现有链接检查只验证路径…本轮另核对改动文档的锚点」 |
| 证据目录 `2026-10-24-architecture-doc-audit`（184K）、`2026-10-24-native-remaining-ledger`（2.1M）、`2026-10-25-native-arch-perf-review`（44K） | 整体未入库的证据，被台帐/进度稿多处链接 | 入库后干净 HEAD 链接可解析 |
| `.gitignore` +`/data/` | 服务默认 `data/{server-content,native-content,native-rooms}`（`apps/game-server/src/config.rs`）在仓库根运行时生成，本机 895 MB 运行存储 | 运行数据不入库；与批② 的 `apps/game-server/data/` 同理 |

## 2 验证

`python3 -B scripts/check_local_links.py --all` 1809 文件坏链 0；`check-adr-index.py` 通过；`check-source-lines.py` 8612 文件通过。本批无代码/规则/存档改动；前端 test/typecheck 以批③（21:30，80/80、0 错）为准，其后无前端改动。

## 3 B2 收口

四批（① e6ba869a、② a8cd662d、③ da8c7ff0、④ 本提交）后 `git status --short` 为空（`/data/` 与 `apps/game-server/data/` 被忽略）。遗留登记见台帐 B2 行：`sync-original-content.mjs --verify-only` 区域设置依赖；被跟踪 `apps/web-client/dist/` 为 10-11 遗留且被 .gitignore 覆盖。
