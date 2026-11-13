# 2026-11-14T02:31:29.105715+08:00 酒诗恢复修复本地门禁

源码HEAD `605201f3f49e10cea61c31d52a61bbb66c85599d` + tracked dirty；规则333／NGSA107／schema5／ABI759／WS8不变。仅两处恢复期确认定义与既有运行时对齐，未改结算或格式。完整[实际命令与环境](required-code-gates.json)、[最终输入与原始日志核验](final-code-gates.json)及[首次完整preflight](22-preflight-first.log)保留。

- 三crate无过滤：2447通过／0失败／6忽略；workspace无过滤：7843／0／8。
- 独立私有target的5包all-targets、RUSTFLAGS=-D warnings通过；native rustc1.99.1，WASM rustup1.95.0，与既有CI实际stable1.100.0分开。
- 前端489／0／0、typecheck、build:native、Worker冒烟83／0、conformance、native/WASM比较、cross-target、desktop20／0／1及生产构建均exit0。比较器内部3通过／140过滤的范围不代替完整测试。
- 行数单测、行数门禁及首轮完整preflight21项均通过；新增日期文档及U07整合后的文档复核另记，不冒称已经执行。
- 14标准golden及一份旧U01 NGSA107不变；L29独立68份旧档、136次续行及22条直接轨迹见[独立报告](../independent/u06-l29/review.md)。

新测试首次编译失败、修正测试后真正OpenChoiceMismatch红测试、首次绿测试和三次未启动执行器的参数错误均保留，未覆盖或抹去。忽略项、旧线warnings、未跑的UI／真机范围见[验证边界](validation-scope.md)。原672未跟踪路径仍在，8旧工作树块未变，32259冻结产品输入仅两处自有修改，21211门禁输入零漂移；共享target未清。本批尚未提交、发布或取得新代码CI。

2026-11-14T02:33:04+08:00 文件名勘误：[机器记录勘误](final-code-gates-correction-01.json)更正私有构建副本退休记录名为 [production-build-private-copy-retired.json](production-build-private-copy-retired.json)；保留原日期记录，退休事实与数据未变。

## 2026-11-14T02:45:30+08:00 最终日期文档复核通过

同一源码HEAD `605201f3f49e10cea61c31d52a61bbb66c85599d` + tracked dirty，333／107／5／759／WS8。U07清单/日期表整合后的第二次完整命令 `python3 -B scripts/preflight.py` exit0，21检查通过，耗时679.573秒；[实际命令与起止时间](23-preflight-final-docs.json)、[完整输出](23-preflight-final-docs.log)。首轮21通过仍原样保留；两轮均未加quick或过滤。后续仅追加本次已得结果、提交/发布证据，不再改产品输入。尚未提交/发布本批。
