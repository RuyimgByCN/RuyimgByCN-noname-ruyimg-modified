# ADR 0389：限定包批次三（存嗣/勇决）

D 段（opencode）第三个批次：两个 XIANDING（限定包）技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI。

- 「存嗣」（`dccunsi`，DC 糜夫人）：限定技，出牌阶段选择一名角色，其获得【勇决】；
  目标非自己时拥有者摸两张牌。DSL：`active play_phase once_per_game targets 1 1` +
  `add_skill(event_source, dcyongjue_def)`，用 `if event_source == self` 空分支近似
  “目标非自己”的否定判断（DSL 仅支持 `==`/`<`）。
- 「勇决」（`dcyongjue`，由存嗣授予）：出牌阶段首次使用【杀】后，`choose(0,1)` 二选一：
  令此【杀】不计入次数（`refund_current_sha_use`，怒斩先例），或获得此【杀】的实体牌
  （`gain_event_cards`）。过滤器复用 ABI 407 `owner_source_first_sha_this_play_phase`
  （Claude 已落地），本包 ABI 因此提升至 407。

DC 糜夫人（`dc_mifuren`）的「闺秀」已在批次一移植，但闺秀第二段（发动存嗣后回复）依赖
技能使用事件（`useSkillAfter`），DSL 暂无对应事件，武将闭环暂缓。旧 JavaScript 不进入
npack 或运行时。Golden 见 `original_xianding_batch3.rs`（4 例，全过）；
`cargo run -p conformance` 全量通过。
