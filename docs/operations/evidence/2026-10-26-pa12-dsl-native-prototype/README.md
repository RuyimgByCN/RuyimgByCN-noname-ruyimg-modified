# PA12④「DSL/IR→原生」生成原型 + 两处桥接缺陷修复 + 迁移计划（2026-10-26/27）

- 执行时间：2026-10-26 **23:05:55 → 2026-10-27 00:11 +08:00**（`HEAD = 82fddba9`，PA07 批提交后领取；裁决入库 `421a04a8` 在前）；执行者 ZCode A 道
- 工作包：[台帐 PA12](../../../architecture/native-remaining-ledger.md#pa01pa12-性能架构调整2026-10-26) ④（用户 2026-10-26 22:51 裁决：先做 DSL/IR→原生生成原型并交迁移计划，批量迁移前再确认）；交付物：本证据 + [迁移计划](../../../architecture/native-dsl-migration-plan.md)
- 前置：PA08 真机扩面先行探测——`adb devices` 无设备、`xcrun devicectl list devices` 唯一物理 iPhone（iPhone17,2）为 `unavailable`、其余均为模拟器 → **真机未接入**，按指令转本项；台帐 PA08/PA11 行已登记

**一句话结论**：路线成立——sp 包真实 ported 技能 `olyinji`（隐绩，含 `choose(1,2)`）不写任何原生 Rust 宿主代码，经 content-compiler → `bridge_vm_skill_decls` 正式桥接 → engine-native 整局：结束阶段触发、挂起、作答续行、两分支落账、挂起中存档→恢复→续行（同一快照双分支）、旧线 vm_host 对同一字节码差分一致；原型同时暴露并修复两处桥接缺陷（活引擎首答丢失；DSL `phase_end` 被映成"任一阶段结束"每回合触发 5 次）。engine-native 全量 **1564/0/71**、engine-core **3838/0/1**、loader 32/0、wasm 52/0、game-server 358/0。盘点：7,614 技能中文件级近似可直接原生 579；迁移计划按 A–G 七类缺口分六批，**批量迁移前须用户确认**。

## 1 原型（代表技能与证据）

`content/original/ported/sp/olyinji.skill`（原文 `include_str!`）：

```
skill olyinji(2633842541) locked {
  trigger phase_end owner_is_target_not_unique_max_current_hp {
    let option = choose(1, 2);
    if option == 1 { effect recover_hp(self, 1); }
    if option == 2 { effect change_max_hp(self, 1); }
  }
}
```

选它的依据（[盘点](inventory.txt)）：事件 `phase_end` 已映射、滤镜 `owner_is_target_not_unique_max_current_hp` 在原生精确表（`OwnerNotUniqueMaxHp`：他人体力 ≥ 自己）、两个效果都在 `accept_effect` 允许面、包要求 ABI ≤ 753、含真实 choose。

| 测试（[原生](prototype-native.txt) 5/5、[旧线差分](prototype-legacy-differential.txt) 2/2） | 断言 |
|---|---|
| `pa12_dsl_choice_recover_branch` | 4 座、1 号持技、体力 3 与 2 号并列最高 → 结束阶段 `rvm_choice`，选项 `[Tag(1),Tag(2)]`；答 1 → hp 3→4，上限不变 |
| `pa12_dsl_choice_max_hp_branch` | 答 2 → 上限 4→5，体力不变 |
| `pa12_dsl_pause_save_restore_resume_both_branches` | 挂起中 `save()` → `restore_with_extensions` 两次：一次答 1（hp 4）、一次答 2（max 5）；原引擎再答 1 仍落账 |
| `pa12_dsl_filter_not_fired_when_unique_max` | 1 号唯一最高 → 整局无 `rvm_choice`，体力不变 |
| `pa12_dsl_fires_once_per_turn_at_end_phase_and_first_answer_lands` | 挂起时 `turn().phase == End`；首答即落账、无二次询问；下一回合结束阶段恰好再问一次 |
| `pa12_legacy_pause_options_match_native` | 旧线 `run_module` 挂起选项 `[1,2]` |
| `pa12_legacy_branch_effects_match_native` | 旧线 `resume_module` 1 → `RecoverHp{amount:1}`（hp+1）、2 → `ChangeMaxHp{delta:1}`（max+1），落账与原生一致 |

## 2 原型暴露并修复的桥接缺陷（代码在本批提交）

1. **活引擎首答丢失**（`crates/engine-native-kernel/src/rvm_bridge.rs` `VmMachine::run` Paused 分支）：返回 `Flow::WaitChoice` 的同时把 `asked` 置 false，下一 step 再次 `WaitChoice`，引擎已消费的首答被丢，只有第二次作答生效（实证：答 1 再答 2 → 仅上限 +1）。恢复路径 `restore_paused` 早已 `asked: true`，故既有 t8 挂起测试（只在恢复后的引擎上作答）与 nc04（空应答 drain）都没发现。修法：返回 WaitChoice 即 `asked = true`（与内核其它机器同定式）。连带：t8 测试助手 `np_settled` 曾把 rvm_choice 一并用首选项代答（实际依赖首答被吞），改为不代答 VM 挂起。
2. **`phase_end` 语义**：旧线 `TurnStateChanged{phase}` → `phase_{prepare,judge,draw,play,discard,end}`（进入阶段）、`PhaseEnded{phase}` → `phase_{draw,play,discard}_end`/`turn_ended`（离开阶段）；原生阶段帧按阶段逐一发 `begin/end`，装载器把 `phase_end` 映到 `EventKind::PhaseEnd`（任一阶段 `begin:false`）→ 每回合触发 5 次、时机也不对（原生宿主自己的酒尾注释把上游 `phaseEnd` 误读为"每个阶段结束"，同一误区）。修法：新增 `VmFilterPolicy::AtPhase{phase, inner}` 阶段门（`filter()` 头部递归评估），装载器阶段事件表：`phase_end`→PhaseBegin@End；`phase_{judge,draw,play,discard}`→PhaseBegin@X；`phase_{draw,play,discard}_end`→PhaseEnd@X；`phase_prepare` 原生无准备帧仍拒装（登记）；`phase_begin` 旧线产生者未核清，暂保持原映射（登记）。

## 3 盘点（[inventory.py](inventory.py) → [inventory.txt](inventory.txt)/[json](inventory.json)）

7,614 技能声明：事件 49 名/已映射 24；滤镜命中 1,463 次、推迟 2,358 次（708 名，`owner_alive` 754、`owner_is_source` 206 为命名不一致的低垂果实）；效果基础 2,782 次 vs 宿主解析 2,307 次/719 名；文件级近似可直接原生 579（yxs 91/111、offline 92/851、huicui 69/255、yijiang 55/281、sixiang 45/181…）。细表与批次见迁移计划 §2–§3。

## 4 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --no-fail-fast`（修 t8 助手后重跑） | **1564 / 0 / 71**（[逐二进制](engine-native-full-after-t8-fix.txt)；首轮 1562/2/71 的两处为 t8 助手代答问题，见 §2.1，[首轮汇总](regressions-run1.txt)） |
| `cargo test --offline -p content-native-loader` / `-p engine-native-wasm` / `-p game-server` / `-p engine-core` | **32/0/0、52/0/0、358/0/0、3838/0/1** |
| `cargo check --offline --workspace --all-targets` | 见提交信息 |
| `pnpm --dir apps/web-client test` / `typecheck` | 80/0/0；通过（本批无前端改动） |
| 行数 / 链接 / ADR 门禁 | 通过 |

## 5 边界

- 原型只证明一个代表技能的全链路；不代表 sp 包或任何来源整包验收；主动技（`active`）、view-as 转化、推迟滤镜、未映射事件与 719 个宿主解析原语仍是缺口（计划 §2）。
- 装载器对推迟滤镜仍是"静默不触发"——计划批①改为拒绝/告警。
- `phase_begin`/`phase_prepare`/`game_started` 等映射改动未在本批做，避免无验收扩面。
