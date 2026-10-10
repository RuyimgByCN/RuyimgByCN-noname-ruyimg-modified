# ADR 0750：官方武将迁移续批——吕蒙（克己）

日期：2026-09-16
状态：已实施（engine-native）
关联：[0739](0739-native-diaochan-biyue-zhenji-luoshen.md)、[0747](0747-native-sunquan-zhiheng-parameterized-active.md)、进度文档 N08

## 背景与决定

官方迁移链路模板继续复制：**吕蒙「克己」（KE_JI）**——条件型手牌上限修正：
若本回合未使用杀，则结束阶段无需弃牌（手牌上限视为无限）。

### 实现

- 目录注册 `KE_JI`（权限型，无触发时机——弃牌阶段规划经 skill_by_name 查询）；
- 内核弃牌阶段规划接入：`EventData::Phase(Discard, begin)` 的固有机器构造前
  检查 `KE_JI` + `!turn.sha_used_play_phase`——未出杀则不产生弃牌机器
  （无弃牌询问，手牌保留）；
- 出过杀的回合克己不生效，正常弃牌逻辑接管。

### 验收

`lvmeng_keji_skips_discard_without_sha`：手牌塞 6 张且未出杀，推进至 End
阶段——无弃牌询问、手牌保留 6 张。对照路径（出过杀→正常弃牌）由
discard_to_limit golden 与既有用例覆盖。

## 影响

- 测试 184 项（native_kernel 105），净增 1。规则版本 17 → 18
  （克己改变弃牌语义），14 golden 重录并通过；runner 复跑退出 0。
- 官方迁移武将十五名。N08 后续按模板继续（国色 view-as 延时锦囊需
  声明牌种传播，属独立机制批次）。
