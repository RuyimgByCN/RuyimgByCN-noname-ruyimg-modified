# swd deferred 消解批 142：天书（`swdtianshu`）——获取目标武将的一个技能（字段批 v317/存档 v101，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **天书**（`character/swd.js` 主技 + 组技 `swdtianshu_remove`）
- **规则 v316→317、存档 v100→101**（`PlayerState` 追加天书两字段）
- 上游回读：`character/swd.js`（`swdtianshu` 120 行 + `swdtianshu_remove` 13 行）

## 上游语义与原生接线

| 步 | 上游 | 原生 |
|---|---|---|
| 类型 | `enable:'phaseUse'`，`unique:true`；`filterCard`=锦囊牌，`filter`=有锦囊手牌 | `SwdTianshu`（tag 500）：出牌阶段主动面（`activate`；实例启用 + 有锦囊手牌） |
| ① 用牌消耗 | `phaseUse` 用牌（`filterCard`） | 机器内 `SelectCards`（自己的锦囊手牌，min1/max1，不可取消）→ `discard_from_hand`（登记差异：原生主动技入口不带牌，成本在机器内收） |
| ② 目标 | `filterTarget`：其武将（`lib.character[name][3]`）存在**非 unique** 且自己未持有的技能 | `SelectPlayers`（存活且 `swdtianshu_skills` 非空） |
| ③ 选技能 | 对话框列出技能（`createDialog`），点击选择 | `PickTag`（**候选下标**；应答时按同序重枚举映射到技能定义——定义 id 可能超出 u8） |
| ④ 获取 | `addAdditionalSkill('swdtianshu', link)`（永久） | 运行时插入技能实例（无到期）+ 记录 `swdtianshu_from`/`swdtianshu_granted` |
| 移除 | 组技 `swdtianshu_remove`：`global dieAfter`，来源阵亡 → `removeAdditionalSkill` | `DeathMachine` 首步内联臂：遍历持有者，`swdtianshu_from == 阵亡者` → 移除实例并清两字段 |

包数据：`shahai-swd-upstream-noname` 中该将的 `swdtianshu` 由 `mapping:null` 改为 `mapping:"host:swdtianshu"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 11 → 10**（映射 180 → 181）。
- 台账口径（登记表）：**172/184 → 173/184 映射，deferred 10 → 9**。

## 验证

```
cargo test --offline -p engine-native --test swd_swdtianshu_gain
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（天书） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：弃锦囊牌 → 选 1 号（关羽）→ 选其一个技能 → **永久获得**（实例在册、锦囊已弃、`swdtianshu_from=1`）→ 1 号阵亡后**移除**并清字段；对照（无锦囊不可发动、未持技/非出牌阶段拒绝）；两字段存档往返 |
| engine-native 全量 | **1015 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 重录 + 版本回退探针（317→316 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v317.log](replay-golden-v317.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 497（tag 500 追加） |

## 登记差异

- 原生技能元数据无 `unique` 标记，故天书候选**不排除限定技**（上游 `!lib.skill[x].unique`）；候选来源为目标**武将技能表**（标准 + 扩展武将段）。
- 用牌消耗在机器内选牌弃置（原生主动技入口不带牌参数）。
- 技能候选以「下标 Tag + 应答重枚举」承载（原生 Tag 载荷为 u8，技能定义 id 可为 u16）。
- 移除时机为 `DeathMachine` 首步（上游 `dieAfter`——原生死亡帧为 BeforeTriggers→…→Intrinsic，语义等价）。
