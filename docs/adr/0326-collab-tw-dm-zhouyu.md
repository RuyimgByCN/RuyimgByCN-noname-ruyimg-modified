# ADR 0326：COLLAB TW魔周瑜 选将池闭包

本批为 COLLAB 包 TW魔周瑜（`tw_dm_zhouyu`，2765035469）的选将池闭包：
四个技能均已在既有包中完整移植并部署，无需新原语或新 .skill 文件。

## 技能解析

- `twjuyan`（2964355463，TW 包，已移植）：造成火焰伤害后摸一张牌并令目标减体力上限。
- `twlihuo`（2910350044，TW 包，ABI 389 `resolve_twchengpu` 已落地）：普通【杀】转火、
  加目标及濒死失血代价。
- `yingjian`（3382160794，MOBILE 包，已移植）。
- `yiran`（3082356765，COLLAB 包，已移植）。

跨包引用由运行时按 def_id 解析（先例：tw_zhangfei 引用 refresh `new_repaoxiao`、
old 小乔引用 shenhua `tianxiang`/`hongyan`）。

## 打包与验证

- catalog 中 `tw_dm_zhouyu` 标记为 `ported`；加入 collab `characters.json` 选将池
  （男/群/4 体力，技能 [twjuyan, twlihuo, yingjian, yiran]）。
- collab npack 重建（24 模块，19 武将），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/collab/`；README 补充闭包说明。
- 旧 JavaScript 不进入 npack 或运行时。
