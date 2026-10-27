# PA12 批⑤-2：PhaseBegin 前置窗口修复（计划 §2 K）+ 奋励整局断言回填 + golden 重录（2026-10-27）

- 执行时间：2026-10-27 **15:07:01 → 16:10 +08:00**（`HEAD = 6d867a91`；按已声明默认路线代理开工；真机复测仍 0 台接入）；执行者 ZCode A 道
- 工作包：[迁移计划](../../../architecture/native-dsl-migration-plan.md) §2 K 修复；台帐 PA12 批⑤-2 / UP37-N01

**一句话结论**：§2 K 修复落地——PhaseBegin 帧从 Commit 起步改为 **BeforeTriggers 起步**（三处协同：`resolution.rs` 起步表、`machine.rs` 的 `Ctx::emit` 起步表、`next_stage` 推进表 BeforeTriggers→Commit→AfterTriggers→Intrinsic→Done），且**前置窗只前移 VM 技能**（`SkillBehavior::is_vm_skill` 探针，默认 false、`VmSkillBehavior` 覆写 true）：宿主 PhaseBegin 技能（61 个）保持在入场行为之后的原执行点。全量回归三阶段定位：①无门控=16 宿主技能/golden 失败（前移破坏「摸牌后」语义）；②after 段全静默=31 失败+1 挂死（宿主技 after 派发被砍）；③**VM-only 门控 + after 段只跳过 VM（防重复入队）= 仅 golden 3 场景哈希差异**——根因=状态哈希域含帧栈 `stage/planned`（engine/hash.rs:601-608），快照点落在阶段推进中途的场景哈希随调度结构变化，行为等价，**golden 3 处重录**（注记入测试，规则/存档版本不变——调度内部态不入存档语义面）。奋励（ol_fenli 真实 npack）整局断言回填通过：确认门开于 Draw（入场行为之前）、确认后摸牌被跳过（手牌与确认时一致、旗标被消费、相位越过 Draw）。

## 1 变更

| 文件 | 变更 |
|---|---|
| `resolution.rs` | 起步表 `PhaseBegin → BeforeTriggers`；`next_stage` 补 `(PhaseBegin, BeforeTriggers)→Commit→AfterTriggers→Intrinsic`；`plan_triggers` 门控——PhaseBegin 帧 before 段只入队 VM 技能、after 段跳过 VM（防重复），宿主技两窗原样（注释含 16/31 失败实测依据） |
| `machine.rs` | `Ctx::emit` 起步表 PhaseBegin 同步 BeforeTriggers（E03 铁律同位）；（探针在 catalog） |
| `catalog.rs` | `SkillBehavior::is_vm_skill()` 默认 false |
| `rvm_bridge.rs` | `VmSkillBehavior::is_vm_skill() = true` |
| `tests/replay_scenarios.rs` | golden 3 处重录（shouyi_dynamic_skill/discard_to_limit/judge_phase_delayed_tricks）+ 变更注记 |

## 2 验收

| 测试 | 断言 |
|---|---|
| `pa12_batch5_fenli_bridges_with_skip_wiring`（真实 onlyOL npack） | 前置窗判据：确认门开于 Draw（VM 机器在入场行为之前）；确认后 skip 生效——手牌与确认时一致（摸牌被跳过）、`skip_draw` 旗标被消费、相位越过 Draw |
| `replay_scenarios_match_golden` | 重录后全绿 |

**三阶段回归定位过程（全量 engine-native）**：无门控 1565/16（宿主技前移破坏摸牌后语义：tuxi/yingzi/gwent/longxue/qingzheng…）→ after 全静默 317/30+挂死（宿主 after 派发被砍）→ VM-only 门控 1581/1（仅 golden）→ 重录后 **1582/0/71**。

## 3 回归

- engine-native **1582/0/71**（+1 fenli 断言）；其余四 crate 见 [b52 汇总](../2026-10-26-pa12-dsl-native-prototype/b52-regressions.txt)。
- golden 重录依据：哈希域含帧栈 stage/planned（结构性），无 VM 场景行为等价；规则 v322/存档 v105 不变；变更注记写入测试与[规则基准待补行]（见提交信息，规则基准同步登记）。

## 4 后续（批⑤下一子批）

jie_danshou（STORAGE_SET + RESOLVE_OL_JIE_DANSHOU 771 宿主解析状态机）→ pingkou（656，依赖 §2 J 选择窗基建）→ R01/R03 → ol_zhuhuan 位置考证 → 升 `RULE_VM_ABI_VERSION` 整包重装。
