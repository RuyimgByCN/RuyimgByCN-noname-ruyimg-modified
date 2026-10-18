# L10 史诗卡牌 八项整包验收（2026-10-17）

- 日期：2026-10-17
- 来源：**L10 史诗卡牌**（`extension/史诗卡牌/extension.js`）
- 结论：**八项整包验收通过**（`T12-SRC-L10` → resolved；整包计数 20/37 → **21/37**）
- 证据：[切片 1](../2026-10-17-l10-epic-cards-slice1/README.md)（G1 包 + 蛊惑）、[切片 2](../2026-10-17-l10-epic-cards-slice2/README.md)（让节）、[切片 3](../2026-10-17-l10-epic-cards-slice3/README.md)（装载器 + 依赖判定）

## 八项对照

| 项 | 结论 | 证据 |
|---|---|---|
| 功能 | `reguhuo`（蛊惑，tag 515：声明任意基本/锦囊牌使用）与 `rangjie`（让节，tag 516：受伤后按点数逐次获得指定类型牌/移动场上牌）均落地；八项用例内含两技端到端复跑 | [eight.log](eight.log)、切片 1/2 定向 |
| 配置 | `assets/config.json` 经 `parse_epic_config` 严格解析（`skillEffectsScale` 0–10 校验）；展示面开关汇总为 `PresentationFlags`；商店副本逐字节可读 | eight 用例「配置」段 |
| 依赖 | 包 manifest 依赖为空；十常侍面的**内容级依赖**（`k_danggu`/`k_mowang`）经 `shichangshi_plan` 四态判定——本仓未定义 → `MissingSkill`（结构性阻塞，已登记）；上游对「十周年UI」的要求以 `required_extension` 登记 | eight 用例「依赖/规则差异」段 |
| 组合 | 与 swd 包**同时装载**建局，两包技能（reguhuo/rangjie/jilve）均可授予；同一次 `import_extensions` 中其余包内容锁不变 | eight 用例「组合」段 |
| 保存恢复 | 带包存档往返：技能实例与体力保持；商店重开后内容锁哈希不变 | eight 用例「保存恢复」段 |
| 安装升级 | 真实 `import_extensions` 安装（内容哈希与 npack 逐字节一致）；2.0.0 覆盖 → `rollback` 回 1.0.0；未安装版本 `NotFound` | eight 用例「安装升级」段 |
| 目标平台 | 展示面（卡面/技能特效、emoji、折角、幻影、皮肤特效）为浏览器/桌面渲染面——原生**不迁移渲染**，仅登记配置；包内无平台专有代码（不 eval/不 fetch） | 配置断言 + 登记差异 |
| 规则差异 | ①蛊惑仅出牌阶段面（上游 `chooseToUse`/`chooseToRespond`）、每回合一次近似每阶段一次、载体仅手牌、**花色点数未清空**、属性变体未单列；②让节牌堆取首张匹配（非搜索面）、移动牌为简化移牌；③十常侍面不执行（外部技能缺失）；④展示面不迁移 | 各切片 README「登记差异」 |

## 验证

```
cargo test --offline -p engine-native --test l10_epic_cards_eight
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 八项用例 | **1 passed / 0 failed**（[eight.log](eight.log)，单用例覆盖八项分段） |
| engine-native 全量 | **1060 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **69/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**（无版本变化）——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 结构性阻塞登记（不因整包验收而隐藏）

十常侍面（`shichangshi` 武将技能替换为 `k_danggu`/`k_mowang` + 换图）**未执行**：这两个技能在本仓（含 3.4 MB 的 L08 十周年UI）中均无定义，上游注释中的 `k_shichangshi` 亦被注释。装载器对此显式返回 `MissingSkill`，属**内容缺失**而非实现缺口；若后续来源补齐这两个技能，可按 `ShichangshiPlan::Apply` 直接接入。
