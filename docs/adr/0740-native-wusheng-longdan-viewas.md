# ADR 0740：官方武将迁移续批——关羽（武圣）与赵云（龙胆）view-as 转化语义

日期：2026-09-16
状态：已实施（engine-native）
关联：[0736](0736-native-simayi-guicai-fankui.md)、[0738](0738-native-caocao-jianxiong-guojia-jiji.md)、进度文档 N08

## 背景与决定

官方迁移链路模板第四类技能语义：**view-as 牌转化**（将一张牌按另一种牌种
使用）——关羽武圣（红色手牌当杀）与赵云龙胆（杀当闪/闪当杀）。

### 1. 转化机制（use_as）

- `CardUseData` 增加 `use_as: Option<CardKind>`：转化使用时以此声明牌种结算；
- `PlayCard` 校验管线：目录牌种 →（武圣：红色手牌；龙胆：闪）→ 有效牌种 Sha
  ——路由（杀的距离/杀数上限/目标校验）与 CardUsed 帧全部按声明牌种执行；
- `commit_card_used` 与 `intrinsic_machine` 的牌种判定改为 `use_as` 优先
  （修复：commit 曾按目录牌种把武圣转化的桃当桃结算——弃置+自我回复）；
- `CardUseData` 编解码同步（archive use_as 字段）。

### 2. 龙胆防御侧

`ShaResolveMachine` 出闪询问的选项并入目标手牌中的杀（龙胆持有者）——
恢复路径按「应答牌即弃置」统一处理，杀当闪与闪同构。

### 3. 验收（2 项）

- `guanyu_wusheng_red_card_as_sha`：红桃桃当杀——伤害 1（DamageDealt card=该桃）、
  牌照常弃置、守恒；
- `zhaoyun_longdan_sha_as_dodge`：出闪询问选项含目标手牌中的杀、出杀代闪
  免除伤害、转化牌照常弃置、守恒。

### 4. 版本 13

转化语义 + CardUseData.use_as 编码 → `STANDARD_RULESET_VERSION` 12 → 13，
14 golden 重录并通过；runner 复跑退出 0。

## 影响

- 测试 173 项（native_kernel 96），净增 2。规则版本 12 → 13。
- 迁移模板五类触发齐备：权限型（鬼才/武圣/龙胆）、伤害触发（反馈/奸雄/遗计）、
  阶段触发（闭月/洛神）。
- 附带修复：`commit_card_used` 的 view-as 盲区（转化牌按目录牌种错误结算）。
