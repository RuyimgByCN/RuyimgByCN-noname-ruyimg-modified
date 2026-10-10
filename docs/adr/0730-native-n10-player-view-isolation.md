# ADR 0730：N10 权威状态与可见视图分离（信息隔离第一批）

日期：2026-09-16
状态：已实施（engine-native）
关联：[0730 前序批次](0729-native-n08-armor-bagua-generals-n09-counter-compare.md)、进度文档 N10、性能架构 v3.2 平台阶段

## 背景与决定

N10 平台阶段首批：**权威状态与可见视图分离**。此前引擎只暴露权威状态
（`state_core`/调试 API）——客户端/AI 若直接读取即获得全知视角（对手手牌、
牌库内容与顺序）。本批引入只读投影层，使「宿主能拿到什么」与「权威状态有什么」
解耦。

### 视图模型（`src/view.rs`）

- `PlayerView`：座位视角的完整可见状态——公开字段（弃牌堆/牌桌面/装备区/
  判定区/体力/存活/回合/阶段/轮次/胜负/规则版本）+ 牌库仅数量（`deck_count`）。
- `PlayerCellView.hand: HandView`：`Own(Vec<CardId>)`（仅本人）或 `Hidden`
  （对手与观战者，仅 `hand_count` 有意义）。
- `Engine::view_for(seat)` / `Engine::view_spectator()`：纯投影构造——不修改
  状态、不入快照与摘要（视图不在权威状态中，无哈希影响，规则版本不变 8）。

### 原型契约（本批边界）

- 死亡玩家手牌仍 Hidden（官方规则死亡公开手牌，留待后续批次）；
- 开放选择（`OpenChoice`）的可见性未纳入视图（仅状态投影，选择可见性随
  表现事件批次）；
- 技能授予情况暂不入视图。

### 信息隔离验收（新测试目标 `tests/view_isolation.rs`，3 项）

- `seat_views_hide_opponent_hands_and_deck`：逐座位——自己手牌可见且与权威一致、
  其余 Hidden；**视图可见牌 id 集与牌库内容求交为空**（牌库内容与顺序均不可泄露）；
  观战者视图全部 Hidden。
- `view_public_fields_match_authoritative_state`：三视角公开字段与权威状态逐项
  一致；视图构造不改变状态哈希（纯投影）。
- `isolation_holds_across_game_progress`：纯命令推进 40 步，每步后重查隔离
  主断言（隔离随对局推进持续成立）。

## 影响

- 测试 151 项（新增 view_isolation 3 项）。
- 规则版本不变（8）；golden 不变（视图为只读投影，不改状态与编码）。
- N10 后续：开放选择/表现事件的可见性契约、AI 信息隔离接入与有界计算池、
  原生牌桌与输入延迟。
