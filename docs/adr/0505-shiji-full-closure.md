# 0505 SHIJI 包全量技能移植与武将闭包（收尾）

日期：2026-08-03  
状态：已落地  
领取者：A（Claude，A 段自己负责的包收尾）

## 决策

全包盘点发现 SHIJI（A 段负责包）此前仅 79/164 技能落地，缺 85 个技能
（binglun/buqi/chuhai/debao/dinghan/dingzhou/fubi/fengjie/huaibi/zuici 等
及 mj 系瞑瞑、sp 系、mb 系辅助技能）。本批次按「旧 JS 语义参考 → DSL 重写
→ npack → extensions」补齐全部缺失技能，并登记全部 48 名武将。

分级：能精确的精确（如 splirang 弃置所有手牌、spyinju2 跳过阶段、fubi2/
refubi 手牌上限加成、huaibi 势力加成、duoji 弃牌取装备等），多数依赖
expansion（仁库/备/仁牌）、关联角色追踪、双人选择等未建模原语的技能以
「核心收益保留 + 近似」或空定义+注释表达（缺口登记见 ABI-COORDINATION.md
E 段）。

## 变更

- `content/original/ported/shiji/`：新增 85 个 `.skill`；`characters.json`
  由 30 名武将扩展至 48 名；`shiji.npack.json` 重新生成。
- `extensions/shiji/shiji.npack.json`、`package.npack.json` 同步为同字节。

## 验收

- SHIJI 包 178 项技能（164 上游 + 14 辅助）、48 名武将、167 个模块；
  logic_hash `2931c30b97525b53a628a3e694d1976b2bb788572e1a0b826166630c32d722ad`。
- 26 包冷导入全绿（`cold_import_all_26` 测试）；conformance ALL PASS。
- 编译产物按架构计划 §37.10 在批次结束后清理；`.npack` 与 `extensions/` 保留。
