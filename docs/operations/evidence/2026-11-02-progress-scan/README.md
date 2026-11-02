# 进度扫描与文档勘误（2026-11-02）

核验日期 2026-11-02（+08:00），执行者 Claude Code；rustc／cargo 1.95.0，macOS（Darwin 27.2.0）。

## 树身份

- 开始扫描时 main HEAD `a4483b64`（330／105／758／WS6），工作区有进行中的 `git merge c331-phase7`（MERGE_HEAD `6d36f47a`，340 路径已暂存、冲突已解）。
- 扫描期间另一会话于 21:16 提交合并 `f5e560db`；其 308 个非文档路径与本扫描测试时的暂存内容相同（`a4483b64..f5e560db` 非文档路径数 308，与扫描开始时暂存集合一致），因此下列结果适用于 `f5e560db` 的代码面。
- [00-tree-identity.txt](00-tree-identity.txt)：提交后记录的 HEAD、索引树 `89543e2b`（＝`f5e560db^{tree}`）与 `stash@{0}` 摘要。

## 命令与结果

| 文件 | 命令（在 `noname-next/` 根目录） | 结果 |
| --- | --- | --- |
| [01](01-source-lines-unit.log) | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | OK，exit 0 |
| [02](02-source-lines.log) | `python3 -B scripts/check-source-lines.py` | 9075 文件通过，exit 0 |
| [03](03-three-crates.log) | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native --no-fail-fast` | 1903 passed／0 failed／75 ignored，exit 0 |
| [04](04-workspace.log) | `cargo test --offline --workspace --no-fail-fast` | 173 个 test result 段（含 doc-test）合计 7154 passed／0 failed／77 ignored，exit 0 |
| [05](05-preflight.log) | `python3 -B scripts/preflight.py` | 全部通过，exit 0（运行时已包含本次 file:line 引用修正） |

未跑：前端 `pnpm test`／typecheck、`pnpm build:native`、Worker smoke、跨目标脚本、`cargo test -p desktop-client`、浏览器 7 案例、legacy WASM 重建、真机与 CI。

## 发现与处理

1. 进度主稿 C331 C9L 条目同时写“未合入”和“已合并提交合入 Main”，扫描开始时实际未提交 → 改为简体并按提交后的事实重写合入状态。
2. C331 把 `NETWORK_PROTOCOL_VERSION` 6→7（`ChoiceRequest` 计数 u32），裁决 22／29 要求保持 WS6 或另报边界、不得借用 R05 的 WS7，未见报告与授权 → 登记为待决，不改代码。
3. 历史证据 `2026-10-18-nc08-upstream-kuwu` 引用的 `damage.rs:988-1015` 在 C331 拆分后越界，preflight 的 file:line 检查会失败 → 按既有拆分先例改指 `damage_flow.rs:99-131` 并注明原位置。
4. README、架构索引仍把原生内核写成单一 `crates/engine-native`，常用测试命令只测门面 → 改为三 crate 及 F7 分层说明，阅读入口补剩余台账。
5. 剩余台账标题“当前阻断”下 B1–B9 早已全部关闭 → 改为历史阻断并保留原锚点；页首导航更新到 `f5e560db`。
6. 执行提示词仍要求入库九批、修 B9、做 F7 第 3–4 步（均已完成） → 按当前状态重写。
