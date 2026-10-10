# ADR 0472 — shenhua 孙亮【溃诛/掣政/立军】(ABI 577)

- 状态:已接受
- 日期:2026-08-03
- 归属:C (omp)
- 关联 ADR:0471(卢植)、0466(蒯良蒯越)
- ABI:577

## 目标

发布 shenhua 孙亮完整武将:

- **溃诛**:弃牌阶段结束后,若你本回合弃置过牌,可令至多 X 名角色各摸一张牌,
  或对任意名体力值之和为 X 的角色各造成 1 点伤害(X = 本回合弃牌数)。
- **掣政**:出牌阶段结束时,若你本回合使用牌数少于攻击范围不包含你的角色数,
  弃置其中一名角色的一张牌。
- **立军**(主公技):吴势力角色于其出牌阶段使用实体【杀】后,你获得该牌,然后其
  可令你摸一张牌。

## 决策

### 溃诛 (ABI 577)

- `trigger phase_discard_end owner_is_target`(可选)→ `resolve_kuizhu(player)`。
- X = 弃牌阶段权威计数旗标 `DISCARD_PHASE_DISCARDED`(旧 JS 取本弃牌阶段丢牌数)。
- 分支选择(0 摸牌 / 1 伤害)→ 目标选择(至多 X 名)→ 各摸一张 / 体力值之和恰为
  X 时各受 1 点伤害(恢复时重校验)。

### 掣政 (ABI 577)

- `trigger phase_play_end owner_is_target`(可选)→ `resolve_zhizheng(player)`:
  本回合用牌计数(回合态 `CARD_ACTION_COUNT_BY_PLAYER`)与「攻击范围不包含你」
  角色数比较,开目标选择 + 弃其一张牌。
- 攻击范围判定复用 `dcchiying_in_range`(攻击范围查询 vs 距离查询)。

### 立军 (ABI 577)

- `skill lord_only` + `trigger card_used owner_alive`(可选)→ `resolve_lijun(player,
  source, card)`:校验来源为吴势力(`player_group`)、出牌阶段、事件牌为实体【杀】
  → 持有者获得该牌 + 来源摸一张。

## 接线

`ResolveKuizhu`(操作码 560)/`ResolveZhizheng`(561)/`ResolveLijun`(562)→ vm 弹栈 →
vm_host 注入 → primitives 复合 → 续体标签 `0xDCDC_5770..5772` → hash 臂 532-534 →
DSL 词 `resolve_kuizhu(player)`/`resolve_zhizheng(player)`/`resolve_lijun(player,
source, card)` → IR/codegen/门控 `ADDITIVE_RULE_VM_ABI_VERSION_577`。
(操作码初定 557-559 与并发 ADD_FIRST_MISSING_SKILL=557 冲突,移至 560-562。)

## 测试与发布

- Golden `original_shenhua_sunliang.rs` 5 项:契约(ABI 577/三触发)、溃诛摸牌分支、
  溃诛伤害分支(体力值之和校验)、掣政弃置攻击范围外角色牌、立军获吴势力实体杀
  并令其摸一张。全绿。
- 54 个 shenhua 测试文件零失败;engine-core 全量 0 失败;conformance 8/8 ALL PASS。
- npack:ABI 577、技能 139、角色 43(孙亮 id 2952980924,吴 3 体力 男 主公)、触发
  104、active 16、修正器 46、模块 120;`extensions/shenhua/{shenhua,package}.npack.json`
  同步。

## 近似与遗留

- 溃诛 X 以弃牌阶段权威计数表达(旧 JS 含弃牌阶段内的全部弃牌)。
- 立军旧 JS 的「然后其可令你摸一张牌」以来源自动摸一张表达(无额外征询)。
