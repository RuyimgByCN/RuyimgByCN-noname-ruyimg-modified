# L27 选将美化 批 1：撷芳（`xiefang`）（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L27 选将美化**（`extension/选将美化/extension.js`；G0 见 [G0 证据](../2026-10-17-l27-select-beautify-g0/README.md)）
- 本批：技能 `xiefang`（撷芳）+ 转换包 `packages/extensions/select-beautify-upstream-liuli`（生成器 `gen_select_beautify_package.rs`）
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化——距离修正 + `vm_aux.marks` 复用）
- 台账：L27 由「G0 完成；技能面待批次实施」→ **部分重构（批 1/N）**

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 距离修正 | `mod.globalFrom`：`distance - countPlayer(hasSex('female'))` | `GameStateCore::computed_distance` 新增撷芳臂：持技者（启用）计算与他人的距离 `-` 存活女性数（末尾 `max(1)` 钳制同口径） |
| 标记刷新 | `trigger {global:['dieEnd','gameDrawEnd']}` 锁定：存活女性数 > 0 → 置 `xiefang_mark`；否则移除 | `Xiefang`（tag 517）：`Death`（任意角色）+ 自己摸牌阶段结束 → 刷新 `vm_aux.marks[(座位, 技能定义)] = 女性数`（0 时移除） |
| 女性判定 | `current.hasSex('female')` | `GameStateCore::effective_gender(p) == Gender::Female`（主将槽武将性别；无槽默认男性） |
| 子技/标记文本 | `subSkill.mark`（`marktext:"撷芳-"`） | **登记差异**：标记以技能定义为键存于 `vm_aux.marks`，未建独立标记技与文本 |

## 验证

```
cargo test --offline -p engine-native --test l27_xiefang_batch1
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（撷芳） | **2 passed / 0 failed**（[targeted.log](targeted.log)）：把 2 号换为女性武将后，持技者到其距离**变小**且标记 = 存活女性数（1）；对照（未持技：距离不变、无标记） |
| engine-native 全量 | **1068 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **88/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**（标准 14 场景不可达该技）——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 513（tag 517 追加） |

## 登记差异

- 标记未建独立子技与 `marktext`（以技能定义为键存 `vm_aux.marks`）。
- 上游 `init`（开局初始化标记）未单独建模：首次触发（死亡/摸牌阶段结束）即刷新；开局即持有标记的时点差异已登记。
- 上游 `onremove`（移除技时清标记）未建模。
- **其余 41 个在用技能待批次实施**；选将 UI 主面为混淆代码（不逐行重构，仅登记可观测接口）。
