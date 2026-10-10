# 2026-10-07 维护批次核验（O1 依赖修正 + C3 生命周期场景 + P3 deny 接线）

本批次完成主稿剩余可闭环项：O1 低风险依赖修正、P1/E02·C3 生命周期跨层
正反例、P3 尾项（lib `-D warnings` 防线接入 CI）。E03 monkey 规则宿主切片
经评估转待做（见「结论边界」）。上一批：[未完成任务批次](../2026-10-07-unfinished-batch/README.md)。

## 检查与结果

| 检查 | 结果 |
|---|---|
| O1 依赖修正 | `engine-core`：`tempfile` 从普通依赖移至开发依赖（src/ 零使用，仅 tests 经 `content_runtime` 别名使用）；删除 dev-dependencies 中与普通依赖重复的 `content-compiler`/`content-schema` 声明 |
| O1 验证 | `cargo tree -p engine-core --no-dev-dependencies` 与 `-p engine-wasm --no-dev-dependencies` 均 0 命中 tempfile；engine-core 全量 **3777/0**（含直接受影响的 `content_lock_extensions`、`real_extensions_lock`）；Cargo.lock 随检查更新。wasm32 目标本机未安装，真实 wasm32 构建留待目标环境（依赖闭包层面已验证） |
| C3 生命周期场景 | 新增 `e02_c3_lifecycle`（自构造 ADR 0778 自描述包，v1/v2 仅 pack.version 不同）：① v1 建局保存（内容锁绑定）→ ② 升级 v2 后用 v2 集合恢复原快照 → `RuleVersionMismatch` 明确拒绝 → ③ `rollback` 回 v1（`load_active_package_bytes` 可重取 v1 npack）→ 原快照恢复成功并继续对局 |
| P3 尾项 | CI `.github/workflows/ci.yml` 新增 Gate 1b：`RUSTFLAGS=-D warnings` 下 `cargo check -p engine-native --lib`（只限原生 lib；旧线工程存量警告不宜全局开启）；本地同命令通过 |
| 全量验收 | engine-native **348/0**（含 C3 新场景 1 项）、loader 1/0、content-runtime 31/0、engine-core 3777/0，合计 **4158 通过 / 0 失败**（[test-logs.txt](test-logs.txt)）；`RUSTFLAGS=-D warnings` 检查通过 |
| 规则/格式 | 规则 v60、格式 v2 不递增（依赖与 CI 配置改动不影响状态编码；14 golden 场景通过） |

## 结论边界

- 已实现并验收：O1（含 wasm 依赖闭包验证）、C3 跨层生命周期正反例、
  P3 lib `-D warnings` 防线（本地验证；CI 实际运行待推送后观察）。
- E03 monkey（欢乐暂缓项）**本批未实现，转待做**：上游语义为 equip5 宝物、
  他人使用【桃】时可弃猴子取消其结算并收桃（`card/skill.js` `useCardToBegin`
  取消 + 弃装备 + gain）。实现需要「使用前取消窗口」原语接入出牌提交状态机
  （现桃为同步结算、无前置窗口；`CardUseData` 加字段会动哈希与存档编码），
  实现深度超出本批预算，已作为主稿 E03 的推荐切片登记。
- 其余大块待做不变：E01/G0 37 来源功能清单、P2 规则对齐与整包 0/37、
  N10/N12 AI 与产品、O2–O5、名称类型 Arc<str> 深化。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（shasum -a 256，8 文件，相对
noname-next 根）。
