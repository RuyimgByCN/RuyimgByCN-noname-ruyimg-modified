# ADR 0739：官方武将迁移续批——貂蝉（闭月）与甄姬（洛神）

日期：2026-09-16
状态：已实施（engine-native）
关联：[0736](0736-native-simayi-guicai-fankui.md)、[0738](0738-native-caocao-jianxiong-guojia-jiji.md)、进度文档 N08

## 背景与决定

官方迁移链路模板第三/四次复制：**阶段触发**类技能（闭月/洛神）——首次启用
`EventKind::PhaseBegin` 触发元数据（此前仅有伤害类触发）。

### 1. 貂蝉「闭月」（BI_YUE）

结束阶段开始时摸一张牌：触发 `PhaseBegin` + 过滤（本人、End 阶段、begin），
`BiYueMachine { owner }` 单步摸一张即 Done。快照变体 `BiYue`（六处同步）。

### 2. 甄姬「洛神」（LUO_SHEN）

判定阶段开始时可进行判定——黑色获得此牌并**再次判定**（循环至红色）：

- 触发 `PhaseBegin` + 过滤（本人、Judge 阶段、begin）；
- `LuoShenMachine { owner }` 单次判定一步：黑色 → 判定牌入手牌 + Yield 续行
  再判；红色 → 判定牌弃置、循环结束；
- 简化契约（已入档）：整链一次确认（OptionalGate），不分次放弃；
- 快照变体 `LuoShen`（六处同步）。

### 3. 版本 12 与验收

- 新机器变体 + 目录语义 → `STANDARD_RULESET_VERSION` 11 → 12，14 golden 重录；
  runner 复跑退出 0。
- `diaochan_biyue_draws_at_end_phase`：闭月闸门确认后 End 阶段摸一张
  （可选触发闸门断言 + 手牌增量 + 摸牌日志 + 守恒）。
- `luoshen_repeat_judgement_gains_black_cards`：洛神确认后至少获得一张黑色
  判定牌、循环终止（无悬挂选择）、守恒；实现中修正红色判定牌需弃置
  （初版直接消失破坏守恒）。

## 影响

- 测试 171 项（native_kernel 94），净增 2。规则版本 11 → 12。
- 迁移模板已覆盖：权限型（鬼才）、伤害触发（反馈/奸雄/遗计）、阶段触发
  （闭月/洛神）三类——后续武将按需组合。
