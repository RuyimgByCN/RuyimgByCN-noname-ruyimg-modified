# L01 AI优化 切片 2：AI 消费面（内容调参参与原生规划器评分）（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L01 AI优化**（`extension/AI优化/extension.js`）
- 本批：原生规划器**消费面**——`engine_native::ai::ContentTuning` + 策略变体 2（席位权重加权）
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化——评分面改动）
- 台账：L01 仍「部分重构（切片 2/N）」；「同信息集同决策」已具证据

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 威胁度覆盖 | `lib.config.extension_AI优化_cf[i]` → `lib.skill[i].ai.threaten` | `ContentTuning.skill_threat`（技能名 → 威胁度）+ `threat_of_skills(...)`（取最大值；无记录 1.0——对应装载面 `QzMode::Threat`） |
| 武将权重 | `qz` 表 / `ckQz` 来源（评级 `[0.8,1.95]` / 威胁度） | `ContentTuning.seat_weights`（调用方按武将/技能解析后注入；`weight_of(seat)` 缺省 1.0，钳制 `[0.1,10.0]`） |
| 效益结算 | `qzCf`：权重 × 结果 | `ContentTuning.benefit(weight, value)` |
| 评分接入 | AI 对敌方损失按威胁/权重取舍 | 规划器**策略变体 2**：`policy_score` 中敌方体力/手牌/装备损失与击杀奖励按**席位权重**加权（变体 0/1 行为不变） |
| 同信息集同决策 | — | 同种子同局面同调参 → 同决策（测试断言两次规划一致） |

## 验证

```
cargo test --offline -p engine-native --test ai_content_tuning
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（消费面） | **3 passed / 0 failed**（[targeted.log](targeted.log)）：同调参同局面两次规划决策一致（确定性）；**权重改变决策**——`weight(3)=5` → 目标含 3，`weight(1)=5` → 目标含 1；缺省权重 1.0、`threat_of_skills` 取最大、`benefit` 两态、权重钳制 `[0.1,10.0]` |
| engine-native 全量 | **1063 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **75/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**（变体 0/1 评分路径未变）——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 登记差异

- 席位权重的**解析**（武将权重表 / 评级映射 / 技能威胁度）在装载面完成（切片 1）；原生规划器只消费已解析的 `seat_weights`（宿主注入；未做自动装配）。
- 变体 2 为**新增**评分路径（变体 0/1 行为逐位不变，golden 零位移）；上游更细的评分修正（牌序、卖血倾向、濒死求桃）未逐项迁移。
- 身份/模式专项策略（`dizhu`/`nongmin`/`nei`/`fan`/`findZhong`/`mjAi`/`sfjAi`/`btAi`）仍未入。
