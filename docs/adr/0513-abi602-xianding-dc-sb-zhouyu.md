# ADR 0513：ABI 602 XIANDING 新杀谋周瑜

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。新杀谋周瑜（`dc_sb_zhouyu`，3870127835）的
「融火」（`dcsbronghuo`，2240860827）在任意火焰伤害结算后获得不可叠加的一次性蓄势；每次使用
火【杀】或【火攻】均令基础伤害增加存活势力数，蓄势则在下一次此类用牌时先消费，并允许追加一名
合法且在范围内的目标。「英谋」（`dcsbyingmou`，4166950825）保留通用转换技状态：阳态摸至所选
原目标的手牌数（至多五张）并通过普通管线使用虚拟【火攻】；阴态选择另一名并列最多手牌角色，
按权威随机顺序连续使用其手牌中所有当前合法的实体【杀】和伤害锦囊。若其起初没有此类牌，则弃至
周瑜手牌数；所选目标未在该精确序列中受伤时，周瑜对其造成 1 点火焰伤害。

ABI 602 新增 `resolve_dc_sb_zhouyu(player, source, card, action)`。实现复用当前牌基础伤害和追加目标、
标准 Choice、通用转换状态、权威 RNG、实体牌完整使用以及 `CardResolved` 原目标/受伤目标记录；
选择恢复时重新校验角色、技能、实体牌和目标合法性，不保存或执行旧 JS 事件树与动态 storage。

验收：谋周瑜 Golden 2/2、XIANDING 43 个专项测试目标共 145 项、content-compiler 499/499、
Rule VM 15/15、engine-api 4/4、conformance 3/3、26 个官方包空 store 冷导入均通过。XIANDING 包为
ABI 602、20 名武将、94 项技能、103 个模块，logic hash 为
`9eaadcc182e649d533915a81e499af4dbb4502c8bd64322927d849b326c03670`；源码与
`extensions/xianding` 四份 npack SHA-256 均为
`d25a4b2115d26b1f9c235f0eae3cd3d90416e7c77473527e4ce0c1aa55a9e148`。

全局 `sync-original-content --verify-only` 仍被既有 `diy:junkjizhi` 两个定义 ID
`3934300054/2783324283` 冲突阻断；该冲突不属于本 ABI 或 XIANDING 包，本批不以覆盖、占位或
白名单跨包修复。收尾按架构计划 §37.10 只清理 Rust/Cargo 编译缓存；`.npack`、`extensions/` 与
README 属于发布物，必须保留。工作区先清理 5,368 个文件（1.7 GiB），共享
`CARGO_TARGET_DIR` 清理 90,371 个文件（40.4 GiB）；随后三轮并行短时编译各重建 2,119 个文件
（各 722.7 MiB），均在对应进程退出后补删，最终两个编译目录均不存在。清理后四份 XIANDING
包哈希保持一致。
