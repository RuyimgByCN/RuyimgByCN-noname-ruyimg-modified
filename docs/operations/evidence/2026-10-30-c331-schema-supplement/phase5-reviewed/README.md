# C331 phase5：17排序修正及44来源绑定（2026-10-30 UTC，2026-10-31登记）

对象仍是私有331、Main保持330。阶段5相对已审final4恰3文件：source_order.rs新增17元组、c331_source_order_phase5.rs三测试、nc08注册；其余132文件相同。135源manifest SHA`cbeab39b1316d77df884242b5494c8b7da2cc5baf8737431ab8a716da6eaaf16`、[完整135源tar](final-freeze5/source-files.tar.gz) SHA`34023da5cc41d2a7971ad5b3c1a8396a49f0ff40d4a8141e7876fdfada107aef`；父[freeze核验](parent-freeze-results.json)核全部135源与46项私有证据bytes／SHA。

[实现原交接](implementation.raw.txt)、[源码／来源复审](source-review.raw.txt)、[行为／冻结复审](behavior-review.raw.txt)均完整保留，两复审No issues／OK with notes，仅接受此增量，整C331仍BLOCK。旧phase4两calculator记录P2经[补充](calculator-corrections.json)正式关闭：NewMeibu／NewZhixi实际get/index.js、SHA8515ed…／621–646行，生成器明确缺路径拒绝且自检；旧错误.ts档案／原生成器不覆盖。

[44项完整绑定](validated-ordering44.json)、[实际API结果](current-bindings44.json)及[注册补充](registration-supplement.json)对应22A＋22B、17修正（14A＋3B）／27保持。修正包括Boss十silent从First→Normal/1/不可选，HuanrenDie Normal/-999/不可选、Yunvyuanshen Normal/610/可选、两Gwent及XueliangFanbei Normal/1/不可选、YuchanSwap／Xiefang Normal/-99/不可选；未变event／optional／extra_events。Core与Liuli计算公式区分、真实译名／subskill回填与group不继承规则保留；未知译名不强转false。

父[retained-only脚本](parent-retained-summary.py)与[摘要](parent-retained/summary.json)核44与剩28不相交、合计72，17／27及每行原bodySHA相等；[逐行44摘要](parent-retained/ordering44-compact.jsonl)和[17修正定义body](parent-retained/corrected17-definition-bodies.raw.txt)便于完整读取。66是caller对照记录、仅6唯一文件：2取父冻结135，4取不可变base38ab Git blob，不冒称66唯一文件或新live-tree扫描；18注册关系及6模块身份中3有retained完整excerpt，其余按原SHA／group literal限定资格。

真实Boss反例：原First在owner1的Normal peers前已写SixiangChain，修后等待owner桶；真实玉蝉反例：原Last令乾仍在手，修后后一个owner选序前已换坤。两测试均在3候选和2候选真实暂停处NGSA编解码／restore，完整save仅归一auth_generation、hash及后续全日志相等，不只镜像tuple。

| 实际运行（passed/failed/ignored） | 结果 | 记录 |
| --- | --- | --- |
| 修前3测试 | 0/3/0，exit101 | [red](source-order-red.log) |
| nc08全组 | 77/0/50，exit0 | [green](nc08-green.log) |
| engine-native全量 | 1739/0/75，exit0 | [native](engine-native-full.log) |
| 本批唯一十包defaultparallel | 2906/0/76，exit0 | [aggregate](affected-default-final.log) |

三范围重叠不相加；原argv无test-threads1、既有六C331fixture锁保持；行数自测2/2、8997文件／Rust有效行及F7零通过。[full-results](full-results.json)与父原103条result行计数匹配。三次原静态parser失败、warnings及最初审查超时原样保留；超时恢复时Git-not-repository诊断属于审查基础设施，不记产品失败。

[三份freeze比对](parent-retained/pre-checkpoint-post-freeze.json)确认pre-full／checkpoint20／final5的135有序path/SHA数组相同，compact JSON18,936B、SHA`dcedfa22a6cff71adeb4a6107b911e462623bc609e37c7359b7226bdfd0e64f6`与原binding一致。没有重跑实验或producer。

独立复审接受限定排序证明 **262/290，剩28**：原218加44实际绑定，不是candidate自动累加；历史234sourceRows含16非current不混算。[剩28](remaining28.json)为12歧义／16译名上下文未闭，继续在范围内；PhaseBegin时机、死亡压窗、救援／到期桥、indexed／forceDie／out／hidden／filterStop等功能另缺。未新跑WASM／IPC／GUI／平台或阈值rebase；final4产品资格仍属其历史源。本增量不并Main、不占新版本；改命332仍待实际331验收。

[复制映射](copy-map.json)绑定29份有限证据／源码tar；没有复制target／serverbinary及完整原证据目录，原manifest保留未复制身份。NC27的独立4dd工作树不属于本冻结或本复审。
