# NC10：隐藏牌抽样与公开VM选择叶的局部修正（2026-10-30）

**状态：隔离实现及两轮独立审查完成；父在`5ec62ae`之后接入纯12路径，当前主树功能验收通过。NC10整体未完成。** 隔离基准为`38789f54c798829d7b32bff1c990def50cf7f611`，主树基准为`5ec62ae49a7b0b731a2ff53f1de9318ef81e861c`加本批；规则328／存档105／原生ABI753／WS5未变。前置8路径与复审候选指纹完全一致；下表隔离结果与后述主树结果分别登记。

## 代码范围

敌方VM私有牌堆进入未知物理牌抽样池；同一映射更新已审计的牌引用及物理归属，保持本方和公开知识。按类型遍历替代旧决斗限定路径；Host通用槽可能是牌号、花色或其他标量，不作盲目u32重映射。

真实VM query→STORE→HOST_CHOICE反例显示：公开PolicyInput相同，旧规划仍因隐藏存储出现Tag0/nodes3与Tag1/nodes6差异。当前根节点及每层lookahead对公开`rvm_choice`／`rvm_discard_pick`选择窗，仅依投影options/min/max/cancel/Pass生成合法答案，nodes0；策略不按私有storage是否存在选择。保留已有模拟与回退入口，不重新引入直接查询权威隐藏状态的兜底。

初审发现两个本增量引入的问题：VM非零CardId46被抽样为native0后，VM Store/Move把它当零哨兵；为其他观察座位抽样时，当前响应者的Shan35变成FireSha97而响应选项仍旧，恢复报OpenChoiceMismatch。修正对VM commit/paused机器及其OptionalGate等嵌套包装、属于其他座位的当前选择采取保守路径，保持其手牌、私有牌堆、选择和续行payload；仅保留已有确定性牌堆顺序打乱。**这是有效性保护，保守状态仍保留私有信息，不是完整信息边界验收。**

真实Liaoyuan恢复反例保持注册；借用独立Referee批已审查的8条恢复前置路径后，测试穿过OptionalGate查找真实Host，验证花色标量、候选、初次恢复和后续恢复。8条前置路径不在本批纯增量中，见[前置清单](borrowed-prerequisites.json)。主树对应改动由Referee批单独整合。

## 实测与复审

以下为隔离候选的原始结果；独立审查读取日志和源码，没有重新执行测试。

| 检查 | 结果与证据 |
|---|---|
| 原两项P2的真实回归 | 0通过／3失败，含两条VM Store/Move与一条Shan响应，见[red](review-fix/p2-first-red.log) |
| 修正后NC10定向 | 53通过／0失败／4忽略，见[完整日志](review-fix/p2-final-green-nc10.log) |
| 最终原生三包全量 | 1658通过／0失败／75忽略，exit0，见[全量](review-fix/p2-final-full-native3.log)、[按目标统计](review-fix/p2-final-full-native3-counts.json) |
| 原生三包严格检查 | 全部exit0，RUSTFLAGS=-D warnings，见[命令](review-fix/commands-index.json) |
| WASM门面原生投影/权限/动作 | 11通过／0失败，见[日志](review-fix/p2-wasm-native-privacy.log) |
| 实际生成WASM32 | release构建exit0；Node smoke 11通过／0失败，含235步终局，见[构建](review-fix/p2-wasm-build.log)、[smoke](review-fix/p2-wasm-smoke.log)；已有壳层dead-code警告保留 |
| 源码门禁 | 当时8923文件未超上限、分层上行引用0，见[源码](review-fix/p2-source-final.log)、[分层](review-fix/p2-layering.log) |

初次全量1582／76／75的上游夹具路径失败与定向52／1／4的Liaoyuan查找失败均保留；后续独立完整运行通过，不将单项补跑回填为旧全绿。参见[初次全量](review-fix/p2-full-native3.log)、[初次定向](review-fix/p2-green-nc10.log)、[最终命令](review-fix/final-native-commands.json)。更早Liaoyuan真实恢复失败与opaque VM原始red也保留在isolated/。

两轮审查原文：[初审](independent-initial-review.raw.txt)、[修正复审](independent-final-review.raw.txt)。最终结论为“OK with notes for this honest partial NC10 increment”，两项P2关闭，无新发现；不代表完整NC10通过。

## 主树接入验收

当前主树定向 **53/0/4**、原生三包 **1679/0/75**、完整workspace **6796/0/77**，16条执行命令均exit0。严格三包、分层0、源码门禁、conformance、当前源码实际WASM重建、Worker82/0、前端146/0/typecheck及native/WASM正式包轨迹通过。相对PA09/Referee主树增加14通过和2个显式手工质量忽略项，未将忽略记为通过。见[主树交接](main-acceptance-handoff.raw.txt)、[命令索引](main-integration/recovery-command-results.json)、[全量原始日志](main-integration/recovery-workspace.log)、[按目标统计](main-integration/workspace-detailed-results.json)。未为AI独立增量重复不受影响的WKWebView重载/性能测试。

新增真正的opaque planner WASM证明：仅一个query→STORE→HOST_CHOICE的`rvm_choice`案例，实际私有查询得到local0=0或4；同一共享harness在native和实际生成wasm32直接调用`Engine::plan_action`，预算0/48、策略1、depth2。每目标4次规划及4次字节恢复后应用，完整PolicyInput/choice/events/Plan在私有反事实间相等，nodes/sim_faults均0、源archive不变、应用后可恢复；Node深比较两目标完整输出（包括archive/frames/hash/结果）相等。**没有用NativeSession.auto_step的首选项路径冒充规划证明。** 见[harness源码](main-integration/opaque-harness/lib.rs)、[输入拷贝审计](main-integration/opaque-harness/fixture-copy-audit.json)、[实际WASM报告](main-integration/opaque-wasm-report.json)、[native完整输出](main-integration/opaque-native-output.json)及[WASM完整输出](main-integration/opaque-wasm-output.json)。只关闭本案例的实际WASM缺口，author-DSL `rvm_discard_pick`的WASM反事实仍缺。

验证期间生产维护源码hash完全不变，12条纯增量及8前置路径复核一致；父新增临时`.pi-lens.json`防自动编辑被完整记录，提交前由父移除。见[终态审计](main-integration/source-final-audit.json)。完整构建产物和harness留存SHA，私有输出在比较后退休，stage3缓存仅为紧接着的B/J2主树整合暂留，编译后清理理由见[逐次记录](main-integration/post-command-cleanup.jsonl)及[交接](main-integration/target-cleanup-handoff.json)。

## 质量基线与尚未验收项

同一诊断矩阵的debug节点8：288局、269终局／19截断、96569请求样本；release节点64：288局、272终局／16截断、109778样本。只有两个种子及座位轮换/重复，不是288独立种子；不含VM包，延迟仅诊断，不构成跨profile性能收益或完整质量结论。原始汇总见[debug](isolated/node8-debug-summary.json)、[release](isolated/node64-release-summary.json)，运行hash见[记录](isolated/quality-runtime-hashes.json)。

仍缺：非D3Moyan Host槽位语义审计；无选择窗的VM storage/flags/缓存标量来源；既有依赖storage的trap分支；保守状态修正后的隐私矩阵；完整typed续行等价；公开记忆/已知牌策略；牌堆形状/数量信息；J2新续行。实际opaque-VM WASM反事实已补上述一个`rvm_choice`案例，author-DSL提交窗及其他状态仍未补；标准WASM smoke和无VM质量矩阵不能替代。

## 接入边界与产物

[纯12路径补丁](nc10-corrected-complete.patch) SHA256为`de121a0f23019a9226627a46d88f54ac9906673373cf05fbac4796b81303bd05`；[五路径修正补丁](nc10-p2-vs-frozen-11.patch)为`838591a55219384b2bee21d419eb8e6cedaf8c45571fead13a67870fd27ce32f`。父逐一核对候选12个源码hash并验证对当前树git apply --check exit0，见[父核验](parent-candidate-verification.json)、[源码清单](corrected-source-manifest.json)。父在`5ec62ae`后解除冻结并实际apply，12路径结果hash逐一与复审候选一致，借用前置8路径保持不变，见[接入源码核验](parent-integration-source-check.json)。主树16条功能验收命令已通过，父最终完整preflight **21/21**、行数门禁及2项自检通过，见[最终检查](main-integration/parent-final-checks.json)。一次父工具60秒中断只记未完成，后续独立完整preflight耗时320.5秒exit0，原记录保留，不重复全量功能测试。旧PA09/Referee结果仍属于未接NC10的源码，本页隔离表也保持其原候选范围。

按用户“每次编译完要清理”要求，任务结束并确认无打开文件使用后，已回收其约3GiB私有target；源码、补丁、日志、质量结果与两份实际WASM产物仍在本证据目录及原隔离任务目录，详见[回收记录](../2026-10-30-pa09-desktop-native/post-compile-nc10-private-target-cleanup.json)。父后续接入将复用自己的构建目录，不重复恢复这份已结束缓存。
