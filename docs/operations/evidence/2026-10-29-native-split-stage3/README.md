# F7 第 3 步：内核依赖闭包降至零（2026-10-29）

基准：九批已重建并接入 main，HEAD `c7a55d98`；本批在此之上整合。规则 v328、存档 v105、原生宿主 ABI 753 保持不变。实施报告、审查报告和命令输出属于本次实际隔离源码，不代表第 4 步已拆 crate 或新 WASM 已验收。

## 实现与范围

- 内核闭包对 content/engine 的源依赖 240→0，`BASELINE=0`，新增 vocabulary 纳入扫描；[88 路径及前后 hash](changed-files.json)、[原补丁](f7-stage3.patch)。主线程 `git apply --check` 后应用，未改 index，保留 B9、桌面夹具及文档门禁修复。
- 纯技能名、公共宿主键、HostBehavior 定义及全部 inherent impl、TriggerMeta 静态描述、Gender/不可变武将表、牌种伴随技能描述下沉 vocabulary。十个完整移动文件逐字一致；具体技能机器、filter、activation、factory、step 留在 content，旧公开路径以重导出保留。
- 神弓借牌通过 `Catalog::hooks.borrow_options/borrowed_owner` 回调。标准与扩展目录仍装内容 hooks，原始 `Catalog::build` 保留默认武将表及 `ContentHooks::NONE` 的空借牌结果。
- `apply_general_assembly` 在 state 层保留单实现，kernel、engine、content 共用。PA01 既有用例增加默认目录与扩展后 Boss 索引断言。
- 13 个 Python 文件含 11 个扫描消费者的路径/集合修正，以及分层扫描器和测试的解析增强；[六类实际扫描结果守恒](scanner-source-conservation.json)，没有抬高阈值或删除检查。

## 红绿回归、审查和整合

- [最初分层红灯](layering-red.log)：冻结九批底稿 240 != 0；[隔离树归零](layering-final.log)。
- 独立审查判定可整合，发现 P2：raw identifier 导入会漏检。主线程补七行直导入、分组、raw 别名及正文路径回归，先得到 [红灯](raw-identifier-red.log)（仅计 2/8），再在分词、别名和正文路径三处统一去掉 `r#` 前缀，取得 [绿灯 4/0](raw-identifier-green.log)。没有引入 parser 依赖。
- 主树[分层门禁](layering-main.log)已为 0/0。Python 用例的 loader 增加非空断言，主动 Python LSP 检查无诊断；不能把该结果当 Rust LSP 或完整验收。
- 搬移后遗留的缺失路径/11 处越界引用按源码片段及符号校正，保留历史语境：[逐引用修改](doc-reference-adjustments.json)、[补丁](doc-reference-adjustments.patch)。旧文档中卡牌伴随技能“定义”的定位本来已有偏移，另按真实函数指向 `vocabulary/card_skills.rs:8`，不依靠行数合法冒充符号正确。
- 主树整合检查命令、实际结果及源码指纹见 [integration-checks.json](integration-checks.json)；主树原生全量 **1644/0/73**（[原始输出](integration-native-tests.log)），完整 preflight **21/21** 通过（[输出](integration-preflight.log)）。第一次文档用例因本文引用了已失效的旧行号而失败（[保留输出](integration-doc-reference-unit-first.log)），改为描述旧定位并保留有效新符号后，由完整 preflight 再跑全部脚本用例通过。首次及中间失败不被后续结果覆盖。

## 实际测试与边界

- 隔离源码 engine-native 全量 [1644 passed / 0 failed / 73 ignored](native-tests-final.log)；依赖 wasm/loader/game-server [465/0/0](dependent-tests-final.log)，[Gate 1b exit 0](gate1b-final.log)。神弓三个既有回归、golden replay 均通过，golden 期望文件未改。测试名多重集合 1717→1717（[前](test-list-before.log)、[后](test-list-after.log)）。
- 没有声明 content/engine 的[内核闭包入口编译](kernel-closure-final.log)通过，但保留 dead_code warning；不能写成独立内核 `-D warnings` 通过，不能代替真正拆 crate。
- [标准轨迹对照](cross-compare.log)与[huanle 轨迹对照](cross-package-compare.log)支持所测 native 与旧 WASM 一致；旧 WASM 没有因此变成本轮新构建。实际三 crate、跨 crate 可见性、workspace 与新 WASM、冷/增量性能仍由 F7 第 4 步验收。
- 隔离树原文档门禁失败见 [doc-line-refs.log](doc-line-refs.log)，当时 preflight 不计全绿。主树后续修正后的完整检查单独登记，不能反向修改隔离运行结果。
- [完整实施报告](implementation-report.txt)与[只读独立审查](independent-review.txt)保留原文。当前主稿与台账只按实际主树结果回填，不把整合和编译当产品、平台或上游规则等价验收。
