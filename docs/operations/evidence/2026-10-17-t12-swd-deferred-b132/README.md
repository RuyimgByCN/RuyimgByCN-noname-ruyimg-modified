# swd deferred 消解批 132：魂殉志（`hxunzhi`）+ 殉（`hxunzhi2`）（v314 不变，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **魂殉志**（`character/swd.js`）+ 其授予技 `hxunzhi2`
- **规则/存档版本不变**（v314 / 存档 v98；无字段新增——觉醒以**实例禁用**承载，授予走运行时实例）
- 上游回读：`character/swd.js`（`hxunzhi` 66 行 + `hxunzhi2` 13 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 类型 | `enable:'phaseUse'`，`unique` + `awakenSkill`（限定技，`!storage.hxunzhi`） | `Hxunzhi`（tag 488）：出牌阶段主动面（`activate`，`turn.phase==Play && current==owner` 且实例启用） |
| ① 觉醒 | `awakenSkill('hxunzhi')` | 实例 `enabled = false`（引擎主动技入口对禁用实例直接拒绝；无字段新增） |
| ② 万箭 | `useCard({name:'wanjian'}, 全部 canUse 目标（座位序）)` | `use_virtual_card(ArcheryAttack, 全部其他存活角色（座位序）)` |
| ③ 授予 | `addSkill('wusheng')` + `addSkill('paoxiao')` + `addSkill('hxunzhi2')` | 运行时插入三实例（武圣/咆哮取标准目录技；`hxunzhi2` 由包级 `skills` 声明 → `host:hxunzhi2`） |
| 殉 | `hxunzhi2`：`phaseUseEnd` forced → `removeSkill('xunzhi2')` + `player.die()` | 出牌阶段**收束**内联臂（原生无 PhaseEnd 帧）：持有「殉」实例者移除该实例并发射 `Death{player, killer: None}`（直接死亡，非伤害） |

包数据：`shahai-swd-upstream-noname` 中该将的 `hxunzhi` 由 `mapping:null` 改为 `mapping:"host:hxunzhi"`，并新增包级技能声明 `hxunzhi2 → host:hxunzhi2`；npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 21 → 20**（映射 170 → 171）。
- 台账口径（登记表）：**162/184 → 163/184 映射，deferred 20 → 19**。

## 验证

```
cargo test --offline -p engine-native --test swd_hxunzhi_wanjian
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（魂殉志） | **3 passed / 0 failed**（[targeted.log](targeted.log)）：发动 → 万箭齐发（三名其他角色各受 1）→ 授予武圣/咆哮/殉 → 限定技二次发动被拒 → 出牌阶段收束死亡且殉实例移除；两对照（非出牌阶段/未持技拒绝；无殉者阶段收束不死亡） |
| engine-native 全量 | **975 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | `replay_scenarios` 2/2 **零位移**（无版本变化，未重录）——[replay-golden-v314.log](replay-golden-v314.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 486（tag 488/489 追加） |

## 登记差异

- 万箭目标按「全部其他存活角色（座位序）」；上游按 `canUse('wanjian')` 过滤（帷幕/藤甲类目标禁用未逐一建模，同既有虚拟 AOE 口径）。
- 死亡为直接死亡（`Death{killer:None}`，非伤害/非失去体力路径），与上游 `player.die()` 同口径。
