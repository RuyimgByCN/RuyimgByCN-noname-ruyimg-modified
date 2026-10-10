# 2026-10-10 T10：机关桶（jiguantong）补齐 contentAfter 摸牌与逆序结算（规则 v193）

任务来源：[进度主稿 T10](../../../architecture/native-progress.md)；开工 HEAD `99d05026`。上游定义：`archive/upstream/noname-liuli/noname/card/swd.js:669`（机关火筒）。

## 上游语义（与 v192 原生差异的两处）

- `reverseOrder:true`：逐目标结算序为选定序的**逆序**（原生原为座位序）。
- `contentAfter: if(!event.preResult) player.draw()`：`preResult` 由每个受伤路径置位（无手牌直接火伤、取消弃牌改受伤）；被无懈抵消的目标不执行 content 不置位。全员无受伤 → 使用者摸 1。

## 改动内容

| 文件 | 改动 |
|---|---|
| `kernel.rs`（机关牌工厂臂） | jiguantong 机器装载时把 PlayCard 目标表 `clone` 后 `reverse`（座位序 → 结算逆序） |
| `content/host.rs`（`SwdJiguantongCard` Act 完成点） | targets 清空后先按日志判定：`ctx.out` 中无以本牌归因（`card == body`）的 `DamageDealt` → `ctx.draw_cards(owner, 1)`；随后本体入弃堆（原顺序）。按**实际结算伤害**判定，与上游 `preResult` 在 `damage()` 调用即置位的差别仅当伤害被完全阻止时可达，当前内容无此交互——已登记为口径（见规则基准 v193 条目） |
| `content/standard.rs` | `STANDARD_RULESET_VERSION` 192 → **193** |
| `tests/g1_wudian.rs` | `swd_jiguantong_aoe_discard_or_fire` 补「有人受伤不摸」断言（使用者手牌保持 2）；新增 `swd_jiguantong_reverse_order_and_content_after_draw`（首个弃牌询问 observer=2 证逆序；全员弃牌免伤后使用者摸 1 → 手牌 3；弃牌与本体入弃堆） |

存档编码未动（机器字段未增，`ARCHIVE_FORMAT_VERSION` 仍 v41）；摸牌判定读引擎日志（`Ctx.out`），不新增机器持久字段。

## golden 重录核验（AGENTS.md 第 4 条）

- 版本字段入 canonical_hash，升 193 必变全部 golden。
- **版本回退探针**：临时回退 `STANDARD_RULESET_VERSION=192` 复跑 `replay_scenarios` → **2/2 全绿**；随即恢复 193。证明 14 个哈希位移唯一来源是版本字段。
- jiguantong 行为分支仅经 swd 扩展牌种（`CardKind::Jiguan`）可达，14 个标准场景不使用扩展包，逐场景命令与断言不变。
- 按上述依据重录 14 个哈希（`replay_scenarios.rs` SCENARIOS 表，附重录依据注释），重录后 `replay_scenarios` 2/2 通过（含覆盖清单联动守护）。

## 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --test g1_wudian swd_jiguantong` | 2 通过（更新后回归 + 新增逆序/摸一） |
| `cargo test --offline -p engine-native --test replay_scenarios` | 2/2（重录后） |
| `cargo test --offline -p engine-native --no-fail-fast` | **527 通过 / 0 失败 / 1 ignore**（526+1，exit 0）——[完整日志](engine-native-tests.log) |

执行环境：本机，git `99d05026` + 本批改动；NGSA v41 不变。

## 边界

- 逆序仅作用于机关桶目标表反转；死目标跳过、无懈窗包装路径未变。
- 全部规则语义变化已登记入[规则基准 v193 条目](../../../architecture/native-rules-baseline.md)；v192 条目两条登记差异由本批撤销（原文保留作历史记录）。
