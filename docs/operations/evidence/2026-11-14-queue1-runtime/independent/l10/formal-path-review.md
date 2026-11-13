# L10 正式登记与常侍消费复跑（2026-11-14）

原始26案使用 bridge_pack，4 host / 0 VM；独立复跑使用现有 attach_formal_package_upstream_rules("upstream-liuli", "epic-cards", prior_packs=[])，不补造定义。正式路径为4 host / 10 VM，catalog234–243确有十名常侍。该入口是正式登记函数的单包探针，尚不等于包安装、清单验证或整个产品入口验收。

原始载入与正式载入的内容锁分别为8553837534554433482和6741593128147419874，绑定ruleset分别为17777209571409208967和15964965165002195887。全局规则/NGSA/schema/ABI/WS仍为333/107/5/759/8；不能跨不同内容锁比较存档字节来推断恢复失败。

| cohort | 源码与原始结果 | 具体结果 |
| --- | --- | --- |
| bridge-only原始与独立重放26案 | [原探针](../../L10/native-probe/src/main.rs)、[独立重放](native-replay01/native-results.json) | A/B只有k_danggu实际技能和常侍flags，不能据此推整个产品缺失 |
| 正式登记原26案 | [保留首版probe](formal-probe-round1-source.rs)、[登记](formal-run01/formal-registration.json)、[目录](formal-run01/formal-catalog.json)、[原始26案](formal-run01/native-results.json)、[命令/指纹](formal-run01-command.json) | 除内容绑定hash外，最终采集差别仅k_A/B实际授予两项常侍技能；其余原反例与待答状态保留 |
| A/B选伙伴后正常推进Draw、Play | [第二版probe](formal-probe/src/main.rs)、[原始followup](formal-consumption02/formal-followup-results.json)、[命令/指纹](formal-consumption02-command.json) | 唯一Answer是伙伴选择，之后两条NextPhase不提供技能/目标选择却自动结算 |

| continuation | 实际授技 | 座1 HP（授技后→Draw→Play） | 额外实际事件 |
| --- | --- | --- | --- |
| k_A_followup | scsniqu + scstaoluan | 4→3→2 | 两次无主动选择火伤，进入Play自动弃无中104；未产生转化使用牌事件 |
| k_B_followup | scsniqu + scschiyan | 4→3→1 | 两次无主动选择火伤；没有PlayCard/杀事件，进Play还自动鸱咽伤1 |

上游 [mobile.js 4669](../../../../../../../archive/upstream/noname-liuli/noname/character/mobile.js#L4669) 的滔乱为phaseUse、usable1，须选转化牌及实体；[4727](../../../../../../../archive/upstream/noname-liuli/noname/character/mobile.js#L4727) 的鸱咽要求用杀指向目标且其he非空；[5183](../../../../../../../archive/upstream/noname-liuli/noname/character/mobile.js#L5183) 的逆取为phaseUse、usable1，须选目标。本次原生自动结算构成具体差别。上游完整党锢尚未越过dui异常，所以这里是“原生实跑对照上游源码”，没有冒称上游该续行已跑。

正式26案的51+51=102个pending档均恢复成功，direct/restored命令与最终Debug除auth_generation外相同；两个额外continuation再查2+2=4个pending档，两端恢复前后的命令、阶段状态与最终Debug也相同。存档恢复成功只证明这些固定答案序列可恢复，未证明上游规则正确。见 [正式重算摘要](formal-summary-recalculation.json)、[逐案结果](formal-case-recalculation.json)、[续行结果](formal-continuation-recalculation.json)。

两次外部probe编译只使用共享target正常构建，没有清理共享target。首版继承原probe未用Suit导入，1条编译warning原样保留；第二版去除此未用导入，0 warning。没有产品源码改动，没有运行产品/工作区测试或preflight。编译/运行原始日志： [首编译](formal-build-first.stderr.log)、[二次编译](formal-build-second.stderr.log)、[正式26案stdout](formal-run01.stdout.log)、[续行stdout](formal-consumption02.stdout.log)。
