# ADR 0340：ABI 469 劫刀伤害增幅复合 + sp2 忙牙长

本批为 sp2 包忙牙长（`mangyachang`，2797705605）的「劫刀」（`spjiedao`，
4242283768）完整移植，需要 ABI 469 复合 `resolve_spjiedao(player)`。

## 原语设计（ABI 469）

- DSL 表面：`resolve_spjiedao(player)`；`trigger damage_before owner_is_source optional`
  （伤害结算前，可确认）。编译为 `effect_tag::RESOLVE_SPJIEDAO`（tag 390）。
- 引擎：`EffectRequest::ResolveSpjiedao { player, skill }`；阶段机（Custom 续体
  `RESOLVE_SPJIEDAO_CUSTOM_CHOICE`）：
  - 守卫：受伤（hp < maxHp）、本回合尚未造成过伤害（`player_dealt_damage`）、
    当前伤害段存在（`current_damage_id`）；
  - 选 X（Custom 数值 1..maxHp-hp，可取消）→ `increase_bound_damage(id, X)` →
    弃 X 张手牌/装备牌（SelectCards 精确 X 张）。

## 文档化近似

旧 JS 的弃牌绑定在该次伤害的 damageEnd（伤害结算后）；DSL 将弃牌前置到增幅同时。
可弃牌数不足 X 时跳过弃牌（近似旧 JS 结算时无法弃牌的情形）。

## 打包与验证

- `spjiedao.skill` 加入 sp2 README 显式列表；`mangyachang`（男/南蛮/4 体力，
  技能 [spjiedao]）加入 sp2 `characters.json`；catalog 标记 `ported`。
- sp2 npack 重建（ABI 469，40 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/sp2/`。
- 运行时测试 `original_sp2_spjiedao.rs`（3 用例全绿）：X=2 增幅（1+2=3 伤害并弃
  2 牌）、取消不增幅不弃牌、本回合第二次伤害不可增幅（确认后无增幅选择）。
- 旧 JavaScript 不进入 npack 或运行时。
