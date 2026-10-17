# T11 Boss 诸葛亮族批 116：起风 + 改命 + Boss 装配（字段批 v304/存档 v89，2026-10-17）

- 日期：2026-10-17
- 范围：`boss_zhugeliang`（祭风卧龙）四技收官的**起风 `hlss_fengqi`（+ 起风·闭 `hlss_fengqi2`）与改命 `hlss_gaiming`**，以及 **Boss 转换包与装配**
- **规则 v303→304、存档 v88→89**（`PlayerState` 追加 `hlss_fengqi_used`；选择项新增 `ChoiceOption::Kind(u16)`；判定机追加改命候选位）
- 上游回读：`archive/upstream/noname-liuli/noname/character/extra.js`（`hlss_fengqi` 456–526、`hlss_gaiming` 646–726）、`mode/boss.js:1303`（`boss_zhugeliang:['male','shen',Infinity,[四技],...]`）
- **`T11-BOSS-ZHUGE-LIANG` 关闭**（验收项：hp Infinity 承载设计 ✓、四技 ✓、规则版本递增 ✓、14 golden 查明原因后重录 ✓、规则基准条目 ✓、证据目录 ✓）

## 交付

| 面 | 内容 |
|---|---|
| 起风 `hlss_fengqi`（tag 466） | 准备阶段（Judge 阶段开始）可选：从**可用非延时锦囊牌名**中选一张并使用（无中/五谷/桃园/南蛮/万箭/决斗/顺手/过河/铁索），已用牌名不再出现；候选以目录牌种码承载 |
| 起风·闭 `hlss_fengqi2`（tag 467） | 锁定——来源持技时其使用的牌不可被无懈：`NullifyWindowMachine` 开窗即关（不询问、不抵消，直接委托效果机；覆盖单牌/AOE 逐目标/铁索各窗口） |
| 改命 `hlss_gaiming`（tag 468） | 判定前（判定机取锦囊后、翻判定牌前）：拥有者持技时亮牌堆顶 7 张，由其选 1 作为判定结果；其余 6 张弃置、**改判/天妒窗跳过**（上游 `trigger.cancel + trigger.direct` 同口径）；放弃则 7 张按原序回牌堆顶、照常判定（**登记差异**：上游取消后 7 张滞留处理区） |
| Boss 包 | `packages/extensions/boss-zhugeliang-upstream-liuli`（`upstream-liuli/boss-zhugeliang@1.0.0`）：`boss_zhugeliang`（qun/shen，`hp: "Infinity"`）+ 五技映射（四技 + `hlss_fengqi2`）；资产仅 `assets/cards.json` |
| 协议扩展 | `ChoiceOption::Kind(u16)`（目录牌种码，判别 5；`archive::kind_code`/`kind_from_code` 与存档 AOE need 同表）；wasm JSON 投影 `{ "kind": N }`；客户端 `nativeBackend.ts` 选项类型同步 |

## 语义细节与登记差异

- **起风候选表**：标准非延时锦囊 9 种（无中/五谷/桃园/南蛮/万箭/决斗/顺手/过河/铁索）。**登记差异**：①上游从 `lib.inpile` 取全部 `type=='trick'` 且 `cardEnabled` 者，原生限目录固定表（扩展锦囊牌种与借刀杀人不入候选）；②目标合法性按距离/持牌近似（顺手/过河距离 ≤1、火攻未实现故不在表）；③无目标锦囊里 AOE 显式给出全部其他存活者（自下家座位序冻结），无中/五谷/桃园由各自机器取目标。
- **起风可选性**：上游 `direct:true` + 可选按钮窗（窗内可取消），原生以「可选触发确认门」承载（等价，登记差异）。
- **改命取消路径**：见上（牌堆归还 vs 上游滞留）。
- **无懈闭**：上游 `mod.wuxieRespondable=false` 对「该玩家使用的牌」生效；原生在无懈窗机器入口判定来源技能（覆盖所有窗口）。

## 验证

```
cargo test --offline -p engine-native --test t11_boss_zhugeliang_hlss
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native-wasm --lib
cargo test --offline -p engine-native --test replay_scenarios
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（Boss 诸葛亮族） | **8 passed / 0 failed**（[targeted.log](targeted.log)）：装配初始化+光回满+失星雷伤分配；隐月免除与去隐月对照；三星耗尽觉醒；隐月·判翻面取消/判定失效与对照；无限体力与星系统并存；**起风**选牌名使用（无中摸 2、已用牌种入账、次轮候选排除、放弃不发动）；**改命**选红桃改判（乐不生效、候选入弃堆）与放弃（7 张回牌堆顶、照常判定） |
| engine-native 全量 | **909 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| wasm | **20/0**（[wasm-tests.log](wasm-tests.log)） |
| 14 golden | 重录 + 版本回退探针（304→303 全 14 场 mismatch，[probe-version-rollback.count](probe-version-rollback.count)）——[replay-golden-v304.log](replay-golden-v304.log) |
| Gate 1b | `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 边界（不属于本条目）

- **Boss 关卡流程接入**（哪个关卡/阶段刷出诸葛亮、轮次与增援脚本）属 D4 `BossStageState` 的后续批；本条目只交付 Boss **武将规则面 + 可装配包**。
- `ylyg*`/`mbcm*` 外部依赖（`ylyg_pangtong` 等）与龙血玄黄本体 `mbqianlong`/`_CMeffects` 仍归 `D5-LX-MIGRATION`/`T11-BOSS-PANGTONG-EXTERNAL`。
- 客户端选择 UI 对 `{kind:N}` 的呈现为产品面登记差异（`T13-PRODUCT-CLIENT`）。
