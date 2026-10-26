# EffectRequest 功能拆分核验（2026-10-24）

基准：`7e28a93799a9c01794085e911a72a4faa5673883` 加未提交工作树；规则 **v322**、存档 **v105** 不变。当前状态统一见[进度主稿](../../../architecture/native-progress.md#effect-split-2026-10-24)，本目录记录本次拆分的原始结果与边界。

## 实现与相容性

`crates/rule-api/src/effect.rs` **10778 → 700 行**，保留公开辅助类型、常量及有序登记表；**22 个功能模块**提供完整变体，最大 **1940 行**。按卡牌移动/选择、伤害恢复、判定、回合、技能标记、濒死、事件及各来源武将解析器分组，沿用项目的宏收集模式，生成同一个扁平 `EffectRequest`，调用方与公开导出不变。移除 7 份未接入编译的旧顺序片段，不修改门禁阈值、扫描范围或排除目录。

独立审查重建全部 **815 个变体、460 个有序段**，与拆分前完整枚举体逐字相同；辅助类型前缀亦逐字相同。拆分前文件与该 HEAD 中原文件一致，可从 Git 重现。见[独立审查](independent-review.json)和[最终源码清单](source-manifest.json)。完整枚举体 SHA256：`f4ec6389b3b2ffd974f6bdeecd4b975893346be75425dfd05b0d9d1d44586e71`；完整定义去空白后的顺序摘要：`f7d1026e7af40b0586e2e2ded9c70a702f9a084da667dce5f5f231663d2ef873`。

Rust 新增 **2 项相容性测试**：一项用拆分前固定的 815 条样本逐个验证 Serde 变体序号、字段顺序及 JSON 往返；另一项实跑旧载荷省略 `DealDamage.nature`、`RecoverHp.source` 的两例默认值。其余字段、默认值及属性由完整定义比对保护。未改 golden 期望值、规则版本、存档版本或 VM/哈希标签。

`plan_effect_request_split.py --check` 现为当前功能拆分稽核，拒绝缺失、重复、未使用的模块/段/变体和未知语法；相应单测 **14/14**。`preflight.py` 移除行数预期失败豁免，相应单测 **4/4** 验证失败会阻挡；两组均包含在全量 Python 测试中。

## 最终验证

所有命令从仓库根目录执行；原始命令、退出码及日志摘要另见[验证清单](verification.json)。主机测试的 Cargo 为 rustup 1.95.0，实际 rustc/rustdoc 来自继承的 PATH（Homebrew 1.99.1）；WASM 命令显式优先 rustup PATH（1.95.0）。见[工具链](toolchain.txt)与[主机编译器探针](host-compiler-probe.log)。

| 验证 | 执行命令 | 结果 / 原始输出 |
|---|---|---|
| 原生与直接消费者全量 | `/Users/ruyimg/.cargo/bin/cargo +stable test --offline -p rule-api -p engine-native -p engine-native-wasm -p content-native-loader -p content-compiler -p rule-vm --no-fail-fast` | **2165 passed / 0 failed / 71 ignored**；[完整日志](final-native-and-consumers.log)、[逐 crate/测试目标汇总](final-native-test-summary.json) |
| 旧线 golden | `/Users/ruyimg/.cargo/bin/cargo +stable test --offline -p engine-core --test golden_start --test golden_regression --no-fail-fast` | **6 passed / 0 failed**；[日志](core-goldens.log) |
| 全 workspace 编译 | `/Users/ruyimg/.cargo/bin/cargo +stable check --offline --workspace --all-targets -j 4` | **exit 0**，保留既有 warnings；[日志](workspace-check.log) |
| 两条 WASM 编译 | `PATH=/Users/ruyimg/.cargo/bin:$PATH cargo +stable check --offline -p engine-native-wasm -p engine-wasm --target wasm32-unknown-unknown` | **exit 0**，保留既有 warnings；[日志](wasm-check-final.log) |
| rustup 工具链相容性补查 | `PATH=/Users/ruyimg/.cargo/bin:$PATH cargo +stable test --offline -p rule-api` | **2 passed / 0 failed**；[日志](rule-api-rustup-tests.log)，验证 rustup 1.95.0 下同样的 815 样本与两例默认值 |
| Python 脚本全量重跑 | `python3 -B -m unittest discover -s scripts` | **180 tests / OK**；[最终日志](python-tests-final.log) |
| 行数脚本单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | **1 test / OK**；[日志](source-line-unit.log) |
| 全仓行数门禁 | `python3 -B scripts/check-source-lines.py` | **8563 文件全部 ≤2000 行**；[日志](source-line-gate.log)、[逐根目录与最大文件清单](line-inventory.json) |
| 拆分完整性 | `python3 -B scripts/plan_effect_request_split.py --check` | **exit 0**，815 变体 / 22 模块 / 定义摘要一致；[日志](split-audit.log) |
| preflight 快速检查 | `python3 -u -B scripts/preflight.py --quick` | **18 项通过、2 项跳过，exit 0**；[日志](preflight-quick-final.log)。跳过脚本单测和台账↔枚举一致；脚本全量另行通过，不宣称完整 preflight 已执行 |
| 本次 Rust 文件格式 | 命令与文件列表见独立审查中的 `rustfmt` | **exit 0**；[日志](changed-rustfmt.log)。`skip_children=true` 避免递归检查未改动的 `query.rs`，不代表仓库全局格式通过 |

六个 crate 的通过数：`engine-native` **1547**（另 **71 ignored**）、`engine-native-wasm` **36**、`content-native-loader` **32**、`content-compiler` **533**、`rule-vm` **15**、`rule-api` **2**；合计 **2165**。两项新相容性测试中的 815 条样本不是 815 个独立测试函数。

## 保留的失败与修复原因

- [首次 Python 全量](initial-python-tests.log)运行 180 项，其中 1 项失败：四处历史文档引用 `construction.rs` 的旧第 468–500 行（2026-10-18 范围），已超出当前源码范围。保留当时范围和结论的日期语境，并补充当前通用全局授技位置 `construction.rs:463-477`；未改变台账验收状态。修复后完整重跑 180 项通过。[首次 preflight](initial-preflight-quick.log)同因失败，最终快速检查通过。 新证据页草稿复述旧范围时也曾触发[同一引用检查](draft-evidence-preflight.log)，改成带日期的历史说明后重跑；原始失败日志均保留。
- [首次原生 WASM](initial-wasm-check.log)和[首次旧线 WASM](initial-legacy-wasm-check.log)均 exit 101：rustup Cargo 混用 PATH 中的 Homebrew rustc，后者没有该 WASM 目标的标准库。rustup 本身已安装目标；显式统一 PATH 后两条 WASM `check` 一起通过，无需改仓库或重装依赖。
- [初版分组回归](initial-native-and-consumers.log)为 2163 passed / 0 failed / 71 ignored；功能归属微调并加入 `rule-api` 测试后，对最终源码重新全量执行六个 crate，以上表最终日志为准。既有 NC10 测试竞态的修复与先前产品验证属于[前次扩展交付](../2026-10-24-native-extension-delivery/README.md)，不计为本次新修复或本次产品重跑。

## 修改保护与未验收范围

拆分开始时已备份 binary diff 和 **11099** 个既有 dirty/untracked 文件摘要到 `/tmp/noname-next-effect-split-20261024/`。最终核对 **11096 原样保留**，仅在 `Cargo.lock`、进度主稿、扩展台账添加本次明确增量；反向移除 `rule-api` 的新增 `serde_json` 锁项后，锁文件摘要与开始时完全一致。其余源码改动和原 dist 保留，`check-source-lines.py` 自身未变更，HEAD 未变，index 无暂存。见[修改保护核对](preservation.json)。未暂存、未提交、未推送。

71 个忽略测试没有执行；未重跑完整旧线测试、前端/真实 Worker/浏览器产品流程、GitHub Actions、桌面安装包、移动真机或上游等价验收。WASM 此处是编译检查，不代表重新生成发布包或运行期验收。拆分与绿色回归不提升 37 来源的整包完成状态，亦不提供性能收益结论。
