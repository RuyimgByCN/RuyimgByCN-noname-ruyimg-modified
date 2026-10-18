# swd 批 146：轩辕剑族（引擎侧）+ 牌实体待补登记（字段批 v320/存档 v104，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 **轩辕剑**（`card/swd.js` 卡 + 三子技）与 **轩辕**（`character/swd.js`）——本批落地**引擎侧**（牌种/子技/装备钩/取剑机），**牌实体**留待同批重导种子敏感用例
- **规则 v319→320、存档 v103→104**
- 上游回读：`card/swd.js`（`xuanyuanjian` 卡 + `xuanyuanjian2`/`xuanyuanjian3` 子技）、`character/swd.js`（`xuanyuan` 31 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 牌 | `xuanyuanjian`：equip1、unique、`distance attackFrom:-2`、onEquip（体力 ≤2 且无「轩辕」→ 弃置；否则 +1 护甲） | **新牌种** `CardKind::Xuanyuanjian`（槽=武器、射程 3、键 `xuanyuanjian`、种类码 174）；`commit_equip` 装备钩：体力 ≤2 且无「轩辕」→ 撤技弃置，否则 `change_hujia(+1)` |
| 子技 1 | `xuanyuanjian2`：`source damageBefore` forced，`event.notLink()` → **+1 伤害并转雷属性**、置 `_xuanyuanjian` | `XuanyuanjianDamage`（tag 503）：`DamageBefore`（来源=自己、未防止、非铁索传导）→ 帧内 `amount+1`、普通属性转雷、置 `turn.xuanyuanjian_marked` |
| 子技 2 | `xuanyuanjian3`：`source damageAfter` forced，`_xuanyuanjian` 且无「轩辕」→ `loseHp()` | `XuanyuanjianBacklash`（tag 504）：`Damage`（来源=自己且标记位）→ 无「轩辕」时 `LoseHp` 1；清标记 |
| 轩辕 | `phaseEnd` 限定技，可选：弃一张黑桃牌 → 弃牌堆取【轩辕剑】装备 | `Xuanyuan`（tag 505）：`PhaseEnd` 触发 + 可选确认门；`SelectCards`（黑桃牌）→ 弃置 → 弃牌堆按**牌种**取剑入手 → `Equipped` 帧入槽 |
| 使用门 | `enable`：`player.hasSkill('xuanyuan')||player.hp>2` | `PlayCard` 校验：无「轩辕」且体力 ≤2 → `CardLimit` |

包数据：包内**未加牌实体**（见下）；包级声明 `xuanyuan`/`xuanyuanjian2`/`xuanyuanjian3` → 对应宿主行为键（使引擎侧行为可授予、可测）；npack 已 rehash。

## 牌实体为何留待同批（重要）

加入牌实体会改变**种子化牌堆与武将分配**（同一 seed 的 RNG 序列随之变化），实测导致 **8 项种子敏感用例**失败（3 项为计数冻结、5 项为布阵/行为断言：`swd_cards_alignment_guard_b29`、`swd_cards_trick_b42_yuchan_autoviewas`、`swd_extra_cards_disposition_guard`、`swd_generals_b63_*`、`swd_generals_b69_*`、`swd_hufu_self_dying_only`、`swd_real_host_full_lifecycle`、`lingwu_inserts_extra_play_phase_when_used_reaches_hp`）。

因此本批**先落引擎侧并保持牌堆不变**（89 张，全部用例复绿），牌实体与上述 8 项用例的重导作为**后续批次**（登记项 `T12-SWD-XUANYUANJIAN-CARD`）。包内 `xuanyuan` 仍为 deferred（理由已更新为「牌实体待补」）。

## 计数

- 武将技能条目：**deferred 保持 6**（`xuanyuan` 仍 deferred，但引擎侧行为已落地并登记）；swd 映射 **176/184**。
- 登记表：**新增 open 项** `T12-SWD-XUANYUANJIAN-CARD`（牌实体 + 8 项种子敏感用例重导）。

## 验证

```
cargo test --offline -p engine-native --test swd_xuanyuanjian_card
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（轩辕剑族） | **2 passed / 0 failed**（[targeted.log](targeted.log)）：加伤 +1（1→2）且无「轩辕」时反噬失去 1 体力；对照：持「轩辕」不反噬；`xuanyuanjian_marked` 存档往返 |
| engine-native 全量 | **1027 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)）——牌实体回退后 8 项种子敏感用例复绿 |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 重录 + 版本回退探针（320→319 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v320.log](replay-golden-v320.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 502（tag 503–505 追加）；`CardKind::ALL` 174 |

## 登记差异

- `event.notLink()` 以原生 `DamageData.from_chain`（铁索传导）承载 ✓ 同口径。
- `event._xuanyuanjian` 标记以 `TurnState.xuanyuanjian_marked` 承载（事件载荷未加字段）。
- 取剑路径：弃牌堆 → 手牌 → `Equipped` 帧（经手牌中转，上游 `player.equip(card,true)` 直装）。
- 牌实体与 8 项种子敏感用例重导 → 后续批次（`T12-SWD-XUANYUANJIAN-CARD`）。
