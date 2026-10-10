# 0497 KEY 包全量技能移植与武将闭包

日期：2026-08-03  
状态：已落地  
领取者：A（Claude，接管 Hermes 遗留包）

## 决策

旧 JavaScript 只作为语义合同，不进入内容包或运行时。接管 Hermes 批次一之后的
KEY 剩余技能：上游 259 个技能全部写出 `.skill` 定义（250 个文件，含组合占位
文件），84 名武将登记入册。

分级：约 75 个技能有 DSL 内容（modifier/view_as/trigger 近似，如枢机、
绯驱、神速系列等），约 175 个为注释仅存空定义——依赖 expansion 扩展区、
chooseButton 动态卡名、花色点数比较、双人选择、装备栏废除、转换技/使命技、
跨回合历史、观星、衍生卡等未建模原语（缺口登记见 ABI-COORDINATION.md E 段）。

## 变更

- `content/original/ported/key/`：新增约 208 个 `.skill`；`characters.json`
  由 10 名武将扩展至 84 名；`key.npack.json` 重新生成。
- 清理 3 处重复 def_id 占位（haruka_kanata/chihaya_liewu2/
  hinata_qiulve_clear 与组合文件重复定义）。
- `extensions/key/key.npack.json`、`package.npack.json` 同步为同字节。

## 验收

- KEY 包 261 项技能（259 上游 + 2 辅助）、84 名武将、37 个模块；logic_hash
  `6fbba3f07ae4ce9191b8d305b128bf45964cad56d52e2adc95bd90087475996d`。
- 26 包冷导入全绿；conformance ALL PASS；key 快照测试
  （original_key_riki_nvzhuang）恢复 12 passed。
- 编译产物按架构计划 §37.10 在批次结束后清理；`.npack` 与 `extensions/` 保留。
