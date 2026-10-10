# ADR 0363：ABI 497 远略赠牌后可用牌 + SP2 张郃

本批完整移植 SP2 张郃（`sp_zhanghe`，4161518802）及「远略」
（`yuanlve`，3073596171）。旧 JavaScript 仅作为语义来源，不进入运行时。

## 原语设计（ABI 497）

- 主动选择新增通用过滤 `card_non_equip`，按有效牌类别排除装备牌。
- DSL 效果 `give_selected_then_target_may_use_draw_owner(player, target)` 编译为 host
  effect tag 420。
- 引擎先将所选实体手牌赠予目标；移动事件完全结算后，目标可取消或使用该牌。需要目标
  时复用普通牌的合法目标计算与选择；成功提交普通用牌管线后，拥有者摸一张牌。

## 打包与验证

- `yuanlve.skill` 与 `sp_zhanghe` 加入 SP2 包，重建并同步 `extensions/sp2/`。
- 黄金测试覆盖包内武将闭包、装备牌过滤、实体赠牌、目标使用与成功后摸牌。
