# L10 史诗卡牌 来源切片批 1：蛊惑（`reguhuo`）——G1 包 + 首个技能（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L10 史诗卡牌**（`archive/upstream/noname-liuli/noname/extension/史诗卡牌/extension.js`，133 KB）
- 本批：**G1 转换包落地**（`packages/extensions/epic-cards-upstream-liuli`）+ 首个技能 `reguhuo`（蛊惑）
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化）
- 台账状态：L10 由「待重构；未验收」→ **部分重构（切片 1/N）**

## G0 清点（本批结论）

| 面 | 上游内容 | 处置 |
|---|---|---|
| 技能 | `reguhuo`（蛊惑，4223 字符）、`rangjie`（让节，90 KB 主体为 AI 估值） | `reguhuo` ✅ 本批；`rangjie` deferred |
| 武将 | `lib.character` 4 处（十常侍面） | deferred（`shichangshi` 配置面 + 武将条目） |
| 卡牌 | 无 `lib.card` 定义（扩展为**卡面特效/替换面**） | 展示面 → 登记差异 |
| 配置 | `card`/`cardEffects`/`cardEmoji`/`cardFold`/`cardPhantom`/`meihuaskills`/`shichangshi`/`skillEffects`/`skillEffectsScale`/`skinEffects` | 桥接为 `assets/config.json`（展示面开关，原生不迁移渲染） |
| 依赖 | 无外部包依赖 | — |

## 上游语义与原生接线（`reguhuo`）

| 面 | 上游 | 原生 |
|---|---|---|
| 类型 | `enable:['chooseToUse','chooseToRespond']`；`reguhuo_phase` 每阶段一次 | `Reguhuo`（tag 515）：出牌阶段主动面（`activate`；手牌非空；`host_skill_def` 每回合一次=登记差异：响应面与「每阶段」粒度未落地） |
| 载体 | `backup.position='hs'`（手牌/装备区） | `SelectCards`（手牌，min1/max1，可取消；**装备区未纳入**=登记差异） |
| 声明 | `dialog` 列出全部基本牌/锦囊牌（含杀的属性变体） | `PickTag`（20 项基本/锦囊牌种表；**属性变体**未单列=登记差异） |
| 转化 | `viewAs: {name, nature, suit:'none', number:null}`（**清空花色点数**） | `CardUsed(use_as=声明牌种)`；**实体花色点数保留**=登记差异 |
| 目标 | `evt.filterCard` 由引擎按声明牌种校验 | 复用批 136 `GameStateCore::legal_targets` 通用目标规则表（无目标→直接结算；单/多目标→`SelectPlayers`） |

## 验证

```
cargo test --offline -p engine-native --test epic_reguhuo_viewas
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（蛊惑） | **3 passed / 0 failed**（[targeted.log](targeted.log)）：一张【杀】声明为【桃】→ 回复 1 且牌离手；一张【桃】声明为【杀】→ 目标窗 → 命中 1 点；对照（未持技拒绝） |
| engine-native 全量 | **1056 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**（无版本变化）——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 511（tag 515 追加） |

## 包与生成器

- 包：`packages/extensions/epic-cards-upstream-liuli`（`manifest.json` + `package.npack.json` + `assets/cards.json` + `assets/config.json`）。
- 生成器：`crates/content-runtime/examples/gen_epic_cards_package.rs`（可重复生成；`logic_hash`/`asset_hash` 由 `compute_*` 计算）。

## 登记差异（本切片）

- 仅**出牌阶段主动面**（上游 `chooseToUse`/`chooseToRespond` 两入口）；`reguhuo_phase` 以「每回合一次」近似（上游每**阶段**一次）。
- 载体仅手牌（上游 `'hs'` 含装备区）；声明表未单列杀的属性变体（火/雷杀已作为独立牌种列入）。
- **花色/点数未清空**（上游 `suit:'none', number:null`）——实体牌花色点数保留，影响需要花色/点数的下游判定。
- 展示面（卡面/技能特效、十常侍面）不迁移；`rangjie`（让节）待做。
