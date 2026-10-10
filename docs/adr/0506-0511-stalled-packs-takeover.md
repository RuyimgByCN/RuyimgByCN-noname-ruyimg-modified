# 0506-0511 停滞包接管：offline/refresh/mobile/tw/jsrg/sp2/huicui/collab/sixiang/diy/sp 全量移植

日期：2026-08-03  
状态：已落地  
领取者：A（Claude，按用户目标接管无活跃认领者的停滞包）

## 决策

全包盘点确认 11 个停滞包（≥8 小时无文件写入、无整包完成声明）无活跃认领者，
按「旧 JS 语义参考 → DSL 重写 → npack → extensions」流程全部接管。活跃包
（xianding/shenhua/bingshi/sxrm/sb/yingbian，其他 agent 近期写入中）不冲突。

## 交付

| 包 | 技能覆盖 | 武将 | npack |
|---|---|---|---|
| offline | 749/749 | 339 | ✅ |
| refresh | 312/312 | 113 | ✅ |
| mobile | 428/428 | 177 | ✅ |
| tw | 350/351（tianxiang_daxiaoqiao 跨包辅助） | 146 | ✅ |
| jsrg | 181/181 | 76 | ✅ |
| sp2 | 281/281 | 102 | ✅ |
| huicui | 249/249 | 114 | ✅ |
| collab | 136/136 | 77 | ✅ |
| sixiang | 174/174 | 100 | ✅ |
| diy | 143/144（noname_changeGroup 系统技能） | 54 | ✅ |
| sp | 639/639 | 207 | ✅ |

合计新增约 3100 个 .skill 定义。按既有分级：能精确的精确，多数依赖
expansion/双人选择/转换技/衍生卡等未建模原语的技能以「核心收益保留 + 近似」
或空定义+注释表达（缺口登记见 ABI-COORDINATION.md E 段）。

## 变更

- 11 个包的 `content/original/ported/<pkg>/` 补全 .skill 与 characters.json；
  `extensions/<pkg>/` 的 `<pkg>.npack.json` 与 `package.npack.json` 同步。
- 批量修复子代理产物的语法问题（方括号修饰符、双过滤器 trigger、缺参 effect、
  非法 view_as/modifier 源、locked+optional 冲突、重复 def_id 等）约 400 处。
- 补齐 collab 子代理未完成的 30 个技能（ol 系列/fuchen 系等）。

## 验收

- 26 包冷导入 `cold_import_all_26` 全绿；conformance ALL PASS。
- 全包覆盖 5374/5881（91.4%）；剩余缺口为活跃包（xianding/shenhua/yingbian/
  bingshi，其他 agent 处理中）与内容结构技能。
- 编译产物按架构计划 §37.10 在批次结束后清理；`.npack` 与 `extensions/` 保留。
