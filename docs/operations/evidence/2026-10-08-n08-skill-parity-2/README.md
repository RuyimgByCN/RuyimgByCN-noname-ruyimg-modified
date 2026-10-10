# P2/N08 逐技能语义对照批二：流离弃牌自选 + 转移范围（规则 v76，2026-10-08）

主稿 P2/N08「逐技能语义对照」第二批：消除流离（大乔）两处既有登记的
**选择类**差异，对齐上游
`archive/upstream/noname/apps/core/character/standard/skill.js` `liuli`
（本地参照，2026-10-08 读取）。前置：
[逐技对照批一](../2026-10-08-n08-skill-parity-1/README.md)。

## 上游语义核验（源码为准）

`liuli`（standard 大乔）：成为【杀】目标时 `chooseCardTarget` 一体询问——
`filterCard: cardDiscardable, position: "he"`（**手牌+装备**自选一张）+
`filterTarget: player.inRange(target) && target != trigger.source &&
targetEnabled(杀, source, target)`（转移目标须在**被转移者攻击范围内**、
非此杀使用者、且为该杀合法目标——空城不可）；整体可取消。content：
targets 交换（移除自己、推入新目标）后再 `discard(cards)`。

## 改动（分开陈述）

- **代码已实现（差异消除）**：
  - 弃牌自选：`liuli_discard_choose`（SelectCards，手牌+装备选项，
    min=0 可取消=不发动）取代「弃手牌首张」。
  - 转移范围：候选=存活、非自己、非此杀使用者、
    `computed_distance ≤ attack_range`（被转移者口径，±马/武器射程
    照常折算）、非空城（`liuli_candidates`/`liuli_candidate_ok`）。
  - 两段式（先选牌后选目标，语义等价一体询问）：第二段
    `liuli_transfer`（SelectPlayers，min=1 强制）；转移生效后弃置
    暂存牌（上游 content 顺序——牌在被转移者区域）。
- **状态接线**：`liuli_pending: bool` → `liuli_stage: u8`（1=弃牌挂起/
  2=目标挂起）+ `liuli_card: Option<CardId>`——快照变体、restore/
  enc/dec、canonical_hash、active_ask 两段各自派生（stage-2 候选就地
  计算）、青龙再杀重启/方天队列推进复位、a0 样例表同步。
- **登记差异消除**：基准 v75 节「剩余：流离弃牌自选」随之清账；
  内容差异段「流离涉及弃牌选择与范围」注记已于 v76 消除。

## 验收口径

- **回归已加入/更新**：`n08_liuli_transfers_sha_target`（两段流断言：
  选项=手牌+装备、候选=范围内合法目标（3 人局恰为 2 号）、取消语义、
  转移后新目标承伤/原目标无伤/被弃牌离场）；
  `p2_new_choice_points_per_submission_conservation` 适配两段应答
  （逐提交守恒）。
- **全量通过**：engine-native **382 passed / 0 failed / 1 ignored**
  （唯一 ignore=写轮眼规模诊断探针），单线程
  `cargo test --offline -p engine-native -- --test-threads=1`；
  依赖包 content-runtime + engine-ai + content-native-loader
  **62 passed / 0 failed**，DEPS_EXIT=0。明细见 [test-logs.txt](test-logs.txt)。
- **golden**：版本 75→76 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（无场景装配大乔/流离，语义轨迹不变，重录为
  纯版本位移）。
- **上游等价边界**：本批仅覆盖流离；「随机类」与「失去体力类」登记
  （面里金针/冰魄通/瘟疫等 G1 项）仍未对照，按主稿待做。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（10 文件，相对 noname-next 根）。
