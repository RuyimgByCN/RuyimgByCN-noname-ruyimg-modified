# NC10：决斗挂起窗口的隐藏手牌边界

- 实际执行：2026-10-27 23:29:52 +0800 → 2026-10-27 23:44:16 +0800 （开工与结束各运行 `date '+%F %T %z'`）；源码基准 `f42c16017ac1c9bf3b972016749596959ffc9adc`（[开始](start-time.txt)、[结束](end-time.txt)、[源码](source-head.txt)）。原有八处文档改动保留。

## 反例与修复范围

在 shahai-swd 实际卡牌包、seed 9002 的 18 条真实命令后，0 号座位在决斗的 `duel_sha` 选择中持有杀，栈中有 1 个 `CardUsed` 帧，1 个挂起的 `NullifyWindow → Duel` 机器。对手座位 1 的首张暗牌 `192`（`Sifeizhenmian`）与牌库中 `17`（`Sha`）互换，选择者的 `PlayerView`、`ChoiceView`、手牌牌种、事件帧、挂起机器快照完全相同，且该机器引用的决斗牌 `57` 在桌面、选择引用的 `8` 在选择者手中。实际基线规划结果从出 `Card(8)` 变为 `Pass`，证明挂起模拟会读取对手牌面。以 `git archive HEAD` 导出干净源码、仅叠加[本批测试](../../../../crates/engine-native/tests/nc10_pending_duel_privacy.rs)，使用独立 `CARGO_TARGET_DIR` 重现 **1 passed / 2 failed / 1 ignored，exit 101**（[完整日志](clean-head-repro.log)）。

针对能够核对引用的**单帧、单挂起机器的决斗响应窗口**，检查帧、机器、选择、桌面及其他实体牌引用；存在特殊牌区或未核对的状态时继续走原来的保守路径。合格窗口把对手手牌和牌库按既有内容无关方式填充，保留自己的手牌、桌面、公开牌、选择及挂起机器引用。零预算等回退也在同一脱敏副本上查询，防止回退时重读真实暗牌。真实权威规则、规则版本、存档格式和 ABI 均未变；AI 的实际选择可能因不再使用真实暗牌而改变。

回归对三种策略变体、节点预算 0／1／48 和两层搜索验证相同座位信息得到相同动作、节点数，且两侧动作均可提交、无模拟故障；另验证挂起机器／帧／选择／桌面未改、脱敏状态能够恢复，以及特殊牌区存在时不做该重分配。不是对全部挂起机器的信息边界证明。

## 性能对照

同一台 Apple M3 Max、macOS 27.0.1、`rustc 1.99.1 (58cf8f9c3 2026-10-13)`（[环境](environment.log)），同一 seed、挂起状态、策略变体 1、48 节点及两层搜索；debug 测试中先热身 32 次，再采集 500 次。旧源码使用干净导出及独立 target，修复后使用工作区 target；命令均为 `cargo test --offline -p engine-native --test nc10_pending_duel_privacy pending_duel_timing -- --ignored --nocapture --test-threads=1`。

| 版本 | p50 | p95 | p99 |
|---|---:|---:|---:|
| 干净 HEAD | 484 µs | 509 µs | 538 µs |
| 本批 | 505 µs | 535 µs | 573 µs |

[旧](latency-before.log)、[新](latency-after.log)均 **1/0/0**。单场景 debug 测量只供本机对照，不是质量、端到端或跨平台性能结论。

## 验证

| 命令 | 完整结果 |
|---|---|
| `CARGO_TARGET_DIR=<独立目录> cargo test --offline -p engine-native --test nc10_pending_duel_privacy -- --nocapture`（干净导出加测试） | **1 passed / 2 failed / 1 ignored，exit 101**；两项失败分别是动作受暗牌影响和脱敏结果保留真实暗牌（[日志](clean-head-repro.log)）。 |
| `cargo test --offline -p engine-native --no-fail-fast` | 重跑 **1587 passed / 0 failed / 72 ignored，exit 0**（[日志](engine-native-rerun.log)）；新增 1 ignored 为手动延迟采样。首次套件本身日志同为 **1587/0/72**（[日志](engine-native-tests.log)），但命令包装把返回码赋给 zsh 只读变量 `status`，包装报错，故另用普通变量重跑并记录退出码。 |
| `cargo test --offline -p engine-native-wasm -p content-native-loader -p game-server --no-fail-fast` | wasm **57/0/0**、loader **32/0/0**、game-server **360/0/0**，总 exit 0（[日志](product-tests.log)、[统计](test-counts.txt)）。 |
| `RUSTFLAGS="-D warnings" CARGO_TARGET_DIR=<独立目录> cargo check --offline -p engine-native --all-targets` | **exit 101，未通过**：rule-vm 两个 tag 656 的不可达分支。工作区及 `git archive HEAD` 干净导出、两个独立 target 均同样失败（[当前](gate1b-current.log)、[干净 HEAD](gate1b-clean-head.log)）；这是 B6 已登记根因，未修改或屏蔽严格门禁。门禁在该处停止，未证明其后的其他目标无警告。 |
| `RUSTFLAGS="-D warnings -A unreachable-patterns" CARGO_TARGET_DIR=<独立目录> cargo check --offline -p engine-native --lib` | exit 0（[日志](libcheck-scoped.log)）；只作为禁用一种警告后的库级补充检查，**不能替代上述全目标门禁**。 |
| `rustfmt --edition 2021 --check crates/engine-native/tests/nc10_pending_duel_privacy.rs` | exit 0（[日志](rustfmt.log)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | **1/0**；8623 个文件通过（[单测](source-unit.log)、[行数](source-lines.log)）。 |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py` | 1837 个文件、坏链 0；ADR 检查通过（[链接](local-links.log)、[ADR](adr.log)）。 |

## 未完成

此切片仅覆盖上述决斗形态；其他决斗形态、其他挂起机器、特殊牌区和 VM 私有状态仍可能依赖真实对手手牌，须完成机器引用自省及覆盖后再验收。AI 不同模式／座位／种子／预算的质量基线仍缺；NC10 保持部分完成。B6 导致严格全目标警告门禁未通过，已在干净 HEAD 独立复现，仍待规则版本裁决及修复。
