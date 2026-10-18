# L19 末日乱斗 批 1：誓师（`shishi`）+ 循梦（`xunmeng`）（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L19 末日乱斗**（`extension/末日乱斗/extension.js`，145 KB；G0 见 [G0 证据](../2026-10-17-remaining-sources-g0/README.md)）
- 本批：转换包 `packages/extensions/doomsday-brawl-upstream-liuli`（生成器 `gen_doomsday_brawl_package.rs`）+ 2 个技能
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化）
- 台账：L19 由「G0 完成」→ **部分重构（批 1/N）**

## 上游语义与原生接线

| 技能 | 上游 | 原生 |
|---|---|---|
| 誓师 `shishi`（166 B） | `trigger {source:'dieAfter'}` 锁定——`player.gainMaxHp(1)` + `player.recover()` | `Shishi`（tag 523）：`Death` 触发（`killer == owner`）→ `max_hp += 1` + `recover(1)` |
| 循梦 `xunmeng`（327 B） | `trigger {source:'damageBegin'}` 锁定——`event.card.name=='sha'` 且父事件非 `_lianhuan*` → `trigger.num++`；`player.hp>1` 时 `loseHp()` | `Xunmeng`（tag 524）：**`DamageBefore` 前置窗**（与 隐月 同窗）→ `current_damage_mut().amount += 1`；自己体力 >1 时发 `LoseHp`（无来源、不可防止） |

> 触发窗修正记录：首版误用 `EventKind::Damage` + `Timing::First`（伤害**结算**阶段，改量无效）→ 改为 `DamageBefore`（伤害**前置**窗）后 +1 生效；测试即刻反映（3/3 通过）。

包数据：`doomsday-brawl-upstream-liuli`（`assets/cards.json` 两条技能声明 + `assets/config.json`）；deferred 记录：无尽模式配置面（`_wujinmoshi*`，含 `player==game.me` 人类位判定）、乱斗模式框架（24 处武将引用、`chanceIdentity*` 多为注释）、其余技能批次。

## 验证

```
cargo test --offline -p engine-native --test l19_brawl_batch1
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（批 1） | **3 passed / 0 failed**（[targeted.log](targeted.log)）：誓师——击杀 3 号后**上限 +1 且回复 1**；循梦——【杀】造成 **2 点**伤害（+1）且自己**失去 1 点体力**；对照（未持技：1 点伤害、自己不失去体力） |
| engine-native 全量 | **1081 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **91/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 520（tag 523/524 追加） |

## 登记差异

- 循梦的「父事件非铁索」以 `DamageData.from_chain` 判定（上游比较 `event.parent.name`）。
- 誓师的 `gainMaxHp` 直接改 `max_hp`（原生无独立「获得体力上限」事件/日志）。
- 无尽模式配置面（`_wujinmoshi*`）与乱斗模式框架未迁移；`chanceIdentity*` 系列上游多为注释。
- 其余 64 个在用技能待批次（G0 实测中位 0.5 KB）。
