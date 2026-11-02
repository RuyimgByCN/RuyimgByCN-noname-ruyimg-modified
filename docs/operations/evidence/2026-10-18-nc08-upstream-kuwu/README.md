# NC08 第⑦层样本 5：`kuwu`（苦无，L19 欢乐卡牌）（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层「上游行为」对照样本 **5/37 来源**
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：由上游 JS 推导的 **6 条规则点**：**2 条完全对齐**（装备技、锁定/杀族判定），
  **1 条部分对齐**（强制弃牌成立，但弃牌来源与选择权退化 → K4），**4 处差异登记**
  （K1 铁索传导、K2 `he` 有牌门槛、K3 `_notrigger`、K4 弃牌自选 → 确定性首张）
- 来源覆盖：L19 欢乐卡牌**首次**进入第⑦层上游对照（此前为 L20 / U07 / 标准面）

## 1 上游源码（归档快照，逐字引用；见 [upstream-kuwu.js.txt](upstream-kuwu.js.txt)）

`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:53-70`：

```js
kuwu: {
    audio: "ext:欢乐卡牌/audio/skill:true",
    trigger: { source: "damageSource" },
    forced: true,
    equipSkill: true,
    filter(event, player) {
        if (event._notrigger.includes(event.player)) {
            return false;
        }
        return event.card && event.card.name === "sha" && event.notLink() && event.player.countCards("he") > 0;
    },
    async content(event, trigger, player) {
        await trigger.player.chooseToDiscard(true, "he");
    },
},
```

`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/card.js:81-94`（武器栏 `equip1`，
`distance.attackFrom:-1`，`skills:["kuwu"]`）：

```js
kuwu: { type: "equip", subtype: "equip1", skills: ["kuwu"], nomod: true,
        nopower: true, unique: true, distance: { attackFrom: -1 } },
```

语义辅助（同快照）：
- `notLink()` = 父事件既非 `_lianhuan` 也非 `_lianhuan2`（`noname/library/element/gameEvent.ts:1058`），
  即**铁索连环传导伤害**；
- `untrigger(false, player)` 把该角色加入 `_notrigger`（`gameEvent.ts:646`），
  `noname/library/index.js:11137` 在触发分发时对 `_notrigger` 命中的角色跳过该技能。

## 2 由 JS 推导的规则点 → 原生验收（新增 `nc08_upstream_kuwu.rs`，5 项）

| # | 上游依据 | 判据（断言） | 结果 |
|---|---|---|---|
| 1 | `equipSkill:true` + 卡牌 `skills:["kuwu"]` | 装苦无 → 持有 `huanle_kuwu`；卸下 → 失去 | **对齐** |
| 2 | `trigger.source:"damageSource"`、`forced:true`、`event.card.name==="sha"` | 非【杀】伤害（无牌 / 非杀牌）不触发；触发时**无** `trigger_confirm` 门 | **对齐** |
| 3 | `event.player.countCards("he") > 0` | 受伤角色**无手牌/无装备**时上游 filter 直接为假 | **差异 K2**（原生无此门槛） |
| 4 | `event.notLink()` | 铁索连环**传导伤害**不触发 | **差异 K1**（原生仍触发） |
| 5 | `event._notrigger.includes(event.player)` | 目标被 `untrigger(false,player)` 标记时不触发 | **差异 K3**（原生无该概念） |
| 6 | `chooseToDiscard(true,"he")` | 受伤角色**强制**弃 1 张 `he` 牌（自选） | **差异 K4**（原生确定性弃首张手牌） |

**原生实现引用**（工作树当前布局）：
- `crates/engine-native-content/src/content/host/filtering/common.rs:531-534`：伤害过滤臂
  （`source==owner && !prevented && amount>0 && card.is_sha_like()`）；
- `crates/engine-native-kernel/src/vocabulary/host/metadata/common.rs:6-12`：`EventKind::Damage` /
  `optional:false`（锁定）；
- `crates/engine-native-content/src/content/host/step/skills_status.rs:666-681`：效果——
  `player(t).hand.first()` 弃置；
- `crates/engine-native-kernel/src/vocabulary/host/tags.rs:303,549` 与 `vocabulary/host/keys.rs:315`：
  tag 0 ↔ `host:sha_damage_force_discard`（包 `cards.json` 的映射名）；
- `crates/engine-native-kernel/src/kernel/damage_flow.rs:99-131`：铁索传导伤害以
  `from_chain:true` + 原 `source`/`card` 再入伤害结算（2026-11-02 C331 拆分后位置；原引用为拆分前
  `damage.rs:988-1015`，C331 起按上游逐个目标先解链再传导）。

## 3 差异登记（差异 + 影响分析 + 待办）

| 编号 | 差异 | 影响分析 | 待办 |
|---|---|---|---|
| **K1** | 原生过滤臂不判 `notLink()`：铁索传导伤害（`from_chain:true`）**仍触发**苦无；上游 `notLink()` 为假 → 不触发 | 有属性杀（火杀/雷杀）打横置角色时，链上角色会被额外强制弃牌；上游不会。属**规则可观测**差异（行为已由 `upstream_kuwu_chain_damage_difference` 固定：链上 2 号手牌被弃） | 过滤臂加 `!d.from_chain`（或在 `DamageData` 增加 `linked` 语义位）；改动属内核/宿主面，需与宿主行为 owner 合并后跑全量 |
| **K2** | 原生无「受伤角色有 `he` 牌」filter；上游无牌时不触发 | 目标**只持装备**时：上游强制弃 1 张装备，原生不弃（`upstream_kuwu_filter_gaps` 固定：装备仍在）。目标两区全空时两者结果相同（都无牌可弃），差异仅落在「仅装备」面 | 与 K4 一并处理：弃牌来源改为 `he` |
| **K3** | 原生无 `_notrigger` 概念（`untrigger(false,player)` 无对应状态位） | 上游存在「本次伤害对该角色不触发」的封锁面（如 `character/key/skill.js:11927` 等调用点）；原生该面整体缺失，苦无只是受影响技能之一。**结构差异**，以源码事实断言固定（过滤臂内不得出现 `_notrigger`，若新增须更新本证据） | 评估是否在 `DamageData` 增加 `notrigger: Vec<PlayerId>` 并接到触发分发 |
| **K4** | 弃牌从「受伤角色自选 `he`」简化为「确定性弃首张手牌」 | 选择权与装备区面丢失（与 K2 同源）；AI/录像无差异但**规则面**不同（玩家无法保护关键手牌，也不能用装备顶替） | 宿主 `step_skills_status.rs:666` 改为 `WaitChoice`（观察者=受伤角色，选项=`hand+equips`，`min=max=1`）后弃置 |

**勘误（对照中发现的前批误登记）**：`crates/engine-native/tests/g1_huanle/huanle.rs:1329-1334`
注释称「上游过滤精确杀名」，据此把火/雷杀触发记为差异。核对归档 JS 后确认**该说法不成立**：
杀族牌 `name` 恒为 `"sha"`，火/雷只体现在 `nature` 字段
（`archive/upstream/noname/apps/core/card/standard.js:99-102`：`sha: { nature: ["thunder","fire","kami","ice"] }`），
故 `event.card.name === "sha"` 对火杀/雷杀**同为真**；原生 `CardKind::is_sha_like()`
（Sha/FireSha/LeiSha，`catalog.rs:610-612`）与上游**同口径**。本样本按「对齐」登记，
建议后续把该 g1 注释与说法一并更正（本批不改他人测试文件）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 上游对照定向 | **5 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | **1182 passed / 0 failed / 2 ignored**（1172 → +10，本批两样本各 +5） | [engine-native-tests.log](engine-native-tests.log) |
| 行数门禁 | 8279 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 5 未完成 / 待办（NC08 第⑦层）

- [ ] 差异 K1（铁索传导）上游判定后的对齐（过滤臂加 `from_chain` 判据）
- [ ] 差异 K2/K4（`he` 自选弃牌）合并根治
- [ ] 差异 K3（`_notrigger` 状态位）是否建模
- [ ] 更正 `g1_huanle/huanle.rs:1329-1334` 的「精确杀名」误述（他人文件，未改）
- [ ] 表现/策略面（`audio`、`ai.equipValue`）**不在规则对照范围**（边界声明）
