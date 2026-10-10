# ADR 0431：SHIJI 吴懿闭环（流冰 / 合击，ABI 426 / 427）

本批发布 SHIJI 吴懿（`wujing`，3068213141，男/蜀/4 血）。技能「流冰」
（`liubing`）与「合击」（`heji`）完成闭环；旧 JavaScript 仅作语义来源。

## 决策

- **ABI 426 `resolve_liubing(player, card)`**：出牌阶段首次使用实体【杀】时
  （`card_used` + 既有过滤器 `owner_source_first_sha_this_play_phase`）将其
  花色**直接修改为方块**（`cards.get_mut(card).suit = Diamond`）——与旧 JS
  的持久实体修改一致，无新增状态字段（无 hash 结构性变化）。
- **ABI 427 `resolve_heji(player, target, card)`** + 新过滤器
  `owner_alive_other_single_target_juedou_or_red_sha`（他人单目标决斗/红杀后）：
  可选（`RESOLVE_HEJI_CHOICE`）对原目标视为使用【杀】或【决斗】
  （`UseVirtualCardOn`）。

## 验证

- 新增 `original_shiji_wujing.rs` 2 个行为测试：流冰实体杀改方块、合击打开
  二选一并执行虚拟决斗。
- 打包：71 技能 / 21 武将，`rule_vm_abi_version` 502。
- 顺带修复 tmp 编译缓存损坏（按 37.10 清理后重建）。
