# PA12 批⑤-3：jie_danshou 依赖面确认 + STORAGE_SET 写路径接入（2026-10-27）

- 执行时间：2026-10-27 **16:20:48 → 16:55 +08:00**（`HEAD = 8c669f35`；裁决问询未获答复按默认路线代理开工；真机复测仍 0 台接入）；执行者 ZCode A 道
- 工作包：[迁移计划](../../../architecture/native-dsl-migration-plan.md) 批⑤第三步；台帐 UP37-N01

**一句话结论**：先做**依赖面确认**（纠正本线此前判断）：`ol_jie_danshou` 的 `RESOLVE_OL_JIE_DANSHOU(771)` **挂在主动技段**（`active play_phase`，npack `active_index`）且其旧线实现首步即 `open_choice`（选项→目标→牌→弃置四段询问机，engine-core `effect/primitives/ol_jie_danshou.rs` 320 行）——即它**同时依赖批⑥ 主动技桥与 §2 J 效果级选择窗挂起**两项基建，不能作为独立小步先行；本批转而落地其可独立部分——**`STORAGE_SET` 写路径**（tag 649 → `EffectRequest::StorageSet`）：新增 `vm_aux.storage: BTreeMap<String,u32>`（键 `"{技能名}#{key}"`，VM 桥无实例 id 按执行中技能名；value=0 删除）并全链接线（state 字段、archive 编解码、状态哈希、`is_empty`），**存档/哈希域变化**——`vm_aux.storage` 空表编码零字节（同既有 VmAux 空表约定），既有对局无该字段写入→golden 不变（全量回归验证）；真实 npack 验收：丹守 `phase_play` 触发段（`storage_set(self_skill,1..4,0)`）在批⑤-2 前置窗中于**出牌阶段开始时**清零残值。

## 1 依赖面确认（jie_danshou 拆解）

| 面 | 事实 | 结论 |
|---|---|---|
| 入口 | `active play_phase`（npack `active_index`，engine-native `ActivateSkill` 协议已备但 VM 桥 `activate()` 未实现） | 依赖**批⑥ 主动技桥** |
| 主体 | 旧线 `resolve_ol_jie_danshou` 首步 `open_choice`（选项 1..4），后续目标/牌/弃置共四段 `ContinuationKind::ResolveOlJieDanshou` 状态机 | 依赖**§2 J 效果级选择窗挂起**（VM Halt 后效果落地时开窗续行） |
| 记账 | `storage_set(self_skill, k, 0)` 清零挂 `phase_play` 触发段（已可承载）；选项消耗 `storage_set(skill, option, 1)` 在状态机 stage1 写 | **本批接入写路径**（读侧 StorageGet 类原语待状态机接入时对齐） |

## 2 变更

- `state.rs`：`VmAux.storage`（BTreeMap<String,u32>）+ `is_empty` 纳入。
- `archive/auxiliary.rs`：enc/dec（len 前缀 + str + u32）；空表零开销。
- `engine/hash.rs`：`push_vm_aux` 尾部纳入 storage（同 drawn 段式）。
- `rvm_ops.rs`：允许面 `StorageSet`（无条件）+ commit 臂（键 `"{executing}#{key}"`，value=0 删除）；头注更新。

## 3 验收与回归

- 测试 `pa12_batch5_3_storage_set_clears_on_phase_play`（真实 onlyOL npack）：预置脏键 `{技能名}#2=1` → 推进到 1 号出牌阶段开始（批⑤-2 前置窗）→ 键被 `storage_set(...,0)` 清除。全测试文件 **6/6**。
- 存档/哈希域说明：`VmAux.storage` 为**新增状态字段**——空表不改变既有对局字节与哈希（encode 长度前缀 0、hash 段长度 0；golden 全量回归验证）；字段语义（技能私有存储）与旧线 `player.storage.<skill>[key]` 对齐（实例 id→技能名，登记差异）。存档 v105 **未递增**：VmAux 已有长度前缀式追加编码（marks/piles/… 同族），旧档读至旧字段尾部即 storage 空表，兼容（`restore` 全量测试通过佐证）。
- 回归（[b53 汇总](../2026-10-26-pa12-dsl-native-prototype/b53-regressions.txt)）：五 crate 全量 + workspace `--all-targets` + 前端 + 三门禁见提交信息。

## 4 后续

jie_danshou 主体=批⑥（VM 主动技 `activate()` 启动 active_index 模块）+ §2 J（效果级选择窗挂起：VM Halt 提交时把 Resolve 类效果转为宿主侧续行机器入 parks）——两件基建建议合并为单批并先立 ADR（涉及 VM 桥两层提交协议扩展）；pingkou（656）同依赖 §2 J。R01/R03 走宿主原语路线不受此阻。

## 5 首轮回归失败与修复（如实记录）

首轮全量：engine-native 1483/100、wasm 44/8、game-server 330/29——**存档解码 UnexpectedEof/BadTag**：storage 解码段误用 `r.u32()?` 读长度前缀（`w.len()` 写入的是 len 编码，须用 `r.len()?` 读，与 marks/piles/… 段同法）。修正后全量全绿：engine-native **1583/0/71**、wasm 52/0、game-server 358/0（loader 32/0、engine-core 3841/0/1 首轮已绿）。首轮失败汇总保留于 [b53-regressions.txt](../2026-10-26-pa12-dsl-native-prototype/b53-regressions.txt)。
