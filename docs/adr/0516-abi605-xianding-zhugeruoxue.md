# ADR 0516：ABI 605 XIANDING 诸葛若雪

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。诸葛若雪（`zhugeruoxue`，3591468655）以
3 体力、魏势力发布完整武将。「琼英」（`dcqiongying`，2641115727）在出牌阶段限一次：移动一张
有合法落点的装备牌或延时锦囊，然后弃置一张与该牌同花色的手牌；没有可弃同花色手牌时公开
展示全部手牌，取消移动不消耗次数。「暖惠」（`dcnuanhui`，2308165295）在结束阶段选择一名角色，
其可依次视为使用至多 X 张合法基本牌（X 为选择时的装备区牌数且至少为一）；同次结算使用过
同名基本牌时，最后弃置其全部可弃置装备牌，三种属性【杀】按同一牌名统计。

ABI 605 新增 `resolve_zhugeruoxue(player, target, card, action)`，复用通用场上牌移动、实体花色、
公开展示、标准 Choice、共享处理区虚拟牌载体、普通基本牌用牌管线和装备区弃置。续体只携带剩余
次数、已用基本牌名位和重名位，不新增长期状态或专用牌系统。

验收：诸葛若雪 Golden 2/2、content-compiler 502/502、Rule VM 15/15、engine-api 4/4、
conformance 8/8、26 个官方扩展空 store 冷导入均通过，ContentLock 为
`6cc2fcc6cb44a1ef7e438d000982218b726243e90c2ec59c8b48aa79bdfb97d2`。XIANDING 包为 ABI 605、
23 名武将、100 项技能、14 项主动技、112 个模块、98 个触发，logic hash 为
`5f52a65deece9d16c0144e79076389b486811625e0a7ad02f10ecc46198835ae`；源码与
`extensions/xianding` 四份 npack SHA-256 均为
`a2214360b72c0ad5b1a8d4abb8d87aa4db0c55ea041e95519465577236589677`。

全局 `sync-original-content --verify-only` 仍被既有 `diy:junkjizhi` 定义 ID
`3934300054/2783324283` 冲突阻断，不以覆盖或白名单跨包修复。

收尾按架构计划 §37.10 只清理 Rust/Cargo 编译缓存：工作区 `target/` 原本不存在；共享
`CARGO_TARGET_DIR` 删除 6,455 个文件（Cargo 报告 4.4 GiB）。两目录最终均不存在；`.npack`、
`extensions/` 与 README 发布物保留。
