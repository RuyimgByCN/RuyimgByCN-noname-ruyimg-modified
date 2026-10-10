# ADR 0394：ABI 522 兵势夏侯霸（砺兵/破袭）

本批发布兵势夏侯霸（`pot_xiahouba`，2620190059）及「砺兵」
（`potlibing`，2358132416）、「破袭」（`potpoxi`，4156018278）。旧
JavaScript 仅作为语义合同，不进入 npack 或运行时。

## 决策

- 「砺兵」以技能实例状态保存攻击范围、三段循环、递增摸牌数和待消费伤害加值：
  非伤害牌结算依次获得攻击范围 +1、递增摸牌、下一张伤害牌伤害递增；伤害牌使用时
  消费加值并在结算后清空状态。
- 「破袭」监听拥有者弃置且仍在弃牌堆的实体【杀】，按事件顺序逐张强制选择合法目标，
  无距离与次数限制地复用普通用牌管线；目标手牌数不大于拥有者时，仅该目标不可响应。
- ABI 522 新增 `owner_skill_state_u32`、`owner_discarded_sha_to_discard` 与
  `resolve_potlibing` / `resolve_potpoxi`；其余选牌、移牌、响应和伤害流程复用既有实现。
- 兵势包同步到 `extensions/bingshi/`，包内 18 名武将、42 个技能，
  `rule_vm_abi_version = 522`。

## 验证

- 势夏侯霸 Golden 4/4、兵势 Golden 53/53。
- `content-compiler` 470/470；`rule-vm` 单元与 fuel 门禁、`engine-api` 通过。
- conformance 8/8；兵势扩展在空内容仓库冷导入通过。
- 三份兵势包字节一致，SHA-256：
  `bdab92dbbc36bbe84e0976c3ce98995c06a45c51c33b87c5aa7dd5b51ba5ad56`；
  逻辑哈希：`4eb320aa6e7ef47277aa54c01abc14bb8b0a929fe446d892bb683ce7b0651ee5`。
- 编译产物按架构计划 §37.10 使用统一外部 target，并在批次门禁结束后清理；
  `.npack` 与 `extensions/` 不属于清理对象。
