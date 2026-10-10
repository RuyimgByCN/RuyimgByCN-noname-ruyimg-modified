# 0498 YIJIANG 包全量技能移植与武将闭包

日期：2026-08-03  
状态：已落地  
领取者：A（Claude，接管 Hermes 遗留包）

## 决策

旧 JavaScript 只作为语义合同，不进入内容包或运行时。接管 Hermes 批次七之后的
YIJIANG 剩余技能：上游 285 个技能全部写出 `.skill` 定义（约 285 个文件），
71 名武将登记入册（1 名武将 caozhang 依赖跨包技能 new_jiangchi 未登记）。

分级：约一半技能有 DSL 内容（精确/近似，如醇醪/权计/自立用 ABI 565/566
技能牌堆原语、清弦/御策等近似），其余为注释仅存空定义——依赖 expansion、
双人选择、拼点循环、衍生卡等未建模原语（缺口登记见 ABI-COORDINATION.md
E 段）。

## 变更

- `content/original/ported/yijiang/`：新增约 180 个 `.skill`；`characters.json`
  由 27 名武将扩展至 71 名；`yijiang.npack.json` 重新生成。
- 修复 21 处组合文件与独立文件的重复 def_id（子代理并行写入交互失误，
  重建丢失的 21 个辅助技能定义）。
- `extensions/yijiang/yijiang.npack.json`、`package.npack.json` 同步为同字节。

## 验收

- YIJIANG 包 288 项技能（285 上游 + 3 辅助）、71 名武将、200 个模块；
  logic_hash `4cebda3730cf36adf3773930c96bd82701fb8a4f1e155d36370252ad3741668e`。
- 26 包冷导入全绿；conformance ALL PASS。
- 编译产物按架构计划 §37.10 在批次结束后清理；`.npack` 与 `extensions/` 保留。
