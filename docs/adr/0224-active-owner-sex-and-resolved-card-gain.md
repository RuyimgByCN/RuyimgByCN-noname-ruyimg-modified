# ADR 0224：主动技拥有者状态、性别目标与结算牌获得

## 背景

SP 糜夫人「存嗣」要求拥有者有手牌且正面朝上，将全部手牌交给一名男性角色，授予「勇决」
后翻面；「勇决」在其他角色的【杀】完整结算、未指定拥有者且实体牌仍可获得时每回合限一次
发动。旧 JavaScript 不能进入确定性运行时。

## 决策

ABI 151 增加主动技拥有者 `hand_nonempty`、`face_up` 与目标 `male` 合同；双性角色按男性
目标处理。新增通用 `give_all_hand`（HOST_EFFECT 160）和 `gain_card_if_discard`
（HOST_EFFECT 161），分别复用既有赠予与弃牌区获得管线；「勇决」复用 `sha_resolved` 事件和
技能实例每回合计数，不新增专用状态或 continuation。

## 结果

SP npack 同时发布糜夫人、「存嗣」与衍生「勇决」。新增 `EffectRequest` 使用稳定哈希标签
196、197；旧 JavaScript 仅作为语义与审计依据。
