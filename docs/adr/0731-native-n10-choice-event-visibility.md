# ADR 0731：N10 第二批——开放选择与表现事件的可见性契约

日期：2026-09-16
状态：已实施（engine-native）
关联：[0730](0730-native-n10-player-view-isolation.md)、进度文档 N10

## 独立复核更正（2026-09-22）

ChoiceView 隐藏选项、CardsDrawn 隐藏他人摸牌仍通过；EventView 却直接公开 ChoiceAnswered。
观星私有选择 Card(60) 被另一座位 AiSession 读出（N10-a）。需按事件载荷建立可见性规则，不能称全部日志信息隔离完成。

当前反例、优先级与证据见 [进度主稿](../architecture/native-progress.md)。以下保留当批设计/记录，不作为当前完整验收结论。

## 背景与决定

[0730](0730-native-n10-player-view-isolation.md) 交付状态投影后，本批补齐两类
**动态信息**的座位可见性契约：开放选择（选项会暴露应答者手牌——桃/闪）与
日志事件（摸牌为暗置信息，牌 id 不可见）。

### 1. 选项可见性（`ChoiceView`）

公开知识分层：**询问的存在、观察者与提示类型是公开的**（众人都知道有人被询问
什么——濒死救援/改判/出闪）；**选项内容仅观察者本人可见**（选项即应答者手牌中
可用的桃/闪列表，泄露即全知）。

- `Engine::choice_view_for(seat: Option<PlayerId>) -> Option<ChoiceView>`：
  - `ChoiceView::Own { id, prompt, options, min, max, allow_cancel }`（观察者本人）；
  - `ChoiceView::Other { id, observer, prompt }`（他人与观战者，无选项字段——
    类型层面不可能泄露）。

### 2. 事件可见性（`EventView`）

摸牌是暗置信息：**他人摸到的牌 id 不可见（仅数量）**；其余日志事件（伤害/濒死/
弃置/判定等）为公开信息原样可见；自己的摸牌完整可见。

- `Engine::event_view_for(seat) -> Vec<EventView>`：
  - `EventView::Public(LogEvent)`（公开事件原样，含自己的摸牌）；
  - `EventView::CardsDrawnHidden { player, count }`（他人摸牌——**类型上无
    cards 字段**，编译期即不可泄露）。
- 视图与日志一一对应（同长度同顺序），宿主可按序重放表现。

### 3. 验收（view_isolation 新增 2 项，共 5 项）

- `choice_options_visible_only_to_observer`：濒死救援布阵（1 号持桃被询问）——
  观察者 Own 含桃 id；其他座位与观战者 Other（仅 observer+prompt）；选项牌 id
  不出现在公开视图任何位置。
- `event_view_hides_other_players_drawn_cards`：多轮推进后逐座位断言——Public
  形态的 CardsDrawn 必为本人；Hidden 形态数量与原事件一致（ids 类型上不存在）。

## 影响

- 测试 153 项（view_isolation 5 项），净增 2。规则版本不变（8）；golden 不变。
- N10 剩余：AI 信息隔离接入与有界计算池、原生牌桌与输入延迟；N11/N12 平台续批。
