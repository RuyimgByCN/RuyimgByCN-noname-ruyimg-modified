# 原生收敛剩余工作台帐（2026-10-24）

新增 `docs/architecture/native-remaining-ledger.md`，把 NC01–NC12、R01–R10、N01–N07、I01–I05 与 37 来源统一列成状态/剩余/依赖/证据表；从进度主稿链接。

本批不改变规则、存档或内容；只收敛待办与完成条件。状态依据见 [台帐](../../../architecture/native-remaining-ledger.md)。

## 2026-10-24 追加回归

- `game-server` 完整：**330 / 0 / 0**（23 target）；见 [game-server.log](game-server.log)、[计数](game-server-counts.json)、[退出码](game-server.exit)。
- 目的：在 R03 目标限制/计数字段与编译/加载门禁之后，确认服务端房间、持久化、重连与正式产品路径未回归。

## 2026-10-24 追加：命令收据有界

- `CommandReceipts` 增加 `MAX_COMMAND_RECEIPTS=4096`；每次记入/恢复后按 `new_revision` 淘汰最旧收据，防止长局高频重连无界增长。
- 新增 `receipts_are_bounded_and_keep_newest_revisions`：插入 4099 条后长度恰为 4096，最旧 key 不可重放，最新 key 可重放。
- 新增 `restore_prunes_receipts_above_cap`：恢复时读入 4098 条旧收据也会收敛到 4096，并保留最新修订。
- `game-server` 完整 **336 / 0 / 0**（24 target），见 [日志](game-server-receipts.log)。
- 这是 NC12 长局内存增长的必要修正，不等于 WAL/fsync/多房间负载已验收。

## 2026-10-24 追加：16 房间保存/装载成本基线

- 新增 `multi_room_save_and_load_cost_is_measured_and_bounded`：16 房间 × 20 轮全量保存 + 一次 `load_all`。
- 实测（本机同工具链，两次采样）：中位 **29.0ms**、p95 **58.8ms**（首次采样 p95 131.2ms）；16 房间一次装载 **11.5ms**；测试上限 500ms。
- 结论：优化前每条接受命令的 durable save（journal prepared/committed + 文件 + 目录同步）在本机中位约 29ms、p95 约 59ms，**可能成为多房间写放大瓶颈**。
- **2026-10-25 优化**：已有旧 journal 时，prepared 的目录 fsync 可跳过——文件内容已 sync，rename 未持久化则崩溃后回退旧 journal，后续 committed 再同步目录；首次写入、committed 与故障注入保持目录同步。同一测试、同一机器、同一负载复测：**中位 24.85ms、p95 49.18ms**（约 −14% / −16%），16 房间 load 9.92ms。
- **2026-10-25 二次优化**：状态文件 rename 后的目录 fsync 也省略——journal prepared 已保存撤销依据，committed 写入会再同步目录；崩溃时以 journal 为准。同测试复测：**中位 19.15ms、p95 47.90ms**（相对原始 29/58.8 约 −34% / −19%），load 9.32ms。
- **2026-10-25 三次优化**：状态临时文件的内容 fsync 省略——状态文件不是恢复权威，committed journal 保存完整 JSON，崩溃时按 journal 恢复；故障注入 `FileSync` 仍触发。同测试复测：**中位 15.54ms、p95 35.90ms**（相对原始约 −46% / −39%），load 10.03ms。`nc09_native_durability` 45/45、`native_room_store` 28/28、`game-server` 357/0/0 通过。WAL/批提交仍是待办。

## 2026-10-25 全 workspace 回归

- `cargo test --offline --workspace --no-fail-fast`：**601 targets / 6566 passed / 0 failed / 72 ignored**，exit 0；日志 [workspace.log](workspace.log)、[退出码](workspace.exit)。
- 覆盖 protocol replay 字段、game-server 耐久/actor、engine-native/core、content/runtime/loader/compiler、WASM 等全部 workspace crate；前端仍 **76/0**、tsc 0。

## 2026-10-25 保护核对与强制门禁

- 外部备份 `.worktree-backups/noname-next-2026-10-24-round/` 存在，**5.0 GiB**；`worktree/Cargo.toml` 与工作区 `Cargo.toml` SHA-256 同为 `8d5dab37…f360`。
- `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`：1 test OK。
- `python3 -B scripts/check-source-lines.py`：8600 档 ≤2000 行。
- `node scripts/sync-original-content.mjs --verify-only`：exit 0（legacy-source 366 / portraits 2707）。
- 未执行 git add/commit/push/reset 或破坏性清理。
