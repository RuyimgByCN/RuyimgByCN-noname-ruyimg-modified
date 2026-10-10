# ADR 0491：ABI 596 XIANDING 新杀田丰

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。新杀田丰（`dc_tianfeng`，3051387474）
通过 SP 依赖复用「死谏」（`sijian`，3832046646），并以新 DSL 补全「随势」
（`dcsuishi`，2514713423）：其他角色因伤害进入濒死时，仅当伤害来源与田丰势力相同才摸一张；
其他同势力角色死亡时，田丰强制选择并弃置一至当前全部手牌，空手时不产生该死亡触发。

ABI 596 新增通用 `choose_discard_hand_range(player, min, max)` 以及精确死亡过滤器
`owner_alive_event_player_same_group_hand_nonempty`。选择屏障不可取消，恢复时重新验证角色存活、
实体手牌所有权、数量范围和重复选择；复用既有实体弃牌移动及其嵌套触发管线，不新增技能专属
持久状态。

验收：田丰 Golden 3/3、XIANDING 37 个专项测试目标共 131 项、content-compiler、engine-api、
Rule VM、conformance、同步器双检与 26 包空 store 冷导入全部通过；内容锁为
`abd39dcef345bc66c436e6afe71e59d49bcc2876198e05cdf3d4acb28d79304d`。XIANDING 包为 ABI 596、
15 名武将、82 项技能、86 个模块，logic hash 为
`20a812dab1efad95b27f5dae0a1d79a16f8ae76d2fd9e0b0b90590d191b3c7c7`；源码与
`extensions/xianding` 四份 npack SHA-256 均为
`6198a993afad9a588e243556606103454f1942000f063f624ece11a4b3f7cd13`。同步目录当前为 26 包、
2484 名武将、5881 个顶层技能，其中 526 名武将、1519 个技能已移植。

编译产物按架构计划 §37.10 在全部并行编译消费者释放后强制清理：工作区 `target/` 删除 939 个
文件（Cargo 报告 210.5 MiB），共享 `CARGO_TARGET_DIR` 删除 32,768 个文件（15.2 GiB），两目录
均确认不存在。清理范围仅为 Rust/Cargo 编译缓存；上述 npack 与 `extensions/` 发布物明确保留，
清理后四份包的 SHA-256 仍完全一致。
