# swd 批 153：修罗（`swd_xiuluo`）——魔化前置技之四（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 范围：**魔化（`mohua2`）前置技** `swd_xiuluo`（撒旦形态四技之四）
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化）
- **定义定位**：liuli `character/swd.js` 仅有翻译与说明文本；**离线包** `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7072` 有完整实现（两仓同源，取离线包定义 ✓）
- 上游说明：准备阶段，你可以弃一张手牌来弃置你判断区里的一张延时类锦囊（**必须花色相同**）

## 上游语义与原生接线

| 面 | 上游（离线包） | 原生 |
|---|---|---|
| 触发 | `trigger {player:'phaseZhunbeiBegin'}`，`direct:true`（可选）；`filter`：判定区有牌 | `Xiuluo`（tag 512）：`PhaseBegin(Judge)` 触发 + 可选确认门；过滤：判定区非空且手牌非空 |
| 选牌 | `discardPlayerCard(player, 2, 'hj', ...)`：一张**手牌** + 一张**判定区**牌，**花色相同** | 两窗：`SelectCards`（手牌，min1/max1，可取消）→ `SelectCards`（判定区**同花色**牌，min1/max1，可取消） |
| 效果 | 两张一并弃置 | 手牌 `discard_from_hand`；判定区移除 + `note_cards_lost`（失去牌事件）+ 入弃堆 |

包数据：包级声明 `swd_xiuluo → host:swd_xiuluo`（属魔化形态，随 `mohua2` 批次入将）；npack 已 rehash。

## 计数

- swd 包 deferred 计数不变（前置技落地）；台账口径仍 **179/184**。
- 登记项 `T12-SWD-MOHUA2-PREREQ`：`jifeng` ✅、`moyan` ✅、`miedao` ✅、`swd_xiuluo` ✅ → **4/4 完成**（下一批可落 `mohua2` 本体）。

## 验证

```
cargo test --offline -p engine-native --test swd_xiuluo_judge_clear
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（修罗） | **3 passed / 0 failed**（[targeted.log](targeted.log)）：判定区【乐不思蜀】+ 同花色手牌 → 确认 → 选手牌 → 判定牌窗含该同花色牌 → 双弃（判定牌入弃堆、手牌离手）；两对照（判定区无牌不开窗、未持技不开窗） |
| engine-native 全量 | **1047 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**（无版本变化）——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 508（tag 512 追加） |

## 登记差异

- 上游为**单窗双选**（手牌+判定区，花色一致约束在 `filterButton`）；原生拆为两窗（手牌 → 同花色判定区）。
- 上游 `ai`（按钮估值）未迁移。
- 定义取自离线包（liuli 角色文件仅有翻译/说明）——两仓同源，已在证据中标注出处。
