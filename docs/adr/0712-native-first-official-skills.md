# ADR 0712：鬼道/天妒首组技能切片（规则验收未完成）

- 状态：2026-09-17 初始切片及复核历史；当时 111 项通过，首组未完整迁移。
- 后续：2026-09-18 [ADR 0714](0714-native-guidao-tiandu-semantics-n04d.md) 已修复天妒范围、鬼道旧牌归属和 N04-d，规则版本为 5。
  **下文“当前/本轮”均指 2026-09-17**；装备资格/事件/恢复扩展及新 N04-e 以 [当前进度](../architecture/native-progress.md) 为准。
- 关联：[kernel](../../crates/engine-native-kernel/src/kernel.rs)、[snap](../../crates/engine-native-kernel/src/snap.rs)、
  [standard](../../crates/engine-native-content/src/content/standard.rs)、[当前进度](../architecture/native-progress.md)。
- 前序：[ADR 0711](0711-native-judge-order-alignment.md)；后续排序/版本见 [ADR 0713](0713-native-judge-grouping-version4.md)。

## 已落地

目录末尾追加鬼道/天妒（def 5/6，trigger=None）；实例复用 (owner, def)、enabled、授予与死亡移除模型。
目前测试通过 debug_grant 授予，尚无完整武将授予流程。JudgePhase 新增 JudgementProgress，保存
当前锦囊/判定牌、改判和天妒队列、asked/gained/in_rewrite，并入快照与摘要。

- 鬼道自判定者起按座位询问持有者；黑色手牌离手成为新判定牌，旧牌弃置，打出的牌最终弃置或被天妒获得。
- 天妒当前询问所有存活持有者，一人确认后获得最终判定牌；随后 apply_judgement 处理延时锦囊效果。
- 新增 4 项机制测试和 pause_roundtrip_guidao_rewrite；恢复守恒计入机器持有的锦囊/判定牌。

## 本轮纠正的规则来源

原文“旧线只有天妒常量、无实现”和“标准包文本为一名角色”均不成立：

| 来源 | 已核实内容 | 对当前实现的含义 |
|---|---|---|
| [旧线天妒内容](../../content/standard/tiandu.skill) 与 [标准包定义](../../content/standard/standard_m1.skill) | judgment_resolved + owner_is_target + gain_event_cards | 旧线已有内容实现，不能只查 characters.rs 常量 |
| [仓库上游天妒文本](../../../archive/upstream/noname/apps/core/character/standard/translate.js) / [实现](../../../archive/upstream/noname/apps/core/character/standard/skill.js) | “当你的判定牌生效后，你可以获得之。”；player: judgeEnd | 原生询问其他角色的天妒持有者与该来源不符（N08-a） |
| [仓库上游鬼道](../../../archive/upstream/noname/apps/core/character/shenhua/skill.js) | 支持黑色牌，含装备；替换时持有者获得旧判定牌 | 原生仅手牌且旧牌弃置，两项差异均须登记（N08-b） |
| [旧线判定](../../crates/engine-core/src/command/judge.rs) / [鬼道内容](../../content/original/ported/shenhua/zhangjiao.skill) | 座位改判队列、黑色手牌/装备资格 | 旧实现是参考，不能据此豁免上游差异或宣称官方等价 |

这些是本地版本证据；本轮没有完成外部官方规则版本认证。张角技能来源在上游 shenhua，
不应笼统把两位角色都写成同一标准包。

## 独立复核与测试边界

1. seed 207、4 人，1 号判定乐、2 号鬼道改黑牌、0 号天妒：0 号确实能取得他人的最终判定牌。
   两个现有天妒测试也锁定该行为，测试通过不能关闭 N08-a；修复须同时改正确的范围断言。
2. 实际顺序为初翻牌记录 Judged→改判→天妒询问→延时锦囊效果。窗口时 skip_play_phase 未设置，
   确认后才设置；判定牌生效与锦囊效果须分清，不能单凭此旗标认定时机错误。
   当前日志仅记录初翻牌，没有最终判定/获得事件，需定义事件契约并补对照。
3. 新恢复工程案例实际答 Pass；本轮独立探针补主动出黑牌→天妒确认，两选择点保存恢复的摘要和
   最终应答事件增量相符。尚未固化为工程回归，也不能替代天妒放弃、双方改判、成功救活等分支。
4. “鬼道使乐跳过出牌”的机制测试未直接断言 skip_play_phase/跳阶段；多持有者测试只覆盖
   第一人 Pass、第二人出牌。结果与边界断言仍需补强。
5. **N04-d 新反例**：在鬼道选择点快照中清空 current.rewriters，保留 asked/in_rewrite=true，
   restore 仍成功；回答 Pass 会触发 remove(0) panic。应在恢复时校验阶段、队列与选择关系，
   保留合法改判恢复及非法状态拒绝回归。

## 版本与验证

- 原生全套 **111 passed / 0 failed**：2 源码单元、64 机制、42 边界快照、2 场景、1 调试。
- 比上一轮净增 5 项；14 golden 随 ADR 0713 的版本 4 重录并通过，不包含首组技能完整场景。
- 版本 4 同时应登记本轮目录追加及 JudgePhase.current 机器编码扩展；后续修正技能语义继续遵守版本约定。

## 后续完成条件

1. N04-d 拒绝空应答队列及不一致选择，合法/非法测试成对保留，应答不得 panic。
2. N08-a 天妒仅对自己的最终判定牌生效；他人不询问/不获得，拥有者确认/放弃和改判组合均验证。
3. N08-b 明确选定鬼道版本，处理旧牌归属与装备资格/卸装效果，断言所有实体牌恰一份。
4. 定义最终判定、获得事件及先后关系；补直接规则结果、双持有者主动改判、技能失效和恢复矩阵。
5. 首组验收完成后继续内容扩面；N09 可准备 runner，但不能把当前技能差异计为规则等价性能负载。
