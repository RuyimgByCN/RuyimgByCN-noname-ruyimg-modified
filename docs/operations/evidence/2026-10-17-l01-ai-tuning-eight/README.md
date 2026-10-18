# L01 AI优化 八项整包验收（2026-10-17）

- 日期：2026-10-17
- 来源：**L01 AI优化**（`extension/AI优化/extension.js`，430 KB）
- 结论：**八项整包验收通过**（`T12-SRC-L01` → resolved；整包计数 **21/37 → 22/37**）
- 证据：[切片 1](../2026-10-17-l01-ai-tuning-slice1/README.md)（调参面）、[切片 2](../2026-10-17-l01-ai-tuning-slice2/README.md)（AI 消费面）

## 八项对照

| 项 | 结论 | 证据 |
|---|---|---|
| 功能 | 装载面参数与口径（威胁度 `cf` / 权重 `qz` / `applyCf` 三态 / `ckQz` 三来源 / `qzCf` 效益 / 评级映射 `[0.8,1.95]`）+ **原生 AI 消费**（`ContentTuning` + 策略变体 2） | eight 用例「功能」两段 |
| 配置 | `parse_ai_tuning_config` 严格解析（非法模式/越界数值/非 JSON 报错）；商店副本逐字节一致 | eight 用例「配置」段 |
| 依赖 | manifest 依赖为空；上游的**评级来源**（千幻聆音）以参数注入（`rating`），不构成包依赖 | eight 用例「依赖」段 |
| 组合 | 与 swd 包同时装载建局，调参规划器（变体 2）可用 | eight 用例「组合」段 |
| 保存恢复 | 带包存档往返；**恢复后同调参同决策**；商店重开内容锁哈希不变 | eight 用例「保存恢复」段 |
| 安装升级 | 真实 `import_extensions`（内容哈希一致）→ 2.0.0 覆盖 → `rollback` → 未安装版本 `NotFound` | eight 用例「安装升级」段 |
| 目标平台 | 纯参数 + 评分面，无平台专有代码（不 eval/fetch）；设置菜单交互不迁移 | 配置断言 + 登记差异 |
| 规则差异 | **不改规则**：同种子同操作序列下阶段推进与体力状态与未调参完全一致；变体 2 仅影响评分（变体 0/1 行为不变，golden 零位移） | eight 用例「规则差异」段 |

## 验证

```
cargo test --offline -p engine-native --test l01_ai_tuning_eight
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 八项用例 | **1 passed / 0 failed**（[eight.log](eight.log)） |
| engine-native 全量 | **1064 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **75/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 包与生成器

- 包：`packages/extensions/ai-tuning-upstream-liuli`（`manifest.json` + `package.npack.json` + `assets/config.json`；**无技能/卡牌/武将条目**=不改规则）。
- 生成器：`crates/content-runtime/examples/gen_ai_tuning_package.rs`。

## 登记差异（整包验收后仍保留）

- 身份/模式专项策略（`dizhu`/`nongmin`/`nei`/`fan`/`findZhong`/`mjAi`/`sfjAi`/`btAi`）未迁移。
- 上游更细的评分修正（牌序、卖血倾向、濒死求桃判定）未逐项迁移；原生以策略变体 2 的**席位权重加权**承载威胁度/权重面。
- 席位权重解析在装载面完成、由宿主注入（未做自动装配）；评级来源不绑定具体扩展。
- 设置菜单交互（编辑/复制/清空威胁度）不迁移。
