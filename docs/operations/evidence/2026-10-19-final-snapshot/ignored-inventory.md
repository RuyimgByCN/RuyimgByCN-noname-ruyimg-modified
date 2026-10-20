# `engine-native` 的 **21 个 ignored 用例**逐条归类（2026-10-19）

命令：`cargo test -p engine-native -- --list --ignored`（21 条），原因取自源码 `#[ignore = "…"]`。

## 1 分类汇总

| 类别 | 数量 | 说明 |
|---|---:|---|
| **A. 已登记缺陷守护**（修好后应转正） | **17** | 每条的 ignore 文案都指向具体修复清单项或台账行 |
| **B. 待查根因**（未定项） | 3 | `jijiang` 代打链路、`luweiyan` 链杀嵌套、`zonghuo` 弃牌 |
| **C. 诊断探针**（人工运行） | 1 | `xly_state_growth_probe` |

> 注：A 与 B 有重叠（`jijiang`/`luweiyan`/`zonghuo` 三条**也在**修复清单里，但文案写「根因待查」）⇒ 上表按**文案措辞**分，不作语义裁定。

## 2 逐条

| # | 用例 | 原因（摘录）| 归口 |
|---:|---|---|---|
| 1 | `xly_state_growth_probe` | 「诊断探针：手动运行观察规模曲线」| **C** |
| 2 | `nc04_pause_and_resume` | 「NC04 待办：DSL 选择语句关键字待确认后补挂起→续跑→故障覆盖」| **A**（NC04 待办）|
| 3 | `upstream_dinvxuanshuang_recovers` | 帝女玄霜 `selectTarget -1` 应全员回 1 血 | **A**（第 39 项）|
| 4 | `upstream_liyutang_target_gains_the_card` | 目标应获得此牌，原生效果为空 | **A**（第 37 项）|
| 5 | `upstream_longxugou_steals_and_equips` | 获得后应自己装备 | **A**（第 36 项）|
| 6 | `upstream_mapodoufu_target_gains_the_card` | 同鲤鱼汤 | **A**（第 37 项）|
| 7 | `upstream_mizhilianou_heart_as_tao` | 红桃牌应可当【桃】用，实测 Rejected | **A**（第 41 项）|
| 8 | `upstream_qiankunbiao_target_needs_he_cards` | 目标无牌应被拒，原生更宽松 | **A**（第 35 项）|
| 9 | `upstream_shihuifen_blocks_cards_on_others` | 未出闪后应限制用牌，限制未生效 | **A**（第 38 项）|
| 10 | `upstream_shenenshu_replaces_hand_with_basics` | 效果未实现（源码自述『暂缓』）| **A**（第 44 项）|
| 11 | `upstream_jijiang_shu_ally_plays_sha_for_lord` | 代打链路未对上，**根因待查** | **B** |
| 12 | `upstream_rangjie_move_branch_must_draw_one` | 分支 1 未摸牌（上游无条件 `player.draw()`）| **A** |
| 13 | `upstream_baishouzhihu_opens_target_window` | **第 57 项**：`TriggerMeta` 注册 `PhaseEnd` 而 filter 要求 `begin==true` | **A**（第 57 项）|
| 14 | `upstream_cangchizhibi_target_discards` | 属转换缺口（未交付）| **A**（台账第 117 行）|
| 15 | `upstream_swd_hufu_dying_only` | 濒死支未复现（救援窗口未打开）| **A**（第 52 项）|
| 16 | `upstream_wuxing_shengke_three_branches` | 两支未复现（三次均 −1）| **A**（第 51 项）|
| 17 | `upstream_chenhuodajie_window_after_damage` | `damageEnd` 窗口未出现 | **A**（第 53 项）|
| 18 | `upstream_luweiyan_chain_kill_fires_on_first_target` | 询问已出现但虚拟杀无响应询问，**嵌套待查** | **B** |
| 19 | `upstream_zonghuo_discards_one_without_asking` | 确认后未弃牌，**根因待查** | **B**（第 33/34 项）|
| 20 | `nc12_probe_scans_seeds_for_dangling_reference` | **登记缺陷守护**：seed 44170 命中内核悬空引用 | **A**（NC12 缺陷）|
| 21 | `nc12_repro_seed_44170_step_10` | 同上（最小复现）| **A**（NC12 缺陷）|

## 3 判读（**如实**）

1. **没有一条是「因为跑不过就忽略」** ✅ —— 每条 ignore 文案都**指向具体修复项/台账行**，是**缺陷守护**（修好应转正）；
2. **17 条属 A 类**，其中多数（第 33–53 项）正是**核心修复清单**里**受未跟踪拆分阻塞**的条目
   ⇒ 这些用例**会在阻塞解除后自然转正**；
3. **3 条属 B 类**（`jijiang`/`luweiyan`/`zonghuo`）**根因未定** ⇒ 需要**独立排查**；
4. **1 条是诊断探针**（`xly_state_growth_probe`），**人工运行**，不属缺陷；
5. ⚠ **`ignored` ≠ 通过**：这 21 条代表**21 个已知未达标点**，其中 **2 条是内核级缺陷**
   （seed 44170 悬空引用）——**比功能面缺陷更严重**。

## 4 边界

- 本节只**归类**，**未修复任何一条**；
- 第 2 节「原因」为**源码摘录**，未逐条复核其准确性；
- `nc04_pause_and_resume` 的「DSL 选择语句关键字待确认」需**产品决策**（属方案未定项）。
