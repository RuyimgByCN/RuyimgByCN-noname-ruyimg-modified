# NC10：静止态单张敌方陷阱的模拟守恒与遮蔽

- 实际执行：2026-10-28 09:31:18 +0800 → 2026-10-28 09:43:47 +0800（[开始](start-time.txt)、[结束](end-time.txt)）；源码基准 [`78f013d5d0c6c569364bd09266828832a0b8d4f7`](source-head.txt)。规则 v323、存档格式 v105、全局原生 ABI 753 不变。

## 反例、修复与边界

干净 HEAD 用 `git archive HEAD` 导出，复制[反例测试](red-repro.rs)并以[独立 target](red-paths.txt)运行：真实舒莞儿经过正常阶段与选择放置陷阱后，三个非持有者视角的模拟副本都因牌 133 重复而不能恢复，另一测试中同视图、不同陷阱的合成状态不相等；**0 通过／2 失败／0 忽略，exit 101**（[日志](red-clean.log)）。权威对局未被修改。红测命令：`CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=<独立 target> cargo test --offline --manifest-path <干净导出>/Cargo.toml -p engine-native --test nc10_single_hidden_trap -- --nocapture`。

本切片仅处理目录无 VM 技能、无 VM 实体牌堆、结算静止且全场恰有一张敌方陷阱的情况：把该陷阱纳入对手手牌／牌库的同一个确定性合成池，并按同一映射更新丢失史、回合牌引用、玩家牌引用及牌元数据。零预算等回退动作也从合成副本取候选。真实持有者观察时仍保留自己的陷阱。改变的是 AI 模拟副本及规划动作；权威规则、归档编码和同一状态的摘要算法均未改。

真实技能回归覆盖正常放陷阱、四个观察者、三种策略与三档预算，并将计划分别应用到原局及编码归档后恢复的局；构造的 **96** 个不同种子覆盖对手陷阱与牌库互换、合成状态一致、模拟副本可恢复和两档预算下的行动。卡牌引用映射另以人工构造状态验证字段关联与归档恢复；VM 不透明存储的回归验证未覆盖范围内不错误改写其引用。种子测试使用调试授予 `sxianjing_gain`，不代表 96 场真实完整对局。

最初的条件分支在 **96** 对同视图辅助状态中造成 **49** 对原先相同的决策变得不同，因此未采用（[候选日志](candidate.log)、[候选二](guard-candidate2.log)）。最终版本在旗标、存储、丢失史、回合历史、声明牌种、牌属性各 **96** 对中，均没有出现原来相同而现在不同的决策；回合历史组原有 5 对差异变为 0（[干净基线](guard-clean-before.log)、[最终](guard-final.log)、[比较](guard-comparison.txt)、[比较脚本](compare-guard.py)、[探针](guard-probe.rs)）。对照命令为同一导出与独立 target 下的 `cargo test --offline --manifest-path <导出>/Cargo.toml -p engine-native --test nc10_guard_probe_temp -- --nocapture`，先运行 HEAD，再覆盖待测 `ai.rs` 运行最终代码；微基准同法使用 `--test nc10_timing_temp`。这只是所列样本，不证明所有隐藏元数据已脱敏。

**仍待做**：VM 技能或 VM 实体牌堆、多张／多持有者陷阱、挂起状态与其机器引用、其余隐藏元数据的可见性和全范围信息边界；上述未覆盖状态仍可能重复实体或泄漏暗牌。公开投影尚未完整表达技能牌区，AI 质量与多平台尾延迟基线也未完成。NC10 保持部分。

## 验证

除独立基线／微基准外，Cargo 命令均设置 `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=/tmp/noname-nc06-ownership-green.DJEuFN`。

| 命令／范围 | 完整结果 |
|---|---|
| `cargo test --offline -p engine-native --test nc10_single_hidden_trap --test nc10_skill_pile_conservation --test nc10_ai_information_boundary --test nc10_pending_duel_privacy --test nc10_duel_nullify_privacy --no-fail-fast` | **18／0／2**，exit 0；2 项为既有忽略项（[日志](targeted.log)）。 |
| `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=<独立工作树 target> cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader -p game-server --no-fail-fast` | engine-native **1605／0／73**、engine-native-wasm **62／0／0**、content-native-loader **32／0／0**、game-server **360／0／0**；合计 **2059／0／73**、381 个测试目标，exit 0（[完整日志](full-tests.log)、[统计](test-counts.txt)、[目标映射](test-artifacts.jsonl)、[统计脚本](count-tests.py)、[target](target.txt)）。 |
| 同机同负载重分配微基准 | 同一干净导出路径、同一 target 与 debug 构建，预热 200 次、测量 2000 次；无陷阱 p50/p95/p99 **18125／19375／23250 ns → 18416／19917／23459 ns**；单张陷阱 **18083／19167／23500 ns → 25292／26125／32125 ns**。单张陷阱的修改前副本不能恢复、修改后能恢复，行为不等价；这是单机单次微基准而非整局或跨平台结论（[环境与源码哈希](environment.log)、[探针](redaction-timing.rs)、[前](timing-before.log)、[后](timing-after.log)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py`；`rustfmt --edition 2021 --check crates/engine-native/tests/nc10_single_hidden_trap.rs` | 自检 **1／0**、**8640** 个源码文件合格、测试格式通过，均 exit 0（[自检](check-unit.log)、[行数](check-source.log)、[格式](check-fmt.log)）。 |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py`；`git diff --check` | **1846** 个文件、0 坏链；ADR 索引通过；工作树与暂存 diff 无空白错误，均 exit 0（[链接](check-links.log)、[ADR](check-adr.log)、[工作树](check-diff.log)、[暂存](check-staged.log)）。首轮暂存检查发现 8 份日志的尾空行，补记已确认退出码后复检通过（[首轮](check-staged-initial.log)）。 |

本批前已有的八份用户文档改动保留；进度与台账只暂存本批追加行。
