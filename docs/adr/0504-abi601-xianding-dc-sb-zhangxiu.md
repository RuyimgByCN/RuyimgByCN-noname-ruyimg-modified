# ADR 0504：ABI 601 XIANDING 势张绣

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。势张绣（`dc_sb_zhangxiu`，3827261466）的
「附袭」（`dcsbfuxi`，3319521189）在其他并列全场最多手牌角色的出牌阶段开始时，可交给该角色
一张手牌或装备牌并摸两张，或弃置其一张手牌或装备牌，再无距离限制地视为对其使用【杀】。
「豪义」（`dcsbhaoyi`，2398354936）在自己的结束阶段获得本回合进入弃牌堆、仍在弃牌堆且未曾
造成伤害的全部伤害牌；获得后可反复选择任意子集，分配给任意其他存活角色，也可随时停止。

ABI 601 新增 `resolve_dc_sb_zhangxiu(player, target, action)`。实现复用标准 Choice、实体牌移动、
虚拟【杀】完整使用管线和批量赠牌；只在实际加载「豪义」过滤器时记录本回合进入弃牌堆的实体牌
及已造成伤害牌的物理材料，并纳入规范状态哈希。选择屏障恢复时重新校验角色、区域、实体牌所有权
和技能实例，不保存旧 JS 事件树、动态 storage 或 gain tag。

验收：势张绣 Golden 3/3、XIANDING 42 个专项测试目标共 143 项、content-compiler 498/498、
Rule VM 15/15、engine-api 4/4、conformance 3/3、26 个官方包空 store 冷导入均通过。XIANDING 包为
ABI 601、19 名武将、92 项技能、98 个模块，logic hash 为
`fbb10aaaf81ae7968c9b27f2887da7b43a522c9ce2c4f7fbf92104bf7ccb3683`；源码与
`extensions/xianding` 四份 npack SHA-256 均为
`8399401445505f11b2543d0bcf2f9193e6d13edf2138bc106c3e333ed65f8a65`。

全局同步复验期间同时删除了 SB `sbyangwei`、YINGBIAN `qiangshou/ybzhuiji` 的旧空占位定义，
并让对应武将索引引用已重写的稳定技能 ID；三项包级回归与 26 包冷导入通过。同步器已越过重复 ID，
正常生成模式继续发现 CLAN `clanshixi` 仍是行为型旧技能的空 DSL，占位治理要求不允许将其标为
`ported`，因此目录状态刷新留待该真实技能重写批次完成，不以空实现或白名单绕过。

收尾按架构计划 §37.10 只清理 Rust/Cargo 编译产物：工作区清理 151,517 个文件（41.4 GiB），
共享 `CARGO_TARGET_DIR` 清理 199,232 个文件（48.5 GiB）。清理结束后其他迁移会话立即启动新测试
并重建工作区缓存；等待其 Cargo/rustc 释放后又补删 2,119 个文件（721.9 MiB）。最终工作区
`target/` 与共享目录均不存在。`.npack`、`extensions/` 与 README 是发布物，必须保留，清理后
四份 XIANDING 包 SHA-256 仍完全一致。
