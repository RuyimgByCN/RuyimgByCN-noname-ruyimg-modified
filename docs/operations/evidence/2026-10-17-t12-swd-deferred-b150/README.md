# swd 批 150：疾风（`jifeng`）——单目标牌多目标 + 多目标概率失误（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 范围：**魔化（`mohua2`）前置技之一** `jifeng`（`character/swd.js`，29 行）——魔化后撒旦形态的四技之一
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化——目标上限豁免 + RNG 判定）
- 上游回读：`character/swd.js`（`jifeng` 全量）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| mod | `selectTarget`：`range[0]==1&&range[1]==1` 且（`card.name=='sha'` 或锦囊且其 `selectTarget` 为 [1,1]）→ `range[1]=Infinity` | `PlayCard` 校验：持 `jifeng` 时【杀】族 `max_targets = usize::MAX`（并入既有 天弦/狂笑/方天/抗yin2/飞刃 上限计算） |
| 触发 | `trigger {player:'useCardToBefore'}`，priority 6，**forced** | `Jifeng`（tag 509）：`CardUsed` 结算窗（AfterTriggers）锁定触发 |
| 过滤 | `event.targets.length>1` 且为【杀】或单目标锦囊 | 同口径：自己使用、未取消、目标数 >1、牌种为杀类或 `jifeng_single_target_trick`（决斗/顺手/拆桥/乐/兵/借刀） |
| 效果 | `Math.random() > (1.3 + n/5)/n` → `trigger.cancel()`（整牌取消，popup「失误」） | 千分位 RNG：`cancel_permille = 1 - (1.3 + n/5)/n`；`gen_range(1000) < permille` → 写回 `cancelled = true`（Intrinsic 前取消整牌结算）+ 日志「疾风·失误」 |

包数据：包级声明 `jifeng → host:jifeng`（该技属魔化后撒旦形态，包内暂无武将条目；随 `mohua2` 批次一并入将）；npack 已 rehash。

## 计数

- swd 包 deferred 计数不变（`jifeng` 原不在包内技能表——属**前置技落地**）；台账口径仍 **179/184**。
- 登记项 `T12-SWD-MOHUA2-PREREQ`（本批新建）追踪魔化前置：`jifeng` ✅ / `moyan` / `miedao` / `swd_xiuluo` 待落。

## 验证

```
cargo test --offline -p engine-native --test swd_jifeng_multishot
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（疾风） | **3 passed / 0 failed**（[targeted.log](targeted.log)）：持技者可用一张【杀】指定两名目标（未持技被拒）；多目标多次尝试**同时出现命中与整牌失误样本**（概率判定生效）；单目标 10 seed 全部必命中（目标数 ≤1 不判定） |
| engine-native 全量 | **1038 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**（无版本变化）——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 505（tag 509 追加） |

## 登记差异

- **仅【杀】族支持多目标**：单目标锦囊的多目标扩展未落地（原生其机器为单目标结算：决斗/顺手/拆桥/乐兵/借刀均取 `targets[0]`），故 mod 的锦囊面登记为待做（其触发过滤已按同口径判定，实际只在【杀】上生效）。
- 概率判定以千分位 RNG 近似（上游 `Math.random()` 连续分布；`cancel_permille` 四舍五入）。
- `priority:6` 时序优先级未建模（原生按普通时机）。
