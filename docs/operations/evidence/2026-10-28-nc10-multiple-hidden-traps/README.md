# NC10：静止态多张敌方陷阱的模拟守恒与遮蔽

- 实际执行：2026-10-28 10:03:15 +0800 → 2026-10-28 10:17:09 +0800（[开始](start-time.txt)、[结束](end-time.txt)）；源码基准 [`1e1472def00fbae5df72ade46b83a1b78de5e0d6`](source-head.txt)。规则 v323、存档格式 v105、全局原生 ABI 753 不变。

## 反例、修复与边界

以 `git archive HEAD` 导出干净源码，叠加最初的两个反例测试，用独立 target 运行：四个观察者的多陷阱模拟均出现实体牌重复，同一可见视图下的模拟状态依赖隐藏陷阱；**0 通过／2 失败／0 忽略，exit 101**（[原始红测](red-clean.log)）。最终五项测试在干净 HEAD 上为 **1 通过／4 失败／0 忽略**，包含真实技能路径与引用反例（[最终红测](red-final-clean.log)）；这些是模拟副本，权威对局没有被修改。

本批将无 VM、无 VM 实体牌堆且结算静止时多个对手的陷阱，按持有者和张数并入确定性合成池；观察者自己的陷阱保持原样。对每位持有者按花色容量分配不同花色，并把旧实体的回合、丢失史、玩家牌引用及元数据以同一映射更新。权威规则、存档编码和同一状态的摘要算法不变；AI 规划动作可能随修复发生变化。

真实舒莞儿在调试设置的第一个出牌阶段及自然推进后的下一出牌阶段各放置一张陷阱；四个视角的模拟均可恢复。另有 96 组构造状态覆盖本人陷阱、两个对手的多张陷阱、不同隐藏实体、零／非零预算、可执行计划及恢复后一致性；针对跨牌区引用和花色稀缺的特殊回归分别检查恢复与每个持有者的花色。构造样本不等于多场真实完整对局。最初的真实技能夹具在同一阶段连续发动两次，被 `SkillAlreadyUsed` 拒绝；干净导出复现（[日志](targeted-final.log)），修正为推进回合后通过（[日志](targeted-suits.log)），不是产品故障。初版重分配也会在同一陷阱堆产生重复花色，干净导出实测失败（[日志](suit-multiseed.log)），最终按可用花色容量分配。

在同一导出、同一 target 下，对旗标、存储、丢失史、回合历史、声明牌种、牌属性各 96 组可见视图相同的辅助状态比较规划动作，修复后没有原来相同而变为不同的样本；回合历史组原有 6 组差异变为 0（[修改前](guard-timing-before.log)、[修改后](guard-timing-after.log)、[比较结果](guard-comparison.txt)、[比较脚本](compare-guard.py)、[探针](guard-probe.rs)）。这不证明其他隐藏字段或所有状态均已脱敏。

**仍待做**：含 VM 技能或实体牌堆、挂起机器／选择及其他引用、其余隐藏元数据的全范围信息边界，以及 AI 质量和多平台尾延迟。合法陷阱由技能保持不同花色；对已有不满足约束的畸形输入，花色不足时模拟仍可能保留重复花色，尚缺拒绝或修复策略。NC10 保持部分。

## 验证

除文档与源码门禁外，下列 Cargo 命令均使用 `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0`，独立 target 与 `git archive HEAD` 导出路径见[记录](clean-directory.txt)。两套完整运行分别在干净导出中叠加本批源码和测试，后一次仅整理源码格式；新增稀缺花色测试后再全量复跑内核。

| 命令／范围 | 完整结果 |
|---|---|
| `cargo test --offline -p engine-native --test nc10_multiple_hidden_traps --test nc10_single_hidden_trap --test nc10_skill_pile_conservation --no-fail-fast` | 干净导出早期版本 **12／0／0**（[日志](targeted-suits.log)）；最终新增稀缺花色测试定向 **1／0／0**，另有 4 过滤（[日志](scarce-targeted.log)）。 |
| `cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader -p game-server --no-fail-fast` | 整理代码格式后全量：内核 **1609／0／73**、WASM **63／0／0**、加载器 **32／0／0**、服务端 **360／0／0**；合计 **2064／0／73**、382 个测试目标，exit 0（[日志](rust-full-final.log)、[统计](rust-counts-final.txt)、[目标映射](test-artifacts-final.jsonl)、[统计脚本](count-tests.py)）。初轮统计脚本把 `/tmp` 与 `/private/tmp` 视为不同路径而报错（[记录](count-initial.log)）；规范化两侧路径后重新统计。编译有既存 content-compiler ABI 755 未使用警告。 |
| `cargo test --offline -p engine-native --no-fail-fast` | **1610／0／73**、331 个测试目标，exit 0（[日志](native-full-final.log)、[统计](native-counts-final.txt)、[目标映射](native-artifacts-final.jsonl)、[统计脚本](count-native-tests.py)）。 |
| 同机同负载重分配微基准 | 同一导出、target 和 debug 构建，每种预热 200 次、测量 2000 次。无陷阱 p50/p95/p99 **18167／18916／23458 ns → 18583／19334／23625 ns**；多陷阱 **19167／19792／23958 ns → 29166／31291／37208 ns**。多陷阱修改前副本不可恢复、修改后可恢复，行为不等价；单次微基准不代表整局或跨平台性能（[环境](environment.txt)、[探针](timing-probe.rs)、[前](guard-timing-before.log)、[后](guard-timing-after.log)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py`；`rustfmt --edition 2021 --check crates/engine-native/tests/nc10_multiple_hidden_traps.rs` | 自检 **1／0**、**8643** 个源码文件合格、测试格式通过，均 exit 0（[自检](source-line-tests.log)、[行数](source-lines.log)、[格式](rustfmt.log)）。 |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py`；`git diff --check` | 全部通过，具体输出见[链接](local-links.log)、[ADR](adr-index.log)、[差异](diff-check.log)及[暂存](staged-check.log)。 |

本批前已有的八份用户文档改动保留；进度与台账只暂存本批追加行。空间不足时，仅清理了本会话先前测试生成且已无进程使用的临时源码导出。
