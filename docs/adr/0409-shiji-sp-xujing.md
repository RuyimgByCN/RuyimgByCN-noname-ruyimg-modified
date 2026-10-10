# ADR 0409：SHIJI 许靖闭环（博明 / 恶荐，ABI 413 / 414）

本批发布 SHIJI 许靖（`sp_xujing`，3844942493，男/蜀/3 血）。技能「博明」
（`boming`）与「恶荐」（`ejian`）完成闭环；旧 JavaScript 仅作语义来源。

## 决策

- **ABI 413 `resolve_boming(player, target)`**：出牌阶段限两次的主动技（选 1 张
  手牌 + 选 1 名其他角色），把选中的牌赠给目标（`move_cards_with_reason` +
  `CardMoveReason::Give`，触发 `CardsGiven` 事件供恶荐响应）。赠牌卡由 vm_host
  从主动技 selection 填充。
- **ABI 414 `resolve_ejian(player, target)`**：`CardsGiven` 事件（过滤器
  `owner_alive_gave_cards_to_other_alive`）后，目标若另有与赠牌同大类
  （基本/锦囊/装备）的手牌或装备牌，则二选一（`ContinuationKind::Custom` +
  `RESOLVE_EJIAN_CHOICE`）：受 1 点伤害 / 展示手牌并弃置所有同类牌
  （`RevealCards` + `discard_cards`）。
- 旧 JS 每名目标仅触发一次恶荐（storage 记录），DSL 逐次触发，文档化近似。

## 验证

- 新增 `original_shiji_boming_ejian.rs` 4 个行为测试：博明赠牌、恶荐二选一
  choice、受伤害分支、弃同类分支。
- 打包：53 技能 / 13 武将，`rule_vm_abi_version` 502。
- 顺手补齐并行会话（Codex，ABI 532 孙綝）缺失的 `distinct_suits` schema 字段、
  构造点与 `resolve_potnigu`/`resolve_potlulian` stub（dispatch 已接线但实现
  未落地，stub 返回 ContentFault 显式报错，待 Codex 替换）。
