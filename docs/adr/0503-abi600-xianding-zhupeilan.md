# ADR 0503：ABI 600 XIANDING 朱佩兰

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。朱佩兰（`zhupeilan`，3830349941）的「辞虑」
（`dccilv`，3468569693）在成为普通锦囊目标后，按尚未移除的选项数摸牌；若手牌超过体力上限，
强制永久移除一个仍可用选项：令该牌对自己无效、防止该牌造成的全部伤害，或在该牌结算后获得
仍在弃牌堆的全部实体材料。「痛悼」（`dctongdao`，3625586472）在自己进入濒死时限一次选择一名
存活角色，将其非内部技能重置为开局技能；若重置他人且其体力值更高，朱佩兰回复至其体力值。

ABI 600 新增 `resolve_zhupeilan(player, target, card, helper_def, action)`。实现复用标准 Choice、当前牌
目标排除、伤害阻止、实体牌移动、技能增删及回复管线。包内不可获得的 `dccilv_effect` 只承载精确
牌实例的待结算策略，因此即使「辞虑」本体被移除或被「痛悼」重置，已登记的防伤/收牌仍会完成；
结算结束后辅助技能自动删除，不把旧 JS 事件树带入运行时。

验收：朱佩兰 Golden 2/2、XIANDING 41 个专项测试目标共 140 项、content-compiler 497/497、
Rule VM 15/15、engine-api 4/4、conformance 3/3、26 个官方包空 store 冷导入均通过。XIANDING 包为
ABI 600、18 名武将、90 项技能、96 个模块，logic hash 为
`d721339d4df3805f963812653e1e7e339028f4b71e86ec7667f7ea3f8fbda8fb`；源码与 `extensions/xianding`
四份 npack SHA-256 均为 `ca30181cc3ebc0df7b355f84e6ab8dd6ae2728519993855a2450f4a646217a07`。
全局同步器复验仍被共享工作区 SB 包 `sbyangwei` 的重复 definition ID
`4242421112/3601751055` 阻断，不从本批绕过校验。

收尾只清理 Rust/Cargo 编译产物：本批执行 `cargo clean`，工作区删除 83,648 个文件（34.2 GiB）
并补删并发重建的 2,119 个文件（721.3 MiB），共享 `CARGO_TARGET_DIR` 删除 346,160 个文件
（60.4 GiB）。清理期间新启动的其他迁移编译不得被中断，其新增缓存由对应批次按架构计划 §37.10
收尾；`.npack`、`extensions/` 及其 README 是发布物，必须保留，不属于清理对象。清理后四份包
SHA-256 保持一致。
