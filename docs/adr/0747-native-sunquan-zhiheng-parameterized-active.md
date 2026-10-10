# ADR 0747：N11 内容续批——孙权「制衡」参数化主动技

日期：2026-09-16
状态：已实施（engine-native）
关联：[0746](0746-native-n11-active-skill-command.md)、进度文档 N11/N08

## 背景与决定

[0746](0746-native-n11-active-skill-command.md) 落地主动技能命令面后，本批以
**孙权制衡**验证参数化主动技全链路：弃置任意张手牌，然后摸等量的牌
（出牌阶段限一次；无手牌时无法发动）。

### 实现（ZhiHengMachine 两阶段）

- `Select`：挂起 `zhiheng_select` 询问（SelectCards，min 0 / max 手牌数 /
  allow_cancel）——**空应答 = 放弃发动**；
- `Draw`：按应答弃置（逐张 discard_from_hand + CardDiscarded 日志），
  摸等量张（CardsDrawn 事件/日志）→ Done；
- 关键阶段迁移：Select 挂起前置 stage=Draw（避免恢复后重建同一询问的
  无限循环——迁移中发现并修复）。

### 验收（2 项）

- `sunquan_zhiheng_discard_n_draw_n`：手牌 3 → 弃 2 → 摸 2 → 手牌 3；
  弃牌入弃牌堆恰一份；每回合限一次（重复发动 `SkillAlreadyUsed`）；守恒；
- `zhiheng_decline_draws_nothing`：无手牌时发动 → `InvalidChoice`
  （activate 返回 None）。

## 影响

- 测试 180 项（native_kernel 103），净增 2。规则版本 15 → 16
  （制衡机器快照变体入编码与存档），14 golden 重录并通过；runner 复跑退出 0。
- 主动技命令面完成两类形态验证：无选择（苦肉）与参数化选择（制衡）。
