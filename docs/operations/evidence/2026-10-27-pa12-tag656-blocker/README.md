# PA12 阻断 B6：VM 效果 tag 656 冲突

- 实际执行：2026-10-27 23:27:42 +0800 → 2026-10-27 23:29:01 +0800（开工与结束各运行 `date '+%F %T %z'`）；源码基准 `0cdce88122a2666171b7be423104daaf51bbeece`（[开始](start-time.txt)、[结束](end-time.txt)、[源码](source-head.txt)）。本批只记录反例与裁决边界，未修改规则、协议或版本。

## 当前反例

[效果编号表](../../../../crates/rule-vm/src/bytecode/effect_tag.rs)中 `GAIN_OTHER_SKILL_PILE_CARDS`（英逝）与 `CHOOSE_OTHER_TARGETS_BY_SKIPPED_PHASE_COUNT_DAMAGE_DRAW`（平寇）均为 `656`；[分派代码](../../../../crates/rule-vm/src/vm/effects/resolve_qixing.rs)先匹配平寇，英逝分支不可达，编译器亦给出 `unreachable_patterns`。按该文件直接十进制 `u16` 常量扫描，656 是唯一重复值，772 暂无对应常量；[扫描脚本](inventory.py)与[结果](inventory.log)限定于这一口径。

实际包的英逝模块 `sp@0.1.0` ABI 745 和平寇模块 `onlyOL@0.1.4` ABI 757 都包含 `HostEffect 656`。将[测试](effect_656_collision.rs)单独叠到 `git archive HEAD` 的[干净导出](clean-path.txt)，独立 `CARGO_TARGET_DIR` 运行：

```bash
CARGO_TARGET_DIR=<导出>/target-independent cargo test --offline --manifest-path <导出>/Cargo.toml -p rule-vm --test effect_656_collision -- --nocapture
```

结果 **0 passed / 1 failed / 0 ignored，exit 101**：真实 `yingshi_die_death` 在收集宿主上实际产生 `ChooseOtherTargetsBySkippedPhaseCountDamageDraw { player: PlayerId(2), amount: 0, ... }`，而非预期 `GainOtherSkillPileCards`（[完整日志](collision-red.log)）。这是 VM 实际模块反例；本批没有宣称原生或旧线完整对局已验证。[原生 VM 提交层](../../../../crates/engine-native/src/rvm_ops.rs)有英逝效果处理，因此纠正分派会改变相关规则行为。

## 裁决与未完成项

用户已明确同意：保留英逝 tag 656，平寇改为 772；平寇效果最低 ABI 758，全局原生 ABI 保持 753；ABI 754–757 且含歧义 tag 656 的旧包及冻结存档明确拒绝；保留 onlyOL 0.1.4 原件并生成 0.1.5。上述方案**尚未实施**，仍须覆盖实际包、存档、调用方和回归。

另已询问是否授权原生规则版本从 v322 升至 v323，并在核对实际影响后重录 golden、登记规则基准及明确旧档策略；截至本批结束**尚未收到答复**。现有原生规则 v322、存档格式 v105；规则版本改变会影响旧快照的版本匹配，不可在未裁决前开工该修复批次。此项不代表已批准存档格式升号或全局原生 ABI 升号。

## 文档验证

| 命令 | 结果 |
|---|---|
| `python3 -B docs/operations/evidence/2026-10-27-pa12-tag656-blocker/inventory.py` | 指定常量表与两份实际包扫描完成（[结果](inventory.log)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | **1 passed / 0 failed**；8622 个文件通过（[单测](source-unit.log)、[行数](source-lines.log)）。 |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py` | 本地链接和 ADR 检查通过（[链接](local-links.log)、[ADR](adr.log)）。 |

完整修复、版本门禁、旧包兼容／拒绝、上下游行为差分和全量回归仍待实施；此登记不改变 PA12 或 N01 的完成状态。
