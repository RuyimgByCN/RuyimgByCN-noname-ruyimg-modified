# swd 批 152：灭道（`miedao`）——魔化前置技之三（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 范围：**魔化（`mohua2`）前置技** `miedao`（`character/swd.js`，组技两面 `miedao1`/`miedao2`）——撒旦形态四技之三
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化）
- 上游回读：`character/swd.js`（`miedao` 6 行 + `miedao1` 10 行 + `miedao2` 10 行）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 摸（`miedao1`） | `trigger {player:'phaseDrawBegin'}` 锁定；`filter`：体力未满；`trigger.num += maxHp - hp` | **内联于摸牌收束**：`miedao_draw = maxHp - hp`（持 `miedao` 且体力未满）并入既有摸牌基数修正链（灵矢/神威/脉轮同表） |
| 弃（`miedao2`） | `trigger {player:'phaseDiscardBegin'}` 锁定；`filter`：体力未满；`chooseToDiscard(maxHp-hp,'he',true)`（强制） | `Miedao`（tag 511）：`PhaseBegin(Discard)` 锁定触发 → `SelectCards`（手牌+装备，min=max=min(已损失体力, 可用数)，不可取消）→ 逐张弃置（手牌 `discard_from_hand`；装备撤技后入弃堆） |

包数据：包级声明 `miedao → host:miedao`（属魔化形态，随 `mohua2` 批次入将）；npack 已 rehash。

## 计数

- swd 包 deferred 计数不变（前置技落地）；台账口径仍 **179/184**。
- 登记项 `T12-SWD-MOHUA2-PREREQ`：`jifeng` ✅（批 150）、`moyan` ✅（批 151）、`miedao` ✅（本批）、`swd_xiuluo` 待落 → **3/4**。

## 验证

```
cargo test --offline -p engine-native --test swd_miedao_draw_discard
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（灭道） | **3 passed / 0 failed**（[targeted.log](targeted.log)）：体力差 2 → 摸牌 **+2**（共 4 张）、弃牌阶段出现**不可取消**的「强制弃 2」窗且弃置生效；两对照（体力已满：正常摸 2 且无灭道窗；未持技：正常摸 2） |
| engine-native 全量 | **1044 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**（无版本变化）——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 507（tag 511 追加） |

## 登记差异

- 强制弃置在可用牌不足「已损失体力」时按「弃全部」处理（上游 `chooseToDiscard(...,true)` 不足时行为未定义）。
- 摸牌面为内联收束臂（上游为 `phaseDrawBegin` 触发子技）；两者在「跳摸路径不生效」上同口径（内联臂位于跳摸分支之后）。
- 上游 `check`/AI 面未迁移。
