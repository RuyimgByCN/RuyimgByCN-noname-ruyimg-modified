# ADR 0331：DI/YIJIANG RE步练师 选将池闭包（安徐复用 resolve_reanxu）

本批为 DIY 包 RE步练师（`old_bulianshi`，3877843188）的选将池闭包：两个技能均
已有权威结算，仅缺 .skill 接线与选将池条目。

## 技能解析

- `anxu`（安徐，2523373229，YIJIANG 包）：出牌阶段限一次，选择两名其他角色转移
  一张牌并按来源区域/手牌数结算后续。旧 JS 仅作语义证据；结算复用既有
  `resolve_reanxu` 复合原语（与 MOBILE 仁恤共用管线，engine-core 已实现、
  continuation 已注册）。
- `zhuiyi`（追忆，3427217715，YIJIANG 包，已移植）：死亡时令一名角色摸三张牌。

## 文档化近似

旧 JS 安徐的目标手牌递增过滤与“手牌相等时摸一/回复”选择未在 `resolve_reanxu`
复合原语中表达；复合原语按来源区域与结算后手牌数结算后续摸牌（注释于 .skill）。

## 打包与验证

- `anxu.skill` 加入 `content/original/ported/yijiang/`（该包 README 使用 `*.skill`
  glob，自动纳入）；catalog 标记 `anxu` 为 `ported`。
- RE步练师（3877843188，女/吴/3 体力，技能 [anxu 2523373229, zhuiyi 3427217715]）
  加入 diy `characters.json`。
- yijiang npack 重建（112 模块，含 anxu）与 diy npack 重建（23 模块），README
  构建命令逐字节复现（`cmp` 通过），部署至 `extensions/yijiang/` 与
  `extensions/diy/`。
- 旧 JavaScript 不进入 npack 或运行时。
