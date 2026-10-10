# ADR 0365：ABI 498 统北类别交牌或增伤 + 荆扬曹操

本批完整移植 OFFLINE 荆扬曹操（`jy_caocao`，4130532255）的「统北」
（`jytongbei`，2908493496），并与既有「砺军」闭合武将。旧 JavaScript 仅作为语义
来源，不进入运行时。

## 原语设计（ABI 498）

- 参数化过滤器 `owner_source_target_not_group_<tag>`：伤害来源为技能拥有者，且目标
  势力不等于后缀；本技能使用 `wei`。
- `choose_category_then_target_give_matching_or_damage_plus_one(player, target)` 复用「征服」
  的基本/锦囊/装备类别声明与匹配交牌续体；目标无对应牌或取消交牌时，将绑定的待结算
  伤害增加一点。
- 目标完全没有手牌和装备牌时跳过无意义的类别选择，确认发动后直接增伤，与旧规则等价。

## 打包与验证

- `jytongbei.skill` 与 `jy_caocao` 加入 OFFLINE 包并同步 `extensions/offline/`。
- 黄金测试覆盖同类别交牌不增伤、拒绝交牌增伤及武将技能闭包。
