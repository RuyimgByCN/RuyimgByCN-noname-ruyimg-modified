# 0496 CLAN 包全量技能移植与武将闭包

日期：2026-08-03  
状态：已落地  
领取者：A（Claude，接管 Hermes 遗留包）

## 决策

旧 JavaScript 只作为语义合同，不进入内容包或运行时。接管 Hermes 批次一之后的
CLAN 剩余技能：上游 82 个技能全部写出 `.skill` 定义，37/37 名武将按上游
`character.js` 的性别/势力/体力/技能映射登记进 `characters.json`。

分级与既有包一致：能精确表达的精确（弹道/清励/还阴等 Hermes 已落地项保留），
其余按「核心收益保留 + 复杂分支注明」近似；依赖家族查询、横置指定目标、
expansion 等未建模原语的技能给空定义+注释（缺口登记见 ABI-COORDINATION.md
E 段）。

## 变更

- `content/original/ported/clan/`：新增约 76 个 `.skill`；`characters.json`
  由空扩展至 37 名武将；`clan.npack.json` 重新生成。
- `extensions/clan/clan.npack.json`、`package.npack.json` 同步为同字节。

## 验收

- CLAN 包 82 项技能、37 名武将、20 个模块；logic_hash
  `6597945f15d8da6feb52e50ec6daf3e654cc30bb4a6cd4e45ef2019ccb9323db`。
- 26 包冷导入全绿（`cold_import_all_26` 测试）；conformance ALL PASS。
- 编译产物按架构计划 §37.10 在批次结束后清理；`.npack` 与 `extensions/` 保留。
