# ADR 0752：官方武将迁移续批——甘宁（奇袭）与张飞（咆哮）

日期：2026-09-16
状态：已实施（engine-native）
关联：[0744](0744-native-lvbu-wushuang-huangyueying-jizhi.md)、[0749](0749-native-xiaoji-qingnang.md)、进度文档 N08

## 背景与决定

官方迁移链路模板继续：**甘宁「奇袭」（QI_XI）**（新增过河拆桥牌种 +
黑色牌 view-as 转化）与**张飞「咆哮」（PAO_XIAO）**（杀无次数限制）。

### 1. 甘宁「奇袭」——过河拆桥新牌种

- `CardKind::Dismantle` 新牌种（目录新增 2 张拆桥牌）；
- 出牌路由：对一名其他存活玩家使用；效果机器 `DismantleMachine`：
  弃置目标一张牌（手牌优先 → 装备区 → 判定区），带无懈窗口；
- 奇袭转化：黑色手牌当拆桥使用（出牌校验转化块，优先级低于杀类）；
- 快照变体 `MachineSnapshot::Dismantle`（六处同步）；
- 验收 `ganning_qixi_black_card_as_dismantle`：黑色闪当拆桥——
  弃置目标一张手牌、拆桥牌照常弃置、桌面无残留、守恒。

### 2. 张飞「咆哮」（PAO_XIAO）

出牌阶段使用杀无次数限制：杀限检查经 `skill_by_name` 查询持有者，
持咆哮则跳过 `sha_used_play_phase` 限制。目录注册（权限型）。

### 3. 版本 19 与验收

- 新牌种 + 咆哮语义 → `STANDARD_RULESET_VERSION` 18 → 19，14 golden 重录；
- `zhangfei_paoxiao_removes_sha_limit`：连续三杀全部接受（各造成 1 点伤害）；
- `no_paoxiao_second_sha_rejected`：对照——无咆哮时第二杀 `ShaLimit` 拒绝。

## 影响

- 测试 190 项（native_kernel 111），净增 4。规则版本 18 → 19。
- 官方迁移武将十七名（新增甘宁/张飞）。
