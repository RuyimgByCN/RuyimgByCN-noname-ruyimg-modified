# L01 AI优化 切片 1：AI 调参面（参数 + 结算口径）（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L01 AI优化**（`archive/upstream/noname-liuli/noname/extension/AI优化/extension.js`，430 KB）
- 本批：装载器 `content-native-loader::ai_tuning`（参数解析 + 威胁度/权重/效益结算口径）
- **规则/存档版本不变**（v321 / 存档 v105；无引擎改动——纯 AI 参数面）
- 台账：L01 由「待重构；未验收」→ **部分重构（切片 1/N）**

## G0 清点（本批结论）

| 面 | 上游内容 | 处置 |
|---|---|---|
| 技能威胁度 | `cf` 表（技能 → 威胁度；`lib.skill[i].ai.threaten` 覆盖） | `AiTuningConfig.cf` + `threaten(skill)`（未配置 → `None`=不改动） |
| 补充模式 | `applyCf`：`off`/`pj`（按武将评级自动补充）/`sd`（手动补充） | `CfMode{Off,Auto,Manual}`（严格解析） |
| 武将权重 | `qz` 表；`ckQz`：`off`/`pj`（评级线性映射 `[0.8,1.95]`）/`cf`（以威胁度为权重） | `AiTuningConfig.qz` + `weight(general, rating, skills)` + `rating_to_weight` |
| 效益结算 | `qzCf`：开启后「权重 × 结果」 | `benefit(cfg, w, value)` |
| 其它开关 | `applyQz`/`viewAtt`/`nhFriends`/`ntAoe`/`banQh`/`mjAi`/`sfjAi`/`btAi`/身份策略（`dizhu`/`nongmin`/`nei`/`fan`/`findZhong`）等 ~30 项 | 本批入 5 项（其余为身份/模式专项策略 → 后续切片） |
| 规则变体 | 上游自带若干「规则钩子」（如濒死求桃判定、卖血倾向） | **不改规则**：原生仅登记参数；AI 消费面在 `engine-ai`（登记差异） |
| 交互面 | 设置菜单（编辑/复制/清空威胁度、评级联动千幻聆音） | 展示/交互面不迁移 |

## 验证

```
cargo test --offline -p content-native-loader --test ai_tuning_loader
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（装载器） | **3 passed / 0 failed**（[loader-tests.log](loader-tests.log)）：模式/表解析（非法模式、越界数值、缺字段报错）；权重三来源（表内优先 / 威胁度取最大 / 评级线性映射含中点与钳制 / Off=1.0）；效益结算开关两态 |
| engine-native 全量 | **1060 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **75/0/0**（[deps-tests.log](deps-tests.log)；较上批 +3） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 登记差异

- **不改规则**：上游虽含少量「规则钩子」（濒死求桃/卖血倾向等），原生本批只登记**参数与结算口径**；AI 决策消费面属 `engine-ai`（宿主注入），未接入本参数表（后续切片）。
- 身份/模式专项策略（`dizhu`/`nongmin`/`nei`/`fan`/`findZhong`/`mjAi`/`sfjAi`/`btAi`）未入本批。
- 评级来源（上游经千幻聆音）以参数注入 `weight(..., rating, ...)`（登记差异：原生不绑定具体评级扩展）。
- 设置菜单交互（编辑/复制/清空威胁度）不迁移。
