# 2026-10-11 T11 斗地主 binglin 开局变体（规则 v205）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（v81 登记的「特殊开局摸牌变体未建模」消除）。上游依据：`mode/doudizhu.js` step 3——`game.gameDraw` 后 `next.num` 覆盖：binglin 变体 `player === game.zhu ? 5 : 4`。

## 建模

- `GameMode` 旁新增 `DoudizhuVariant { Standard, Binglin }`（state.rs）：**变体是建局输入，不入模式编码**——开局手牌数已进状态与 canonical_hash，存档无需变体位（存档格式不变 v47）。
- `new_with_mode/new_with_catalog` 内部链路加 `opening` 参数（既有构造器全部传 Standard）；`Engine::new_doudizhu_binglin(seed, seats, lord)`：开局地主 5 张/农民 4 张，其余行为（胜负/无奖惩/存档）与标准一致。
- 发牌数表先于 `core` move 计算（`&core.mode` 匹配 + opening 门控）。

## 登记差异（不建模面）

- binglin 的 `replacePile`（换牌堆）：原生用标准牌堆。
- **online 变体整体不建模**：地骑虚拟装备（`addVirtualEquip`）+选将后按 infoHp 加成的摸牌时点（原生开局发牌在建局、选将在后）——结构性时点依赖，与 mtg 派生包选择族同因。
- kaihei/欢乐等 UI 变体（全局技+流程面）不在「开局摸牌」范围。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `doudizhu_binglin_opening_draw` | binglin 地主 5/农民 4；地主死亡农民胜（胜负面一致）；开局后存档往返手牌一致；标准变体各 4 张 |
| 既有 `doudizhu_mode_victory_and_roundtrip` | 不变通过 |
| 版本回退探针（205→204 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段；斗地主不在标准场景可达路径） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 | **550 通过 / 0 失败 / 1 ignore**（549+1，2026-10-11 本机） |
| Gate 1b | exit 0 |
| 规则版本 | 204→**205**（存档格式不变） |
