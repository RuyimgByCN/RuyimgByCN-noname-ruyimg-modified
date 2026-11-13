# 队列第 0 项：失效源码引用修复（2026-11-13）

- 核验人：`repair_doc_refs`。基准 HEAD：`3a6e398485db6ef0729abe3013c3c4b7705a5d6d`；开始时已跟踪文件无修改，历史未跟踪文件保留；本记录写入时工作区 dirty。本批仅修改文档引用。
- 版本边界：规则 333／NGSA 107／content schema 5／ABI 759／WS8，本批未修改版本或代码。
- 范围：23 处越界源码引用及对应 Markdown 行号锚点，涉及 7 份文档；另经主 agent 授权修复全量链接检查发现的 3 条既有坏链（2 份文档）。未重写历史结论、测试名或签署状态，未运行文档生成器。

## 核验与结果

| 操作 | 结果 | 证据 |
| --- | --- | --- |
| `python3 -B scripts/check_doc_line_refs.py`（修改前） | 8,764 处引用，48 种路径未解析，23 处越界；exit 0（默认模式不以越界返回失败），只打印前 20 条 | [原始输出](before.log) |
| 引用修改范围自检 | 首次发现一处新引用终点填错，修正为实际完整测试文件末行后通过；反向替换验证 7 份历史文档除这 23 处引用与锚点外逐字相同 | [首次错误](self-check-first-failure.log)、[补跑](self-check.log) |
| `python3 -B scripts/check_doc_line_refs.py`（修改后） | 8,793 处引用，48 种路径未解析，越界 0；exit 0。未解析路径仍包含原有示例/缩写路径，不能据此宣称所有源码引用都有效 | [原始输出](after.log) |
| `python3 -B scripts/check_local_links.py` | 默认 7 份主稿，坏链 0（该默认命令不覆盖本批全部修改文件） | [原始输出](local-links.log) |
| `python3 -B scripts/check_local_links.py --all`（首次） | 2,053 文件，4 条坏链：3 条既有路径、1 条仍在生成的 after.log；exit 1 | [首次全量输出](local-links-all.log) |
| `python3 -B scripts/check_local_links.py --all`（补跑） | 2,054 文件，坏链 0；exit 0。补跑时 after.log 已生成，三个既有路径已修复 | [全量补跑输出](local-links-all-rerun.log) |
| 最终引用范围自检、`git diff --check` | 9 份历史文档仅授权引用/锚点/路径与标签发生变化；23 个旧引用对应 40 个现存范围条目；空白检查 exit 0 | [最终自检](final-self-check.log)、[空白检查](diff-check.log) |

## 引用定位范围

逐项旧引用、新落点、替换次数与符号/结算入口依据见 [reference-mapping.json](reference-mapping.json)。四份 2026-11-12 复核证据分别为 U06 2 处、U07 15 处、L10 1 处、L27 2 处，共 20 处。其余 3 处位于 G0 的 U05、L19、L27 文档；此前扫描因脚本只显示首 20 条，误将全部 23 处归于四份复核证据。主 agent 已明确扩展修复范围，三份 G0 文档也只替换源码引用。

- U06 按当前 `HuanleMonkeyCard`、`HuanleMianjuCard` 完整分支定位。
- U07 按当前 HostBehavior 状态机、独立 kernel 查询/结算路径与实际测试模块定位。`kangyin2` 同时指向授予和距离修饰读取；`fuchou`、`jianbi`、`liangce` 同时指出目录锚和实际内核消费点，避免只将原越界范围截短。
- L10 指向 `Rangjie` 四种挂起询问的完整 match 分支。
- L27 的 `huamu` 指向当前惰性 dispatch 行；Batch A 指向完整现存模块。G0 L27 的大分派改为当前两批 handler、asks 与独立 tw 步骤模块。
- G0 L19 的大分派改为两批 trigger/filter/machine/handler 与 asks/步骤模块；G0 U05 指向当前机关牌与具名扩展牌的完整结算入口。

## 全量链接检查补充修复

首次 `--all` 发现的三条既有坏链，已按主 agent 扩展所有权授权只改路径及其标签/同文档重复路径：Step 1 的 BattleView 视图实际位于 `apps/web-client/src/views/BattleView.vue`；U01 的 gallery 函数实际位于 loader 的 `decision34_pack9.rs`，精勾/默言的过滤实际位于 `filtering/common.rs`。见 [link-mapping.json](link-mapping.json)。不借路径修复更改历史等价结论。

## 保留的历史结论与未核验范围

本次仅核对引用能否指向当前实际功能入口，**不构成旧复核表等价结论的再次认证**。U07 旧表中的 `WudianYoudishenruFollowup` 与 `u07_wangmeizhike_six_dimension_real_match_equivalence` 在当前源码不存在；实际诱敌深入响应/追杀位于杀结算的 step/resume，望梅止渴测试位于独立模块及 `u07_existing_skills_and_cards_six_dimension_complement` 分支。旧表对 fuchou、jianbi、liangce、diyjuntun 等的条件/效果描述，以及 L27 huamu 的 disabled 标记叙述，也不能因修复引用而视为已验证，已向主 agent 报告，交队列第 1 项重审。

未修改清单计数或整包签署，未编译、未运行规则回归、未在上游实际触发。完整产品门禁由主 agent 在整合后执行。
