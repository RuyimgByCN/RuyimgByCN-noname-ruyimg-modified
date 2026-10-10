# 0495 NEWJIANG 包全量技能移植与武将闭包

日期：2026-08-03  
状态：已落地  
领取者：A（Claude，接管 Hermes 遗留包）

## 决策

旧 JavaScript 只作为语义合同，不进入内容包或运行时。接管 Hermes 批次二之后的
NEWJIANG 剩余技能：上游 102 个技能全部写出 `.skill` 定义（103 个 def，含
`dunyong_effect` 辅助技能），43/43 名武将按上游 `character.js` 的性别/势力/体力/
技能映射登记进 `characters.json`（技能引用已移植 def_id）。

分级（与既有包一致）：

- 精确表达：beiyu、biancai、dckanghui、dcyiju、duanwan、dunyong、jingyi、kangli、
  lkzhongzhuang、mpbishi、mpjiusong、spqishe、yizu、duchi（牌堆底摸牌+展示）、
  spyishi（伤害-1+取装备）、dcduliang(+dcduliang2 标记)、zhenbian（标记计数+
  手牌上限基数）、dczhifeng/dchuairen/lunshi/yjshenwei（view_as 转化）等。
- 近似表达：核心收益保留、复杂分支注明（各文件注释）。
- 空定义+注释：依赖 expansion 技能牌堆（铃/刺/材/料）、双人同时选择、拼点连续
  循环、按花色集合状态、衍生卡 createCard2 等未建模原语的技能（gnjinfan、
  duwang、cibei、gongqiao、xianmou、sangu、dczhonge 等），缺原语登记见
  ABI-COORDINATION.md E 段缺口表。

## 变更

- `content/original/ported/newjiang/`：新增约 85 个 `.skill`；`characters.json`
  由 1 名武将扩展至 43 名；`newjiang.npack.json` 重新生成。
- `extensions/newjiang/newjiang.npack.json`、`package.npack.json` 同步为同字节。
- 修复并行会话引入的 `content-compiler` ABI 598 常量 use 列表缺失（机械修复）。

## 验收

- NEWJIANG 包 103 项技能、43 名武将、66 个模块；logic_hash
  `71798f0a8b357e93b5f7ba5ccb0ca1c27e98ff503a439cc2aeaf4ceb971b0139`。
- 26 包冷导入 `content-runtime` 测试全绿（新增 `cold_import_all_26` 测试）。
- key 包 3 处重复 def_id 占位文件（haruka_kanata/chihaya_liewu2/
  hinata_qiulve_clear 与组合文件重复定义）删除，恢复 key 包可编译。
- 编译产物按架构计划 §37.10 在批次结束后清理；`.npack` 与 `extensions/` 保留。
