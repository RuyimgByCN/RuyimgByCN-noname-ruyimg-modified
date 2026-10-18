# L10 史诗卡牌 切片 3：装载器模块 + 十常侍面依赖判定（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L10 史诗卡牌**（`extension/史诗卡牌/extension.js`）
- 本批：`content-native-loader::epic_cards`（配置解析 + 十常侍面依赖判定）+ 3 项装载器测试
- **规则/存档版本不变**（v321 / 存档 v105；无引擎改动）

## 本批内容

| 面 | 上游 | 原生 |
|---|---|---|
| 配置 | `cardEffects`/`skillEffects`/`skillEffectsScale`/`cardEmoji`/`cardFold`/`cardPhantom`/`meihuaskills`/`shichangshi`/`skinEffects` | `parse_epic_config`（严格解析 + 缩放 0–10 范围校验）+ `PresentationFlags` 汇总（**展示面不迁移渲染**，仅登记） |
| 十常侍面 | `lib.character['shichangshi'][3] = ['k_danggu','k_mowang']` + 换图；顶部要求「先安装或开启十周年UI扩展」 | `EpicConfig::shichangshi_plan(has_character, has_skill)` → `Disabled` / `MissingCharacter` / `MissingSkill` / `Apply{character, skills, required_extension}` |
| 依赖判定 | 依赖外部包技能 `k_danggu`/`k_mowang` | **本仓未定义**（3.4 MB 的 L08 十周年UI 中亦未定义；上游注释中的 `k_shichangshi` 被注释）→ 依赖判定返回 `MissingSkill`，覆盖不执行（登记差异） |

## 计数与结论

- **L10 规则面已完整**：`reguhuo`（切片 1）+ `rangjie`（切片 2）均落地并各 3 测；本批补齐配置与依赖登记面。
- L10 剩余：**十常侍面**（依赖本仓未定义的外部技能 → 结构性阻塞）+ **展示面**（卡面/技能特效，原生不迁移）。
- 台账：L10 仍「部分重构（切片 3/N）」；八项验收中「功能/配置/依赖」已具证据，「组合/保存恢复/安装升级/目标平台/规则差异」待下一批统一验收。

## 验证

```
cargo test --offline -p content-native-loader
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（装载器） | **3 passed / 0 failed**（[loader.log](loader.log)）：配置解析 + 展示面汇总；缩放越界/缺字段报错；十常侍面四态判定（Disabled/MissingCharacter/MissingSkill/Apply） |
| engine-native 全量 | **1059 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **69/0/0**（[deps-tests.log](deps-tests.log)；较上批 +3 = 新增装载器测试） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 登记差异

- 十常侍面**不执行**：`k_danggu`/`k_mowang` 本仓未定义（结构性阻塞，已在装载器显式判定）；上游的换图/立绘属展示面。
- 展示面（卡面/技能特效、emoji、折角、幻影、皮肤特效）原生不迁移渲染；配置仅作登记与查询面。
- 上游对 `十周年UI` 的启用要求以 `required_extension` 登记（未在原生做扩展间强制依赖门控）。
