# L27 选将美化 批 5：集舍（`jishe`）+ 余技分类（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L27 选将美化**（`extension/选将美化/extension.js`）
- 本批：技能 `jishe`（集舍，tag 522）+ **余技结构分类**（脚本实测）
- **规则/存档版本不变**（v321 / 存档 v105）

## 集舍（`jishe`）

| 面 | 上游 | 原生 |
|---|---|---|
| 类型 | `enable:"phaseUse"`；`filter`：`getHandcardLimit()>0` | `Jishe`（tag 522）：出牌阶段主动面（`activate`；`hand_limit > 0`） |
| 效果 | `player.draw()` + `addMark("jishe2",1)`（累计） | 摸 1 + `vm_aux.marks[(座位, jishe 定义)] += 1` |
| 手牌上限面 | `jishe2` 的 `mod.maxHandcard`——**该技在本文件中被注释**（`}*/`）→ 上游实际不生效 | **不实现**（与上游实际行为一致；登记差异注明） |
| 展示 | `jishe_mark` 文本（`hp - count`） | 不迁移（展示面） |

## 余技结构分类（实测；脚本见文末）

| 类别 | 数量 | 说明 |
|---|---|---|
| 展示专用（纯 mark） | 1 | `rehuashen_mark`（92 B，无 trigger/content） |
| 引用未定义子技（含展示 mark） | 9 | `regushe`/`kuangcai`/`sbtongye`/`clandianzhan`/`jishe`/`qimou`/`reqimou`/`twxiafeng`/`twzuhuo`——所引子技多为**展示 mark** 或上游未定义 |
| 自包含（可移植） | 27 | 其中 **6 个已落地**（xiefang/new_zhixi/new_meibu/new_meibu_range/sbzhaxiang/jishe）→ 余 **21** |
| 大体量（>15 KB） | 6 | `huamu`(17 KB)/`potzhuangshi`(21 KB)/`mbcmqingzheng`(36 KB)/`sbqingzheng`(37 KB)/`spyanjiao`(63 KB)/`mbwuling`(73 KB) |

> 注：`dead_ref` 与 `self_contained` 有重叠（如 `jishe` 既引用未定义 `jishe_mark` 又已落地）；上表按**启发式**分类，用于排期而非验收口径。

## 验证

```
cargo test --offline -p engine-native --test l27_jishe_batch5
cargo test --offline -p engine-native --test l27_xiefang_batch1 / l27_new_zhixi_batch2 / l27_new_meibu_batch3 / l27_sbzhaxiang_batch4
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（集舍） | **2 passed / 0 failed**（[targeted.log](targeted.log)）：发动 → 摸 1 且标记 +1；再次发动标记累加至 2；对照（未持技拒绝） |
| 定向（批 1–4 回归） | 全部通过（[targeted-regression.log](targeted-regression.log)） |
| engine-native 全量 | **1078 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **88/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 518（tag 522 追加） |

## 登记差异

- 集舍的**手牌上限削减面不实现**：上游 `jishe2` 在本文件中被注释（`}*/`）→ 上游实际不生效（原生与上游实际行为一致）。
- `jishe3`（`group`）在本文件中无定义 → 不实现（上游同样无定义）。
- 展示标记（`jishe_mark` 文本 `hp-count`）不迁移。
- 余 **21** 个自包含技能 + 6 个大技能待批次；选将 UI 主面为混淆代码。

## 分类复现

```python
import re, json
s=open('archive/upstream/noname-liuli/noname/extension/选将美化/extension.js',encoding='utf-8').read()
idxs=[m.start() for m in re.finditer(r'lib\.skill\.([A-Za-z_0-9$]+)\s*=\s*\{', s)]
# 按体量/依赖启发式分类（见 categorization-summary.txt）
```
