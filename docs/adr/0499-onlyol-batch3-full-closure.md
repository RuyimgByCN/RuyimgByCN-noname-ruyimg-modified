# 0499 ONLYOL 包全量技能移植与武将闭包

日期：2026-08-03  
状态：已落地  
领取者：A（Claude，接管 Hermes 遗留包）

## 决策

旧 JavaScript 只作为语义合同，不进入内容包或运行时。接管 Hermes 批次二之后的
ONLYOL 剩余技能：上游 176 个技能全部写出 `.skill` 定义，77/80 名武将登记
入册（junk_zhangjiao/shen_sunquan 依赖 diy/xianding 跨包技能，待其包移植后
补充；ol_sunxiu 依赖的 reyanzhu/rexingxue/xinzhaofu 属 yijiang 包，已随
yijiang 批次补齐技能定义但武将登记仍缺）。

分级：约三分之一有 DSL 内容（精确/近似），其余为注释仅存空定义——依赖
技能继承 inherit、装备技 equipSkill、expansion、swapEquip、通用伤害修改
等未建模原语（缺口登记见 ABI-COORDINATION.md E 段）。

## 变更

- `content/original/ported/onlyOL/`：新增约 160 个 `.skill`；`characters.json`
  由 7 名武将扩展至 77 名；`onlyOL.npack.json` 重新生成。
- 清理 4 处组合文件与独立文件的重复 def_id。
- `extensions/onlyOL/onlyOL.npack.json`、`package.npack.json` 同步为同字节。

## 验收

- ONLYOL 包 180 项技能（176 上游 + 4 辅助）、77 名武将、22 个模块；
  logic_hash `2093e148b647d73a6bf5cb0cafcf643b0454d127c66c81ae006476ead3b7b1f3`。
- 26 包冷导入全绿；conformance ALL PASS。
- 编译产物按架构计划 §37.10 在批次结束后清理；`.npack` 与 `extensions/` 保留。
