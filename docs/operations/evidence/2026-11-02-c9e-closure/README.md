# C9e 17 组反例＋chain 出生 P1 的逐项关闭证据收口（2026-11-02/03）

对应[进度主稿](../../../../architecture/native-progress.md)风险③：C9e 登记的 17 组反例（F01–F17）与 chain 出生 P1 在 C9f–C9K 轮次已逐项修复并入 `f5e560db`，但红绿证据散在私有目录未入库。本批把关键红绿日志与三份新证据入库并补两个缺口（F11 双分支断言、F13 红日志复现）。

## 证据来源入库状态

| 轮次 | 证据目录 | 状态 |
| --- | --- | --- |
| C9e 登记（41 项只读对照→17 组＋P1） | [2026-10-31-c331-phase9e-source-predicates](../2026-10-31-c331-phase9e-source-predicates/README.md) | main 已跟踪（原有） |
| C9f 原子修复＋chain P1（红 `atomic-red2.log` 12 项 FAILED；绿 `atomic-green1..10`；全量 `checks-final2` 1845/1/75） | [2026-10-31-c331-phase9f-atomic-source](../2026-10-31-c331-phase9f-atomic-source/README.md) | **本批入库**（原随 stash 未跟踪部分） |
| C9g 真实调用＋普通判定（红 `invocation-red/jiding-group-red/judge-consumers-red/f14-known-enemy-red`；绿 `invocation-green1-3/judge-green1-4`；终局 `final6-validation` 1848/9/76） | [2026-10-31-c331-phase9g-invocations](../2026-10-31-c331-phase9g-invocations/README.md) | **本批入库** |
| 调用契约方案（五组 FSM 设计与来源审计） | [2026-10-31-c331-invocation-plan](../2026-10-31-c331-invocation-plan/README.md) | **本批入库** |
| 9f4 纯文档发布记录 | [2026-10-31-post-c9e-doc-publication](../2026-10-31-post-c9e-doc-publication/README.md) | **本批入库** |
| C9h–C9K | 仍在私有目录 `…/T/noname-c331-phase7-ktqqj1fb/evidence/phase9h(1.7G)/phase9i(492M)/phase9j/phase9k` 与 `~/.pi` 会话记录 | 未入库（体积大；C9L 终局日志已全量入库可复验，见下行） |
| C9L | [2026-11-02-c331-c9l-golden-chain-xuanying](../2026-11-02-c331-c9l-golden-chain-xuanying/README.md) | **本批补齐缺失的 01–20、22、37、41、42、46–48 号日志**（原只跟踪 21、23–36、38–40、43–45、49–56） |

## 逐项状态（main＝`f5e560db` 合并树）

路径缩写：`9f`＝[phase9f/original](../2026-10-31-c331-phase9f-atomic-source/original/)，`9g`＝[phase9g-invocations/original](../2026-10-31-c331-phase9g-invocations/original/)；测试均在 `crates/engine-native/tests/`（经 `tests/groups/nc_rounds.rs` 注册）。

| 编号 | 内容 | 红证据 | 绿证据 | main 测试 |
| --- | --- | --- | --- | --- |
| F01 | 绝情自伤（LoseHp 替换） | 9f/atomic-red2.log | 9f/atomic-green*.log | c331_atomic_source_repair::c9f_jueqing_self_source_replaces_damage_before_armor |
| F02 | 传播虚拟杀有效杀种 | 9f/atomic-red2.log | 9f/atomic-green*.log | c9f_virtual_sha_chain_keeps_effective_kind_through_actual_edge 等 |
| F03 | 招祸自伤取消 | 9f/atomic-red2.log | 9f/atomic-green*.log | c9f_zhaohuo_self_cancel_precedes_begin2 |
| F04 | 书锤 hit 旗标 | 9f/atomic-red2.log | 9f/atomic-green*.log | c9f_shuchui_already_hit_never_readmits |
| F05 | 吸血自来源回血 | 9f/atomic-red2.log | 9f/atomic-green*.log | c9f_xixue_self_source_recovers_once |
| F06 | 炮台全宽扣减 | 9f/atomic-red2.log | 9f/atomic-green*.log | c9f_paotai_full_width_*（3 项） |
| F07 | 青龙可选 he 选择 | 9h（私有；qinglong-*-red） | 9h/9i green；C9L 49-cert | c9h_qinglong_*（6）＋c9i_qinglong_*（6） |
| F08 | 阴贼 he 随机弃牌 | 9f/atomic-red2.log | 9f/atomic-green*.log | c9f_yinzei_*（2） |
| F09 | 瘴雾选择 FSM | 9g/invocation-red.log；9i/wide-host-reader-red | 9g/invocation-green1；C9L 49-cert | c9g_zhangwu_*（2）＋c9i_supported_unique_* |
| F10 | 反噬 terra 到期 | 9f/atomic-red2.log | 9f/atomic-green*.log | c9f_zlfanshi_terra_*／c9f_terra_expires_* |
| F11 | 阿鼻随机火/雷 | 9f/atomic-red2.log；**本批 [02](02-f11-mutation-red.log)** | 9f/atomic-green*.log；**本批 [01](01-f11-abi-branch-green.log)** | c9f_abi_nature_draw_and_source_are_real（本批强化） |
| F12 | 远策选项/判定 | 9g/judge-consumers-red.log | 9g/judge-green1-4.log | c9g_yuance_*（2）＋Tiandao |
| F13 | 极叮整牌后移除 | 9g/invocation-red＋jiding-group-red；**本批 [03](03-f13-c9l-tongji-receipt-red.log)（C9L 收据项红复现）** | 9g/invocation-green1；C9L 49-cert（c331_jiding_proofs 2/2） | c9g_jiding_*（3）＋c9h/c9i/c9j/c9l 合计 9 |
| F14 | 催毒敌我＋具名抽牌 | 9g/f14-known-enemy-red；9i/kingdoms-red | 9i/kingdoms-green（130/0） | c9f_cuidui_*（4）＋c9g/c9i_kingdoms_* |
| F15 | 妒忌另一来源排除 | 9f/atomic-red2.log | 9f/atomic-green*.log | c9f_duqu_other_enabled_holder_is_excluded |
| F16 | 极典目标/判定 | 9g/judge-consumers-red；9h/jidian-log-order-red | 9h green | c9g_jidian_*（2）＋c9h |
| F17 | 冥顽自来源 | 9f/atomic-red2.log | 9f/atomic-green*.log | c9f_minwan_self_source_initializes_real_lock_set |
| P1 | chain 出生校验 | 9f/atomic-red2.log | 9f/atomic-green*.log | c9f_chain_birth_rejects_a_still_queued_recipient |

## 本批新增

1. **F11 双分支断言补齐**：原测试只证「消耗一次同型随机」＋子伤害 `DamageDealt` 存在；现给目标挂「火神」（免疫火伤＋回复），跨种子循环直到火／雷两分支都出现——火分支断言 `DamagePrevented{target:0}` 且 HP 不降、雷分支断言子伤害真实结算且 HP −1（[01](01-f11-abi-branch-green.log) 绿）。变异（属性固定为雷）在 seed 14012 火分支断言红（[02](02-f11-mutation-red.log)），恢复后绿。
2. **F13 红日志复现**：把 `kernel/jiding_card_changes.rs` 的同疾付款验证还原为 C9L 前的「不查收据」旧版 → `c9l_tongji_payment_proof_is_required_on_both_paths` 红（[03](03-f13-c9l-tongji-receipt-red.log)），恢复后 2/2 绿。补上 C9L README 只有文字记述、无独立红日志文件的缺口。
3. **C9L 日志补齐**：`2026-11-02-c331-c9l-golden-chain-xuanying/` 补入缺失的 01–20、22、37、41、42、46–48 号日志（与私有 `…/T/noname-c331-phase7-ktqqj1fb/evidence/c9l-20261102T073042Z` 逐字节相同），56 号日志全量在库。

## 仍未关闭（不因本批豁免）

各轮 disposition 列的追加控制项在 main 均未完成，继续挂在 C331 闭环待办：F02 别名面、F04 直接虚拟杀首/次资格与续期、F08 混合 he、F10 续期、F11 之外的两分支全生命周期、F12/F13/F16 全生命周期矩阵、F14 国战 Jun／跨模式、F15 别名、12 项 provider 行、真正 ChangeHp 与共享 DeathFlow／PhaseBegin／死亡压窗／救援到期桥／indexed／forceDie／out／hidden／filterStop／rescue／operation LossEnd，以及注册 262/290 剩 28（16 译名上下文＋12 机制）。
