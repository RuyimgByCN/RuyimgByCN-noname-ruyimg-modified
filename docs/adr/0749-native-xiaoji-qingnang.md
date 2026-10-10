# ADR 0749：官方武将迁移续批——孙尚香（枭姬）与华佗（青囊）

日期：2026-09-16
状态：已实施（engine-native）
关联：[0739](0739-native-diaochan-biyue-zhenji-luoshen.md)、[0746](0746-native-n11-active-skill-command.md)、进度文档 N08/N11

## 背景与决定

官方迁移链路模板继续复制：**装备失去触发**（枭姬——首次启用
`EventKind::Equipped` + replaced 判定）与**参数化主动治疗**（青囊——
主动技 + 目标/弃牌两段询问）。

### 1. 孙尚香「枭姬」（XIAO_JI）

失去装备区里的一张牌（同栏替换）时摸两张牌：

- 触发 `EventKind::Equipped / optional: true`，过滤
  `Equipped { player == owner, replaced: Some(_) }`；
- `XiaoJiMachine { owner }` 单步摸二即 Done；快照变体六处同步。

### 2. 华佗「青囊」（QING_NANG）

出牌阶段限一次，弃一张手牌并令一名已受伤的其他角色回复 1 点体力：

- `activate` 前置校验：持有者存活、有手牌、场上存在已受伤的其他角色
  （不满足则无法发动）；
- `QingNangMachine` 两段：Target（qingnang_target：已受伤其他角色）→
  DiscardChoice（qingnang_card：弃一张手牌并治疗目标 1 点）→ Done；
- 快照变体 `QingNang { owner, stage: QingNangStageSnap, healed }`。

### 3. 验收（2 项）

- `sun_xiangxiang_jixiang_draws_on_equip_loss`：替换武器触发枭姬——
  闸门确认后手牌 +2、被替换武器弃置恰一份、守恒；
- `huatuo_qingnang_heals_wounded_player`：目标询问（qingnang_target）→
  弃牌询问（qingnang_card）→ 目标回复 1 点体力、药引弃置恰一份、守恒。

## 影响

- 测试 183 项（native_kernel 105、view_isolation 7），净增 3。规则版本 16 → 17
  （枭姬/青囊快照变体入编码），14 golden 重录并通过；runner 复跑退出 0。
- N08 后续：国色 view-as 延时锦囊（需 PlaceDelayed/JudgePhase 声明牌种
  传播，属独立机制批次）。
