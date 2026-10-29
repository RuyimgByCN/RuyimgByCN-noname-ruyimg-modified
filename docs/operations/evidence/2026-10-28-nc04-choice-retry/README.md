# NC04：VM 非法回答、恢复后故障与发现的恢复差异

- 实际执行：2026-10-28 10:24:18 +0800 → 2026-10-28 10:31:12 +0800（[开始](start-time.txt)、[结束](end-time.txt)）；源码基准 [`f74724d12b22c10ccd98806de6b3e0cf7dea1510`](source-head.txt)。规则 v323、存档 v105、全局原生 ABI 753 不变。本批只新增测试与证据，不改变产品代码或协议。

## 已验证的边界

沿用已有手工字节码 `HOST_CHOICE` 夹具，在 VM 选择待答时分别提交空选择、越界 tag、错误选项类型、超额选择、错误座位和错误 choice ID。每次拒绝均核对完整快照（含随机流和去重水位）、状态哈希、日志和开放选择不变；将快照实际编码解码并恢复之后重验，最后使用同一个先前被拒绝的去重键提交合法 token，原局与恢复局的核心状态和结算快照一致。此测试的非平凡拒绝断言经干净 HEAD 变异验证：仅关闭选择集合校验后，非法选择被接受、定向测试 **0/1/0**（[变异日志](mutation.log)）。

另对挂起存档恢复后继续执行非法宿主效果：原局和恢复局均返回 `VmFault`；同一去重键再次提交仍确定性故障，完整快照、随机流、哈希、日志和选择不变，故障槽清空。**本测试不证明故障技能名和事件上下文恢复一致**；下述反例已证实技能名不一致。

## 新发现：VM 挂起恢复会改变规则语义（B7，未修复）

定向测试最初要求恢复后的 `VmFault` 保留技能声明名，工作区及仅叠加该测试的 `git archive HEAD` 导出均 **3/1/0**（[首次](target-initial.log)、[干净复现](clean-reproduction.log)、[探针](fault-identity-probe.rs)）。恢复前为 `nc04_restored_fault`，恢复后却为模块名 `nc04-restored-fault`。检查 [VM 机器恢复](../../../../crates/engine-native-kernel/src/rvm_bridge.rs)发现快照恢复从模块名重建技能名，并将触发上下文、限次消费和命中计数重置。

进一步在同一干净 HEAD、独立 target 中以另一条[探针](storage-divergence-probe.rs)驱动 `HOST_CHOICE` 后的 `STORAGE_SET`，原局存储为 `{nc04_storage_skill#7: 11}`，恢复局额外出现 `{nc04-storage-module#7: 11}`，测试 **0/1/0**（[日志](storage-divergence.log)）。这是实际状态差异，不能按仅诊断差异关闭。探针可追加到本批的 `nc04_vm_resume.rs` 中重跑；故障身份探针则替换同名测试。

此问题的修复涉及规则版本及快照恢复语义，已向用户单独请求授权；答复前不启动该修复。当前回归只断言故障技能名非空，代码以 `shortcut:` 标明后续须改为身份一致性断言。续行来源认证、异常生成的 VM token、其他故障／恢复矩阵与真实产品链路也未由本批完成。

## 验证

| 命令／范围 | 完整结果 |
|---|---|
| `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=<独立 target> cargo test --offline -p engine-native --test nc04_vm_resume -- --nocapture` | **4/0/0**，exit 0（[日志](target-final.log)）；全量后仅更正测试文件中原有的两处过期注释，定向重跑仍 **4/0/0**（[日志](target-after-comment.log)）。首次失败和干净 HEAD 复现见上。 |
| `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=<新独立 target> cargo test --offline -p engine-native --no-fail-fast` | **engine-native: 1612 passed / 0 failed / 73 ignored; 331 test targets**，exit 0（[最终日志](native-full-final.log)、[计数](native-counts.txt)、[脚本](count-native.py)、[目录](full-target.txt)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py`；`python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py`；`git diff --check` | 结果见[自检](source-unit.log)、[行数](source-lines.log)、[链接](local-links.log)、[ADR](adr-index.log)、[差异](diff-check.log)及[暂存](staged-check.log)。 |

首次原生全量在链接阶段因系统错误 28（磁盘空间不足）退出（[日志](native-full.log)）；从干净 `git archive HEAD`、无本批代码改动并使用全新 target 重跑，在链接阶段同样因错误 28 退出（[复现日志](clean-full-reproduction.log)、[被临时编辑文件恢复为 HEAD 后核对](clean-source-check.txt)）。删除本批创建的临时导出和 target 后释放至约 7.6 GiB，再以另一全新 target 重跑全量。

NC04 仍为部分；保留原有八份未提交文档改动，台帐与进度仅暂存本批追加行。

两次磁盘失败日志仅去除空白行的尾随空格，错误内容和退出状态保持原样。
