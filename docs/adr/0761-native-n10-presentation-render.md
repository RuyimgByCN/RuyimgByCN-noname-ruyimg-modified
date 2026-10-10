# ADR 0761：N10 表现事件层与渲染投影

日期：2026-09-22（第六十二批）
状态：已采纳
上游：[0730](0730-native-n10-view-isolation.md) / [0731](0731-native-n10-choice-events.md) / [0741](0741-native-n10-native-table-latency.md)

## 决策

1. **表现事件层**（`presentation::PresentationEvent`）：权威日志
   （`LogEvent`）→ 座位可见表现事件的纯投影。牌 id 经目录解析为牌名
   （UI 消费形态）；可见性规则与 `EventView` 一致——他人摸牌为暗置
   信息，表现层 `Drawn.cards = None`（类型层面无牌名），本人/观战者
   按规则收紧；`project()` 对 LogEvent 全变体穷尽（编译器保证新增
   日志变体必须显式决定表现语义）。
2. **渲染层**（`presentation::render_table`）：确定性文本帧渲染——
   座位行（体力/手牌数[暗]/双将[国战明置门控视图]）+ 阶段头 + 最近
   表现事件 + 终局横幅。纯函数、无时间与随机源，同输入逐字节同输出。
3. **引擎入口**：`Engine::presentation_for(seat|None)` 与
   `Engine::render_frame(seat|None, width)`；均为只读投影，不入快照
   与摘要，无规则版本影响。
4. 与既有层的组合保证：渲染帧的将領行直接取自 `PlayerView.generals`
   （ADR 0757 门控），暗置将因此天然不出现在他人/观战者帧。

## 验收

- `presentation_visibility_and_render_draws`：三类视角的摸牌可见性、
  渲染确定性（两次逐字节一致）、座位行与权威状态一致、终局横幅。
- `render_frame_guozhan_generals_isolation`：国战三视角（本人/他人/
  观战者）帧中暗置将零泄露、明置将公开。
- 全量 **207 passed / 0 failed**（本批 +2）；纯投影，无版本变化。

## 后果

N10 全部子项关闭（视图隔离/AiSession/有界池/原生牌桌/输入延迟/
表现事件/渲染层）。宿主渲染器（web/canvas）可在此基础上消费
`PresentationEvent` 流实现，属平台接入工作（阶段 2）。
