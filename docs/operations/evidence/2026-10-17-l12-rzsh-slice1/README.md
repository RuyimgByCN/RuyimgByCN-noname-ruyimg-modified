# L12 如真似幻 切片 1：G0 清点 + 纯函数面（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L12 如真似幻**（`extension/如真似幻/extension.js`，596 KB；全 JS 6.5 MB）
- 本批：装载器 `content-native-loader::rzsh`（罗马数字 + 两张文案表 + 确定性取模）
- **规则/存档版本不变**（v321 / 存档 v105；无引擎改动）
- 台账：L12 由「待重构；未验收」→ **部分重构（切片 1/N）**

## G0 清点（脚本实测，见 [survey.json](survey.json)）

| 面 | 实测 | 结论 |
|---|---|---|
| `lib.skill` 定义 | **0** | 无技能面 |
| `lib.card` | 2 处（`lib.card.list` 仅牌表引用） | 无卡牌面 |
| `lib.character` | 8 处（展示/立绘引用） | 无武将面 |
| 配置键 | **0**（无 `extension_如真似幻_*`） | 无配置面 |
| 函数 | 59 个；其中 42 个**不含** DOM/WebGL 调用（启发式） | 多数仍为大厅/动画流程 |
| 语义明确的纯函数 | `numtoroma`（1–6 → 罗马数字）、`getRandomSanguoText`（11 条文案）、`getRandomTip`（4 条提示） | **本批迁移** |

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 罗马数字 | `numtoroma(num)`：1–6 → `Ⅰ…Ⅵ`，其余 `null` | `roman(num) -> Option<&'static str>`（逐值对齐） |
| 随机文案 | `sanguoTexts[Math.floor(Math.random()*len)]`（11 条） | `SANGUO_TEXTS`（11 条，**表随模块迁移**）+ `sanguo_text(roll)`（取模、可复现） |
| 随机提示 | `tips[...]`（4 条） | `TIPS`（4 条）+ `tip(roll)` |
| 渲染/大厅 | WebGL/spine 大厅、抽屉、按钮、动画（59 函数主体） | **不迁移**（登记差异） |

## 验证

```
cargo test --offline -p content-native-loader --test rzsh_loader
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（装载器） | **3 passed / 0 failed**（[loader-tests.log](loader-tests.log)）：罗马数字 1–6 与越界；两表条数与内容抽检（11/4）；取模回绕与确定性 |
| engine-native 全量 | **1078 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **91/0/0**（[deps-tests.log](deps-tests.log)；较上批 +3） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 台账口径说明（重要）

- 台账 L12 的**完成条件**原文要求「页面提供者 + 排位会话/档案 + 必要规则钩子」；本批 G0 实测显示该来源**无规则面**（0 技能/0 卡牌/0 武将/0 配置），其主体是渲染与大厅流程。
- 因此**不改**完成条件、**不标整包**：本批只登记「无规则面」这一实测结论与已迁移的纯函数面；若后续要按原条件验收，需要先落地页面提供者/排位会话宿主契约（属宿主能力而非本来源规则面）。
- 登记差异：渲染/大厅/动画不迁移；随机性由调用方 `roll` 给定（可复现）。
