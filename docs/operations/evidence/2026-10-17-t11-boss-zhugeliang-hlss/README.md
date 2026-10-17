# T11 Boss 诸葛亮族批 115：hlss 星系统（禳星 / 隐月 / 隐月·判，字段批 v303/存档 v88，2026-10-17）

- 日期：2026-10-17
- 范围：`boss_zhugeliang` 四技中的三技（**禳星 `hlss_xiangxing`、隐月 `hlss_yueyin`、隐月·判 `hlss_yueyin2`**）——四技之二/三
- **规则 v302→303、存档 v87→88**（`PlayerState` 尾部追加 `hlss_stars`/`hlss_count`/`hlss_damage`）
- 上游回读：`archive/upstream/noname-liuli/noname/character/extra.js`（`hlss_xiangxing` 296–455、`hlss_yueyin` 181–295、`hlss_yueyin2` 内联于同段；Boss 条目 `mode/boss.js:1303`）

## 语义与接线

| 技 | 上游语义 | 接线（原生） |
|---|---|---|
| 禳星 `hlss_xiangxing`（tag 463） | `init`：3 枚星、计数 0；`damageEnd`/`loseHpEnd` 累计伤害/失血（`damageBegin4 num++` 由隐月提供）；计数 ≥3：隐月免除条件满足 → 不失星不放雷，否则失一星（3→0）并**向其他角色分配 3 点雷伤**（全场随机 1 名 +1；随机「敌人」两名各 +1，无敌人时退化为全场；逐目标结算）；星归零 → `hp = maxHp` 并觉醒 | `HlssXiangxing`（Damage 触发 + `extra_events [LoseHp]`）：累计 → 免除判定（当前星数对应位：3=雷/2=火/1=锦囊）→ 失星清计数与记录 → 雷伤分配（`ctx.core.rng` 决定性；「敌人」= `is_boss_side` 异侧，本侧无敌人时退化为全场；**登记差异**：上游按 `isFriendsOf` 态度，原生按 Boss 阵营）→ 星归零回满并禁用禳星实例 |
| 光 `hlss_xiangxing_hp`（子技，内联） | `changeHp`/`loseMaxHp`/`gainMaxHp` 且 `hp < maxHp` → `hp = maxHp` | `kernel::hlss_light_refill`：伤害/失去体力提交点、**濒死判定前**调用（星未耗尽者不会进入濒死）；星归零（技能禁用）后不再回满 |
| 隐月 `hlss_yueyin`（tag 464） | `damageBegin4`：`trigger.num++`（受到的伤害 +1）；`damage`：记录本次伤害的 `nature` 与 `get.type2(card)` 供禳星免除 | `HlssYueyin`（**DamageBefore** 前置窗）：`current_damage_mut().amount += 1` 写回 + 记录位图（雷 1/火 2/锦囊 4）。**登记差异**：上游在 `damage`（结算后）记录，原生同前置窗记录（被防止的伤害在窗口内已记录）；上游 `type2` 的 `trick`/`delay` 两类键合并为「锦囊牌伤害」一位 |
| 隐月·判 `hlss_yueyin2`（tag 465） | `mod.judge`：阶段判定 `true→false`、`false→null`（两臂皆不生效）；`turnOverBefore` 且未被翻面 → 取消翻面 | 判定不生效：并入 `apply_judgement` 的 `shenyi_voids` 臂（神裔同口径，牌去向不变）；翻面取消：`HlssYueyin2` 写回 `TurnOverBefore` 取消位 |

装配初始化：`apply_general_assembly` 见到武将技能含 `hlss_xiangxing` 时置 `hlss_stars = 3`（上游 `init` 同口径）；新增测试工具 `debug_assemble_general_def`（按目录 def 装配扩展/Boss 将）。

## 字段批

- `PlayerState.hlss_stars: u8` / `hlss_count: u8` / `hlss_damage: u8`（位图）；入 `canonical_hash`。
- 存档：`archive.rs` 尾部追加；`ARCHIVE_FORMAT_VERSION` 87→**88**；`STANDARD_RULESET_VERSION` 302→**303**。
- 14 golden 重录（[replay-golden-v303.log](replay-golden-v303.log)）；版本回退探针（303→302 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）。
- `d4_boss_b0_archive_format_v79_rejects_v78_without_migration` 断言同步 87→88；键表守护 `host_behavior_keys_bijective` 更新为 **462**（tag 463–465）。

## 验证

- **定向回归** `crates/engine-native/tests/t11_boss_zhugeliang_hlss.rs` **5 passed / 0 failed**（[targeted.log](targeted.log)）：
  1. `hlss_assembly_initializes_stars_and_light_refill`——装配 3 星；隐月 1 点伤害变 2 点且**光回满**（4/4）；计数按结算值累加；第 4 点触发失星并分配 **3 点雷伤**给其他角色（随机 1 + 随机敌 2，持有者不吃自己的雷伤）；
  2. `hlss_yueyin_skip_prevents_star_loss`——先受雷伤记录雷位，随后普通伤害达阈值 → 三星免除：不失星、不放雷、计数保留；移除隐月实例后同流程照常失星；
  3. `hlss_zero_stars_awakens_and_stops_refill`——三次普通伤害耗尽三星：禳星禁用、记录清空，此后伤害照常（4→1，不再回满）；
  4. `hlss_yueyin2_blocks_turnover_and_voids_delayed_trick`——翻面被取消（`debug_flip` 返回 false），移除后照常翻面；乐不思蜀在黑桃判定下**不跳阶段**，移除隐月·判后照常跳阶段；
  5. `hlss_with_infinite_hp_keeps_stars`——无限体力哨兵与星系统并存：体力不变、星照常耗尽。
- **全量**：engine-native **906 passed / 0 failed / 1 ignored**（exit 0；1 ignored＝既有诊断探针）——[engine-native-tests.log](engine-native-tests.log)。
- **装载三包**：**66/0/0**——[deps-tests.log](deps-tests.log)。
- **14 golden**：`replay_scenarios` 2/2（按上述依据重录）。
- **Gate 1b**：见 [gate1b-all-targets.log](gate1b-all-targets.log)（独立 `CARGO_TARGET_DIR`）。

## 剩余工作（`T11-BOSS-ZHUGE-LIANG` 仍 open）

1. **起风 `hlss_fengqi`**（准备阶段从牌堆/目录选一张未用过的锦囊并使用；含 `hlss_fengqi2`「无懈不可响应」）与 **改命 `hlss_gaiming`**（`judgeBefore`：亮 7 张选 1 作为判定结果，其余弃置）。
2. **Boss 装配**：`boss_zhugeliang` 条目（shen/hp Infinity/四技）+ Boss 关卡接入（D4 `BossStageState`）+ 禁将/乱入登记；`ylyg*`/`mbcm*` 外部依赖按台账继续登记差异。
3. 龙血玄黄本体 `mbqianlong`/`_CMeffects` 与 `mb_caomao` 装配。

## 影响

- 进度主稿新增切片行；规则基准「近期版本」补 v303 与本批语义；台账 L29 行补记；登记表 `T11-BOSS-ZHUGE-LIANG` 备注补记（保持 open）。
- 无内容包变化（本批只落行为与字段；Boss 包待四技齐备后建）。
