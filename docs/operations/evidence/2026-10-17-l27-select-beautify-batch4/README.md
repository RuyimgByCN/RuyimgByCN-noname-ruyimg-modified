# L27 选将美化 批 4：诈降（`sbzhaxiang`）（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L27 选将美化**（`extension/选将美化/extension.js`）
- 本批：技能 `sbzhaxiang`（诈降，tag 521）
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化——复用 `cards_used_play_phase` 等既有状态）
- 台账：L27「部分重构（批 4/N）」

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 条件 | `player.countUsed() < player.getDamagedHp()`（本阶段已用牌数 < 已损失体力） | `host::sbzhaxiang_active(core, p)`：`turn.cards_used_play_phase < (max_hp - hp).max(0)` |
| 无距离 | `mod.targetInRange` 返回 true | 出牌校验：`feiren_range_free = sha_range_free \|\| sbzhaxiang_free`（并入既有距离豁免链） |
| 不限次 | `mod.cardUsable` 返回 Infinity | 出杀次数校验：`lingshi_extra \|\| sbzhaxiang_extra`（并入既有额外额度链） |
| 直伤 | `trigger {player:'useCard'}` 锁定：`countUsed() <= getDamagedHp()` → `directHit` 全场 | 杀机器：`sbzhaxiang_direct_hit`（用牌数 ≤ 已损失体力）→ 跳过防具判定与出闪询问（原生直伤面仅【杀】有闪避窗口） |
| 额外摸牌 | `sbzhaxiang_draw`：`phaseDrawBegin2` 且 `!numFixed && getDamagedHp()>0` → `trigger.num += getDamagedHp()` | 摸牌收束臂：`sbzhaxiang_draw = (max_hp - hp).max(0)`（并入 灭道/灵矢/神威 同链） |
| 展示 | `sbzhaxiang_mark`/`mark2` 文本 | 不迁移（登记差异） |

包数据：`select-beautify-upstream-liuli` 增加 `sbzhaxiang` 声明（生成器同步）。

## 验证

```
cargo test --offline -p engine-native --test l27_sbzhaxiang_batch4
cargo test --offline -p engine-native --test l27_new_meibu_batch3 / l27_new_zhixi_batch2 / l27_xiefang_batch1
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（诈降） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：已损失 2 → 摸牌阶段摸 **4**（2+2）；**第 2 张杀仍可出**（不限次）；两对照（未持技：正常摸 2、第二张杀被拒） |
| 定向（批 1–3 回归） | 全部通过（[targeted-regression.log](targeted-regression.log)） |
| engine-native 全量 | **1076 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **88/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 517（tag 521 追加） |

## 登记差异

- 直伤面**仅【杀】**（上游 `directHit` 对所用牌全场生效；原生仅【杀】有闪避/防具窗口）。
- 上游 `numFixed`（固定摸数）门控未建模：原生在常规摸牌链上叠加（跳摸分支之前已排除）。
- 展示标记（`sbzhaxiang_mark`/`mark2`）与 `aiOrder`/`skillTagFilter` 未迁移。
- 其余 37 个在用技能待批次实施；选将 UI 主面为混淆代码。
