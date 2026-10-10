# P2/N08 逐技能语义对照批一：杀结算目标侧选择类（规则 v75，2026-10-08）

主稿 P2/N08「逐技能语义对照」第一批：消除两处既有登记的**选择类**差异，
对齐上游 `archive/upstream/noname/apps/core/card/standard.js`（本地参照，
2026-10-08 读取）。前置：[批③-c 花色对齐](../2026-10-07-n08-suit-align-3c/README.md)。

## 上游语义核验（源码为准）

- **雌雄双股剑** `cixiong_skill`：目标无手牌 → 来源直接摸 1；否则
  `trigger.target.chooseToDiscard({prompt: 弃置一张手牌，或令X摸一张牌})`
  ——目标自选弃哪张，**可取消**，取消 = 来源摸 1。
- **寒冰剑** `hanbing_skill`：`trigger.cancel()` 防止伤害后，
  `player.discardPlayerCard({target, position:"he", forced:true})` **两次**
  ——每次由**目标**从手牌+装备自选一张强制弃置，第二问前复查
  `hasDiscardableCards("he")`，无牌可弃即止。

## 改动（分开陈述）

- **代码已实现（差异消除）**：
  - 雌雄：Confirm 询问改为目标 SelectCards（手牌选项，min=0 可取消，
    `cixiong_discard_choose`）；恢复臂按应答牌弃置，空应答/过期应答
    回退=令来源摸 1。
  - 寒冰：来源确认后转 `hanbing_pending`（=2）状态，逐次开目标
    SelectCards（手牌+装备，min=1 强制，`hanbing_target_discard`）；
    次数用尽或无牌可弃即止，随后照常跳过伤害/推进多目标队列。
- **状态接线**：ShaResolveMachine 新增 `hanbing_pending: u8`——
  快照变体尾部追加、restore/enc/dec 对称、canonical_hash 入域、
  advance_target 逐目标复位；a0 样例表同步（`hanbing_pending: 2`）。
- **登记差异消除**：规则基准 v67/v72 两处「登记差异」注记「已于 v75 消除」。

## 验收口径

- **回归已加入/更新**：`cixiong_opposite_gender_discard_or_draw`
  （自选指定牌、取消令使用者摸 1、同性不触发三路径）；
  `hanbing_prevents_damage_discards_two_cards`（目标先弃注入的 +1 马
  再弃手牌的选序断言、第二次询问选项收敛、放弃路径伤害照常）。
- **定向通过**：上两测试 + `a0_tag_registry` 7/7。
- **全量通过**：engine-native **382 passed / 0 failed / 1 ignored**
  （唯一 ignore=写轮眼规模诊断探针），单线程
  `cargo test --offline -p engine-native -- --test-threads=1`，EXIT=0；
  依赖包 content-runtime + engine-ai + content-native-loader
  **62 passed / 0 failed**，EXIT=0。明细见 [test-logs.txt](test-logs.txt)。
- **golden**：版本 74→75 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2；两武器无场景装备，语义轨迹不变（重录为纯
  版本位移，非行为变化）。
- **上游等价边界**：本批仅覆盖雌雄/寒冰两技能的目标侧选择语义；
  流离弃牌自选（选择类）、G1 登记的随机/失去体力类差异（面里金针/
  冰魄通/瘟疫等）仍未对照，按主稿待做。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（10 文件，相对 noname-next 根）。
