# PA12 迁移批⑤首步（D 道 N01 原生化起步）：SkipCurrentPhase 接线 + 发现 PhaseBegin 触发窗时序缺陷（2026-10-27）

- 执行时间：2026-10-27 **14:34:18 → ≤15:03（原记 15:10 晚于本批提交 15:03，2026-10-27 扫描按提交时间更正） +08:00**（`HEAD = 1a8b2476`；按已声明默认路线代理开工批⑤——裁决问询未获答复；真机复测仍 0 台接入）；执行者 ZCode A 道
- 工作包：[迁移计划](../../../architecture/native-dsl-migration-plan.md) §3 批⑤（N01 界朱然/界朱桓原生化首步）；台帐 [UP37-N01](../../../architecture/native-remaining-ledger.md)

**一句话结论**：批⑤从 N01 真实包（onlyOL.npack.json）切入，先做**技能级 tag 清单**（36 模块直方图见 §1）：N01 五技能中 `ol_fenli`（奋励三窗口）只依赖 `SKIP_CURRENT_PHASE`——按既有跳过旗标接线（`turn.skip_draw_phase`/`turn.skip_play_phase`/`player.skip_discard_phase`，分别同兵粮寸断/乐不思蜀/入梦口径），`ol_jie_danshou` 需 `STORAGE_SET`+`RESOLVE_OL_JIE_DANSHOU(771)`、`ol_pingkou` 需 `CHOOSE_OTHER_TARGETS_BY_SKIPPED_PHASE_COUNT_DAMAGE_DRAW(656)`（选择链族）。**实测抓到第六个桥接/内核缺口**：VM 机器对 `PhaseBegin` 的执行窗**晚于该阶段入场行为**（奋励摸牌窗口的确认门打开时摸牌已完成，手牌 3≠1）——`skip` 置旗标对当前阶段无效，登记计划 §2 K（需前置窗调度，与 `game_started` 开局帧同族）；接线保留（前置窗落地后即正确），测试锁「三窗口可装载桥接」面并注明时序修复后补整局断言。

## 1 N01 技能 tag 清单（onlyOL.npack.json 真实字节码直方图）

| 技能（模块） | 事件/滤镜 | 依赖 tag | 原生状态 |
|---|---|---|---|
| ol_fenli（三窗口 phase_draw/play/discard） | tied_max_hand/hp/equip_nonempty optional | SKIP_CURRENT_PHASE(81) | **允许面+旗标接线（本批）**；受 §2 K 时序限制 |
| ol_jie_danshou（界朱然·丹守） | phase_play（STORAGE_SET 记账）+ 主动技段（RESOLVE_OL_JIE_DANSHOU 771） | 649/771 | 未接（771=宿主解析状态机） |
| ol_pingkou（平寇） | phase_end skipped_phase_count_positive optional | 656（选择链族） | 未接（依赖 §2 J 基建） |
| ol_zhuhuan（界朱桓） | **不在 onlyOL 包内**（36 模块无 zhuhuan；旧线测试在 engine-core tests） | — | 待考：zhuhuan 的 DSL/宿主位置另查 |
| 包其余（dclihuo/olfuli/oljingce…） | — | 基础 tag + 若干 choose_*/resolve_* | 部分已可承载 |

包 manifest `rule_vm_abi_version=757` 仅因个别模块（如 ol_wenren 757）抬高门槛——**按模块过滤装载可绕过整包 ABI 门**（本批 fenli 三模块实测可装载；整包装载仍需升号+补齐全部原语）。

## 2 变更与验收

- `rvm_ops.rs`：`SkipCurrentPhase` 允许面（座位校验）+ commit 臂（按 `turn.current==p` 与当前阶段置三旗标；判定/结束阶段无旗标静默空操作）；头注更新。
- 测试 `pa12_batch5_fenli_bridges_with_skip_wiring`：**真实 npack 按名过滤装载**（非手写 DSL）→ 三窗口桥接 3/3 + accept 通过；时序缺陷的整局红灯实测过程记录于本 README（确认门 phase=Draw 时手牌已 3）。

## 3 时序缺陷（§2 K，登记）

- **现象**：PhaseBegin(Draw) 的 VM optional 确认门打开时，该阶段入场行为（摸牌 2）已执行——VM 机器执行窗晚于入场行为（`commit_phase` 先 emit Phase{begin:true} 再同段调 `begin_draw`，VM 机器的实际调度点在其后）。
- **影响**：`skip_current_phase` 对 Draw/Play/Discard 当前均不能按上游语义生效（Play 旗标还须摸牌末消费=错回合）；fenli 三窗口原生化被阻。
- **修复方向**：把 VM 机器对 PhaseBegin 的触发排到前置窗口（BeforeTriggers，即入场行为之前），或为 PhaseBegin 提供专用前置帧——属内核调度改动，与 `game_started` 开局帧缺口同族；留批⑤下一子批（先修时序再回填 fenli 整局断言）。

## 4 回归

[b5 汇总](../2026-10-26-pa12-dsl-native-prototype/b5-regressions.txt)：engine-native、loader、wasm、game-server、engine-core 全量 + workspace `--all-targets` + 前端 + 三门禁（见提交信息）。
