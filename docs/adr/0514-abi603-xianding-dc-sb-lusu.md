# ADR 0514：ABI 603 XIANDING 新杀谋鲁肃

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。新杀谋鲁肃（`dc_sb_lusu`，3824772945）的
「明势」（`dcsbmingshi`，4288768048）在摸牌阶段结束后可摸两张，再强制选择三张当前手牌公开，
指定一名其他存活角色由其获得其中一张。「盟谋」（`dcsbmengmou`，2188556454）保留开局可切换的
转换状态；与鲁肃有关的手牌转移发生后，每回合阳、阴分支各限一次，并在发动时切换状态。阳态令
相关角色按普通距离和次数规则使用至多鲁肃体力上限张实体或单牌转化【杀】，每以该精确序列造成
1 点伤害便回复 1 点体力；阴态令其打出至多相同数量的【杀】，停止或无牌可出后失去未打出数量的
体力，响应侧支持实体、单牌转化、无实体和多牌转化管线。

ABI 603 新增 `resolve_dc_sb_lusu(player, source, card, action)`，复用标准 Choice、实体牌移动、公开
展示、通用转换状态、普通【杀】使用/响应、来源技能记录和回合 flag。技能实例只保存当前有界序列
的目标、实体牌、剩余次数及阴阳分支；每次 Choice 恢复均重验角色、技能、实体牌、目标及响应合法
性，回合边界统一清除阳/阴分支使用位。

验收：谋鲁肃 Golden 3/3、content-compiler 500/500、Rule VM 15/15、engine-api 4/4、conformance
8/8、26 个官方扩展空 store 冷导入均通过。XIANDING 包为 ABI 603、21 名武将、96 项技能、108 个
模块、95 个触发，logic hash 为
`0a69834f1fc7b14e3d0753487ba0b47c3f0613045b0aefa98de33621ecf2a20d`；源码与
`extensions/xianding` 四份 npack SHA-256 均为
`cf5499d07ce69bb17a66541123319f48712f88dc251f01a293e344f231f59062`。

XIANDING 扩大门禁在既有 `original_xianding_bailingyun` fixture 的 `bootstrap_match` 阶段因未消费
可选开局 Choice 而停止（2 项，尚未进入柏灵筠断言，且发生于 ABI 603 包重新生成前）；本批不修改
旧 Golden 的启动策略。全局 `sync-original-content --verify-only` 仍被既有 `diy:junkjizhi` 定义
ID `3934300054/2783324283` 冲突阻断，不以覆盖或白名单跨包修复。

收尾按架构计划 §37.10 只清理 Rust/Cargo 编译缓存：工作区 `target/` 删除 7,880 个文件
（Cargo 报告 3.8 GiB），共享 `CARGO_TARGET_DIR` 删除 10,345 个文件（Cargo 报告 5.8 GiB）；两目录
最终均不存在。`.npack`、`extensions/` 与 README 发布物保留。
