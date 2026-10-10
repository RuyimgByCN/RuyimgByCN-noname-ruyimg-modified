# ADR 0521：ABI 608 归档扩展“玩点论杀”费诗

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。归档扩展“玩点论杀”的费诗
（`wdb_feishi`，2227382412）以 3 体力、蜀势力发布完整武将。「率言」
（`nsshuaiyan`，2978279193）在其他角色于拥有者回合外回复体力后可发动，由回复角色强制选择：
令拥有者摸一张牌，或令拥有者弃置其一张手牌/装备牌；没有合法牌时弃牌分支无事发生。
「墨守」（`moshou`，4042669588）锁定阻止费诗成为【乐不思蜀】和【兵粮寸断】的目标，不扩大到
其他延时锦囊。

ABI 608 新增 `resolve_nsshuaiyan(player, recovered_player)`、精确触发过滤器
`owner_alive_other_recovered_outside_owner_turn` 和 `lebu` 牌选择器，复用标准 Choice、摸牌、
跨角色选牌弃置及既有目标取消修正管线。包采用 `community/wandianlunsha@0.1.0` 独立命名空间，
携带归档 GPL-3.0 许可与费诗原始立绘，不执行或打包旧 JavaScript。

验收：费诗 Golden 1/1、content-compiler 505/505、Rule VM 15/15、engine-api 4/4、
engine-core lib 55/55、conformance 3/3，以及包含本包的 27 个扩展空 store 冷导入均通过。
包为 ABI 608、1 名武将、2 项技能、1 个模块、1 个触发、2 项修正器和 1 项资源，logic hash 为
`0514366158dd2b2c546c6e7c65f292d7359a7516e71b30912fb938096ad6d94f`；源码目录与
`extensions/wandianlunsha` 四份 npack SHA-256 均为
`0c9079c325a32f59112ab23a7bdd82ff37739f9010290cc28a9c78fb3f1f24ce`。归档、源码目录和
extensions 三份立绘 SHA-256 均为
`b77aaca1c02056340e93b5642e32524014bd98a453ec268dea37c751f646283a`。

收尾按架构计划 §37.10 只清理 Rust/Cargo 编译缓存：工作区 `target/` 删除 4,101 个文件
（Cargo 报告 2.3 GiB），共享 `CARGO_TARGET_DIR` 删除 6,553 个文件（4.4 GiB）；两目录最终均
不存在，`.npack`、`extensions/`、立绘与许可文件保留。归档扩展的其他内容仍按同一流程继续迁移，
本 ADR 不代表 `/archive/upstream/noname` 全量完成。
