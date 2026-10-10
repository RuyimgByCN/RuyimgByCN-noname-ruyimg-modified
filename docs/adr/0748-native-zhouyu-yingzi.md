# ADR 0748：官方武将迁移续批——周瑜（英姿）

日期：2026-09-16
状态：已实施（engine-native）
关联：[0739](0739-native-diaochan-biyue-zhenji-luoshen.md)、[0740](0740-native-wusheng-longdan-viewas.md)、进度文档 N08/N11

## 背景与决定

官方迁移链路模板继续复制：**周瑜「英姿」（YING_ZI）**——摸牌阶段摸牌时额外
摸一张牌（`PhaseBegin(Draw)` 阶段触发，与闭月/洛神同族但目标阶段不同）。

### 实现

- `YingZiBehavior`：过滤（本人、Draw 阶段、begin）；
- `YingZiMachine { owner }`：单步摸一张即 Done；快照变体 `YingZi`（六处同步）；
- `STANDARD_GENERALS` 新增周瑜条目。

### 验收

`zhouyu_yingzi_extra_draw_at_draw_phase`：英姿闸门确认后手牌 +1、
摸牌日志断言、守恒。

## 影响

- 测试 181 项（native_kernel 104），净增 1。规则版本 16 → 17，
  14 golden 重录并通过；runner/原生牌桌复跑退出 0。
- N08 后续：大乔国色（view-as 延时锦囊——需 PlaceDelayed/JudgePhase 的
  声明牌种传播，属独立机制批次）。
