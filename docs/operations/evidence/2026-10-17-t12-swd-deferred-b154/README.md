# swd deferred 消解批 154：魔化（`mohua2`）——濒死变身撒旦形态（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **魔化**（`character/swd.js`）——前置四技已于批 150–153 落地（`jifeng`/`moyan`/`miedao`/`swd_xiuluo`）
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化——技能装配 + 上限改写）
- 上游回读：`character/swd.js`（`mohua2` 32 行）

## 上游语义与原生接线

| 步 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {player:'dying'}`，`priority:10`，**forced** | `Mohua2`（tag 513）：`Dying` 触发（自己进入濒死；锁定无确认门） |
| ① 移技 | `removeSkill('miles_xueyi')`/`'aojian'`/`'mohua2'` | 按名移除三实例（`SkillToggled(false)` 日志） |
| ② 获技 | `addSkill('moyan')`/`'miedao'`/`'jifeng'`/`'swd_xiuluo'` | 按名授予四实例（幂等；`SkillToggled(true)` 日志） |
| ③ 变身 | `lib.character.swd_satan = ['','qun',4,[四技],['temp']]` + `player.name/name1/name2 = 'swd_satan'` | **体力上限改 4**（形态数值面）；武将名/头像/faction **不迁移**（登记差异：表现为展示面，且新增武将条目会扰动种子化武将分配） |
| ④ 回复 | `player.recover(2)` | `recover(owner, 2)` |
| ⑤ 摸牌 | `player.draw(2)` | `draw_cards(owner, 2)` |

包数据：`shahai-swd-upstream-noname` 中 `swd_miles` 的 `mohua2` 由 `mapping:null` 改为 `mapping:"host:mohua2"`（四前置技已于前批以包级声明入册）；npack 已 rehash。

## 计数

- 武将技能条目：**deferred 3 → 2**（映射 **179/184 → 180/184**）。

## 验证

```
cargo test --offline -p engine-native --test swd_mohua2_transform
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check --offline -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（魔化） | **3 passed / 0 failed**（[targeted.log](targeted.log)）：濒死（1 血受 2 伤）→ 旧三技移除、**撒旦四技在册**、上限改 4、存活、摸 2；对照（未持技：不获撒旦技、上限不变、不摸牌）；**仅触发一次**（技能实例已移除，二次濒死不再变身） |
| engine-native 全量 | **1050 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**（无版本变化）——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 509（tag 513 追加） |

## 登记差异

- **武将名/头像/faction 不迁移**（上游 `player.name = 'swd_satan'` + 新武将条目）：原生以「体力上限 4 + 四技装配」承载形态数值面；不新增武将条目的原因——新增会改变种子化**武将分配**（与批 147 加牌同类的连锁），且该差异属展示面。
- 上游 `priority:10`（濒死窗内优先）未建模（原生按普通时机）。
- 上游未移除「原武将的其它技」（仅显式三技）；原生同口径（按名移除三技）。
