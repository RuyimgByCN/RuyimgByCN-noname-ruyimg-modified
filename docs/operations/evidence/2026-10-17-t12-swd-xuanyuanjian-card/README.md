# swd 批 147：轩辕剑牌实体入包 + 8 项种子敏感用例重导（T12-SWD-XUANYUANJIAN-CARD 收口，2026-10-17）

- 日期：2026-10-17
- 范围：批 146 登记的 **牌实体**（`packages/extensions/shahai-swd-upstream-noname/assets/cards.json` 增 `xuanyuanjian`）+ 因**种子化牌堆/武将分配**变化而受影响的 8 项用例重导
- **规则/存档版本不变**（v320 / 存档 v104；批 146 已含牌种与字段）
- 关联：批 146 引擎侧（牌种/三子技/装备钩/取剑机/使用门）——[证据](../2026-10-17-t12-swd-deferred-b146/README.md)

## 变更

1. 包内新增牌实体 `{"name":"xuanyuanjian","kind":"xuanyuanjian","suit":"spade","rank":1}`（包内 89 → **90** 张）；npack 已 rehash。
2. 包内 `xuanyuan` 由 deferred 改为 `mapping:"host:xuanyuan"`（牌实体就位后可达）。
3. 8 项种子敏感用例重导（加牌改变同 seed 的 RNG 序列 → 牌堆/武将分配变化）：

| 用例 | 处置 |
|---|---|
| `swd_extra_cards_disposition_guard` | 计数冻结 89 → 90（注明批 147） |
| `swd_cards_alignment_guard_b29` | 同上 |
| `swd_cards_trick_b42_yuchan_autoviewas` | 同上 |
| `swd_hufu_self_dying_only` | 目录张数冻结 base+89 → base+90 |
| `swd_real_host_full_lifecycle` | 同上（两处：装载与卸载后恢复） |
| `swd_generals_b63_lianwu_yuchen_duishi_xielei` | **布阵健壮化**：红杀实体改为**任意区**（牌堆/手牌/弃牌堆）取用——原实现假定红杀仍在牌堆 |
| `swd_generals_b69_lingxin_yueren` | **布阵健壮化**：第二张红桃显式排除已压顶的第一张（原实现可能重复取同一张） |
| `lingwu_inserts_extra_play_phase_when_used_reaches_hp` | **布阵健壮化**：出牌前摘除**他方技能**（受击方反击技可能令 p0 失血/致死，与本用例无关）；出牌后复位体力并重授灵武 |

## 计数

- swd 包 deferred：**6 → 5**（映射 **176/184 → 177/184**）。
- 登记项 `T12-SWD-XUANYUANJIAN-CARD` → **resolved**（open 35 → 34）。

## 验证

```
cargo test --offline -p engine-native --test swd_xuanyuanjian_card
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（轩辕剑全链） | **4 passed / 0 failed**（[targeted.log](targeted.log)）：装备入槽 +1 护甲；加伤 +1 且无「轩辕」反噬；对照持「轩辕」不反噬；**结束阶段弃黑桃从弃牌堆取剑装备**；字段存档往返 |
| engine-native 全量 | **1029 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**（swd 包不影响标准 14 场景；无版本变化）——[replay-golden-v320.log](replay-golden-v320.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 说明

- 加牌引起的**种子敏感**连锁是本批的主要工作量：牌堆与武将分配都由同 seed 的 RNG 派生，故新增一张牌即改变同 seed 的整局布阵。三处布阵健壮化（任意区取牌、显式排除、摘除他方技能）使这些用例**不再依赖具体布阵**，后续加牌不必重复重导。
