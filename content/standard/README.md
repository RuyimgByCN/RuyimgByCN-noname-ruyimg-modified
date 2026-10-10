# standard content (DSL)

Official-ish sample skills for noname-next (**M0 surface + M1 pack**).

## Packages

| File | Notes |
|---|---|
| `standard_m1.skill` | **Canonical pack** — auto-loaded on `StartGame` |
| `standard_m0.skill` | Smaller subset for focused tests |
| `*.skill` | Individual samples |

```bash
cargo run -q -p content-cli -- pack \
  content/standard/standard_m1.skill \
  --characters content/standard/characters.json \
  --metadata content/standard/metadata.json \
  --out content/standard/standard.npack.json \
  --package-id standard
```

## Skill def ids (`skill_def` / catalog)

| id | Skill | Type |
|---|---|---|
| 1 | 奸雄 | 受伤后可获得仍在处理区的伤害实体牌 |
| 2 | 马术 | modifier distance_from -1 |
| 3 | 奇策 stub | modifier attack_range +1 |
| 4 | 闭月 | 结束阶段可摸 1 |
| 5 | 反馈 | 受伤后可从伤害来源处获得一张牌；自伤时仅可获得装备区牌 |
| 6 | 天妒 | `judgment_resolved` 后可获得自己的最终实体判定牌 |
| 7 | 刚烈 | 受伤后可判定；非红桃时，伤害来源弃两张手牌，否则受到 1 点伤害 |
| 10 | 制衡 | 出牌阶段限一次，弃置手牌/装备区任意张牌并摸等量牌 |
| 3165045964 | 救援 | 主公技；其他吴势力角色以【桃】响应自己濒死时额外回复 1 |
| 11 | 仁德 | 出牌阶段可给其他角色任意张手牌；本阶段首次累计给出至少两张时回复 1 |
| 2377283906 | 激将 | 主公技；使用或响应【杀】时依座次请求其他蜀势力角色代为打出 |
| 12 | 苦肉 | 出牌阶段可反复失去 1 点体力并摸两张牌 |
| 13 | 青囊 | 每阶段一次弃一张手牌，令一名受伤角色回复 1 |
| 2961041949 | 急救 | 回合外将一张红色手牌或装备牌当桃使用（华佗） |
| 14 | 集智 | 使用普通锦囊牌后可摸一张；延时锦囊不触发 |
| 15 | 突袭 | 摸牌阶段可改为从一至两名其他角色处各获得一张手牌 |
| 16 | 武圣 | `view_as sha play_phase red` |
| 17 | 龙胆 | 闪当杀 + 杀当闪（响应） |
| 18 | 无双 | 杀需两张闪；决斗中对手每轮需两张杀（吕布） |
| 19 | 鬼才 | 判定前改判（司马懿） |
| 20 | 英姿 | `modifier draw_count add 1`（周瑜） |
| 36 | 反间 | 每阶段一次：目标猜花色并获得自己一张手牌，花色不符则受到自己造成的 1 点伤害 |
| 21 | 连营 | `cards_moved` + `owner_from_hand_empty`，可选摸 1（陆逊） |
| 22 | 枭姬 | `cards_moved` + `owner_from_equip_per_card`，每张失去的装备可选摸 2（孙尚香） |
| 23 | 咆哮 | 出杀无次数限制标记（张飞；引擎本无限制） |
| 24 | 倾国 | `view_as shan any black`（甄姬） |
| 25 | 结姻 | 每阶段一次弃两张手牌，自己与一名受伤男性角色各回复 1 |
| 26 | 遗计 | 每受到一点伤害，可将牌堆顶两张牌任意分配给存活角色 |
| 3733125857 | 流离 | 成为【杀】目标时，可弃一张牌并转移给攻击范围内的另一名合法角色 |
| 29 | 裸衣 | 摸牌阶段可少摸一张；本回合【杀】和【决斗】伤害 +1 |
| 2999929189 | 裸衣临时效果 | 来源的【杀】或【决斗】伤害 +1 |
| 3681607931 | 空城 | 无手牌时不能成为【杀】或【决斗】的目标（诸葛亮） |
| 2286640837 | 奇才 | `card_distance_unlimited(trick)` |
| 2669566023 | 飞影 | `distance_to add 1` |
| 2737456606 | 妄尊临时效果 | 每层令手牌上限 -1 |
| 3349403784 | 将驰临时效果 | 禁止使用/响应【杀】，且【杀】不计入手牌上限 |
| 2751572143 | 将驰 | 摸牌阶段结束时选择摸牌限制【杀】，或弃牌强化【杀】 |
| 3515984232 | 神戟 | 【杀】目标数上限 +2、次数上限 +1 |
| 4273235044 | 清剿 | 主公技；本回合伤害过其他群势力角色，结束阶段摸一张 |
| 3872761084 | 耀武 | 红色【杀】造成伤害前，来源强制摸 1 或回复 1 |
| 4213747779 | 狂斧 | 每个出牌阶段首次以【杀】伤害其他存活角色后，按双方当前体力摸 2 或失去 1 点体力 |
| 3504783479 | 淑慎 | 每回复 1 点体力，可独立选择一名其他角色摸牌；空手目标改摸 2 张 |

## 批量生成（模板）

```bash
# 从 skill_catalog.toml 展开模板技能草稿
python3 content/tools/gen_skill_dsl.py
python3 content/tools/gen_skill_dsl.py > /tmp/batch.skill
```

- **模板** = 一类机制（`damage_draw` / `view_as` / `modifier_add` / …）
- **catalog 一行** = 一个技能实例（id + 参数）
- 复杂技能（制衡 Choice、鬼才、无双）仍手写在 `standard_m1.skill`

### DSL 过滤器 / modifier（扩展）

| filter / target | 含义 |
|---|---|
| `owner_is_target` / `owner_is_source` | 事件主目标 / 来源 |
| `always` | 总是（慎用） |
| `owner_alive` | 全局事件结算后技能拥有者仍存活（如功獒） |
| `owner_hand_empty` | 自己是事件主目标且手牌为空 |
| `owner_from_hand` / `owner_from_equip` | `CardsMoved` 从手/装区离开 |
| `owner_from_hand_empty` | 从手牌离开且手牌已空（连营） |
| `owner_from_equip_per_card` | 从装备区离开；按失去的装备张数逐次触发（枭姬） |
| `owner_source_ordinary_trick` | 自己使用普通锦囊（集智） |
| `owner_dying_other_tao_responder_group_wu` | 其他对应势力角色以【桃】响应濒死拥有者 |
| `owner_sha_target_can_discard_redirect_to_other_in_attack_range` | 当前【杀】目标可弃牌并转移给自己攻击范围内的合法角色 |
| `once_per_turn` / `owner_source_once` | 每回合至多 1 次（`turn_fire_count`） |
| **modifier** `draw_count add N` | 摸牌阶段摸牌数 +N（英姿） |
| **modifier** `sha_target_limit add N` | 【杀】可额外指定 N 个目标（神戟、天义） |
| **modifier** `card_target_cancel(selector) override owner_hand_empty` | 无手牌时禁止匹配牌指定拥有者（空城） |

## Characters (30)

曹操/司马懿/夏侯惇/张辽/…/貂蝉/华雄/标潘凤 — see `engine-core` `STANDARD_CHARACTERS`.

## Cards (M1)

Basics + equips + **无中 / 过河 / 顺手 / 决斗 / 南蛮 / 万箭 / 桃园 / 五谷**.  
酒：受伤回血；满血获得 JIU mark（下一张杀 +1 伤）。

## View-as

```text
skill wusheng(16) {
  view_as sha play_phase red;
}
// ActivateSkill { skill, targets, card_targets: [red_card] }
```

### 响应与点选

- **杀**：目标 `RespondToSha`（出闪或取消受伤）；龙胆可将 **杀当闪**
- **过河**：`PickTargetCard` mode=discard
- **顺手**：`PickTargetCard` mode=take
- **制衡**：`ZhihengDiscard` 或多张 `card_targets` 弃 N 摸 N

## Tests

```bash
cargo test -p engine-core --test phase_m1_standard_content
cargo test -p engine-core --test phase_m0_unified_skills
cargo test --workspace
```
