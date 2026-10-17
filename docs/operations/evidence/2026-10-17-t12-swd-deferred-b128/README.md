# swd deferred 消解批 128：诛害（`zhuhai`）——结束阶段角色造成过伤害则用杀（v312 不变，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **诛害**（`swd_muyun` 系，定义在 `character/refresh.js:13643`——**外部包技能但本仓有定义**）
- **规则/存档版本不变**（v312 / 存档 v96；无字段新增）
- 上游回读：`character/refresh.js`（`zhuhai` 19 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {global:'phaseJieshuBegin'}`，`direct:true` | `SwdZhuhai`（tag 483）：`PhaseBegin(End)` 全局触发 + 可选确认门 |
| 条件 | 结束阶段角色存活 且 `getStat('damage')`（本回合造成过伤害）且 `targetEnabled({name:'sha'})`（射程内）且自己有【杀】 | 结束阶段角色存活且 `turn_damage > 0`（回合伤害统计，回合开始清零）且自己手中有 `is_sha_like` 牌且在攻击距离内 |
| 效果 | `chooseToUse`：对自己手中一张【杀】使用（目标固定为结束阶段角色） | `SelectCards`（手中杀类牌，min1/max1，可取消）→ `CardUsed`（实体牌，目标=结束阶段角色）→ 杀结算 |

包数据：`shahai-swd-upstream-noname` 中该将的 `zhuhai` 由 `mapping:null` 改为 `mapping:"host:zhuhai"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 25 → 24**（映射 166 → 167）。
- 台账口径（登记表）：**158/184 → 159/184 映射，deferred 24 → 23**。

## 验证

```
cargo test --offline -p engine-native --test swd_zhuhai_end_phase_sha
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（诛害） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：结束阶段角色本回合造成过伤害 → 确认 → 选杀 → 对其用杀造成 1 伤且杀离手；三对照（未造成伤害不开窗、手中无杀不开窗、未持技不开窗） |
| engine-native 全量 | **960 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | `replay_scenarios` 2/2 **零位移**（无版本变化，未重录）——[replay-golden-v312.log](replay-golden-v312.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 480（tag 483 追加） |

## 登记说明（外部技能可达性勘察）

本轮勘察确认 swd 包 deferred 名单中若干「本仓 swd.js 未定义」的技能**在其它角色文件中有定义**，属「外部包技能但本仓可读」，可继续实现：

| 技能 | 定义位置 |
|---|---|
| `gongxin`（攻心） | `character/extra.js:11396` |
| `songci`（颂词） | `character/sp.js:22323` |
| `tianxiang`（天香） | `character/shenhua.js:6501`（另有 `sb.js:3292`） |
| `zhenlie`（贞烈） | `character/yijiang.js:11678` |
| `zhuhai`（诛害，本批已落） | `character/refresh.js:13643` |

仅 `niepan` 一项确认为**跨来源技能依赖**（庞统限定技，见 `T11-BOSS-PANGTONG-EXTERNAL`，已在批 118 落地为主目录技能）。
