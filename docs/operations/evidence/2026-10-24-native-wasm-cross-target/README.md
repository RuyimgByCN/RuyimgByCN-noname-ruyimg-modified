# 真实 native 与生成 WASM 的轨迹对照（P2/NC08 第一阶段，2026-10-24）

核验基准 `7e28a93799a9c01794085e911a72a4faa5673883` 加未提交工作树；规则 **v322**、存档 **v105**。本批完成 P2 第 8 项的**首个真实跨目标切片**：同一标准内容、种子、命令序列、选择策略与中途恢复，在 Rust native 与**本轮 wasm-pack 生成的真实 WASM** 上逐步比较 revision／state hash／公开事件码。**不是** host 包装层直调，也不是正式扩展包或全来源验收。

## 方法

- native：`crates/engine-native/tests/nc08_cross_target_real.rs`，`Engine::new(seed, 1)`；每步若有开放选择则取前 `min` 个候选（与 WASM 完全相同的确定性策略），否则 `NextPhase`；第 20 步保存、编码、解码、恢复后继续 20 步。
- WASM：`docs/operations/evidence/2026-10-24-native-wasm-cross-target/wasm-trajectory.mjs`，加载 `apps/web-client/src/worker/native-pkg/engine_native_wasm_bg.wasm`（本轮 `build-native.mjs` 产物），同 seed/策略；`save_hex` 后换新实例 `restore_hex` 继续。
- 比较：`compare.py` 逐项比较 40 个 direct 步骤、20 个恢复前步骤、20 个恢复后步骤的 `revision/hash/codes`；并核对 direct 与 restore 两线在恢复点重合。
- 一键复跑：`python3 -B scripts/check-native-wasm-cross-target.py`（包含标准与 huanle 正式包两段）。

## 结果 A：标准内容

见[比较输出](comparison.json)：

```json
{
  "direct_steps": 40,
  "restore_before_steps": 20,
  "restore_after_steps": 20,
  "final_revision": 40,
  "final_hash": "e3ebfbec9232b056cc86edc7899475af431176b1e42a863c01cbed0e5c88cbf6",
  "native_wasm_match": true
}
```

- native 测试 exit 0（[日志](native-test.log)），WASM 探针 exit 0（[日志](wasm-test.log)），比较 exit 0（[checker](checker.log)）；`engine-native` 完整 **1548/0/71**，`engine-native-wasm` 完整 **51/0/0**（2026-10-24 Worker VM 批后）（[计数](engine-native-counts.json)/[计数](engine-native-wasm-counts.json)）。
- 恢复轨迹：direct 第 1–20 步与 restore.before 完全一致；direct 第 21–40 步与 restore.after 的 revision/hash/codes 完全一致。
- 每条命令的事件码已由 P2 事件批接通，不再是空数组；hash 为 64 hex。

## 结果 B：正式安装包 `huanle-cards`

同一 packs 字符串（`huanle-packs.json`，来自 `packages/extensions/huanle-upstream-noname` 的 manifest + `assets/cards.json`）分别进入 `engine_from_packs` 与 `NativeEngine.new_with_packs_json`，20 条命令、第 10 步存档恢复：

```json
{
  "direct_steps": 20,
  "restore_before_steps": 10,
  "restore_after_steps": 10,
  "final_revision": 20,
  "final_hash": "12ac6844f49be1ed5c68e249def977725d5f798507be00dcbf7576f56356d1cd",
  "native_wasm_match": true
}
```

见[包比较](package-comparison.json)、[native 日志](native-package-test.log)、[WASM 日志](wasm-package-test.log)。这证明**正式安装包数据、目录、规则码与存档恢复**在两侧一致；仍不是选将/多座位/扩展技能交互或 37 来源验收。

## 失败记录

1. 首版 native 侧把“命令返回 `Pending`（刚打开弃牌选择）”当成拒绝，在 step 2 失败；WASM 探针同样未处理 `open_choice` 后的应答。改为两边统一的“有选择先答前 `min` 个候选，否则推进阶段”，并接受 `Accepted|Pending` 后通过。该失败是测试驱动逻辑错误，不是 native/WASM 产品分歧；原始错误文本保留在此。
2. 第二次 native 在打开选择后仍提交 `NextPhase`：`open_choice()` 只在选择已存在时返回 `Some`，首条打开选择的命令返回 `Pending` 而非 `Accepted`。修正为接受 `Pending` 并让下一轮应答；两侧策略同步后才得到一致轨迹。

## 边界

- 覆盖标准 1 人局 40 命令与 **huanle-cards 正式安装包 20 命令**；未覆盖选将、多座位、随机技能、私有选择嵌套、剑阁/Boss 或长局。不能据此宣称 NC08 或 37 来源等价。
- 选择策略是“取前 min 个候选”的确定性测试策略，不代表上游玩家行为；但两边完全相同，因此能发现状态/事件/恢复轨迹分歧。
- 事件仍为粗粒度码；未比较事件 payload、可见性投影或录像。
- 未做 GitHub Actions 实跑；脚本可在本地复跑。未改规则/存档版本或包内容锁。
