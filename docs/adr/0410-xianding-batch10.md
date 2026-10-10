# ADR 0410：限定包批次十（敏善/受责）

D 段（opencode）第十个批次：两个 XIANDING（限定包）技能以既有 DSL 原语组合表达，
不新增 Rule VM ABI；包 ABI 保持 451。

- 「敏善」（`dcminshan`，徐信）：`damage` + `owner_is_target_per_damage_point` 过滤器 +
  `choose_target_draw(self, 2, any)`，每受到 1 点伤害后可令一名角色从牌堆获得两张牌。
  旧 JS 的“若育贤存在记录花色则改为获得该花色牌”联动依赖 dcyuxian 的花色存储，
  未建模（无育贤记录时本实现与旧 JS 一致）。
- 「受责」（`dcshouze`，董贵人）：锁定技，`phase_end` +
  `owner_is_target_mark_positive_3884646990` 过滤器（复用 dclingfang 的“绞”标记定义），
  `remove_mark` + `choose_gain_discard_card_by_kind(self, sha)` + `lose_hp(self, 1)`。
  旧 JS 的“随机获得弃牌堆中的一张黑色牌”以“【杀】”近似（DSL 无按颜色取牌原语）。

两个技能均为独立移植（所属武将的另一技能 dcyuxian 育贤 / dclianzhi 连枝需花色存储或开局
私密选角，尚未完成，不提前写入 characters.json）。旧 JavaScript 不进入 npack 或运行时。
Golden 见 `original_xianding_batch10.rs`（3 例，全过）；`cargo run -p conformance` 全量通过。
编译遵循架构计划 §37.10：使用工作区外 `CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target`，
批次结束后执行 `cargo clean --target-dir` 并确认工作区 `target/` 不存在。
