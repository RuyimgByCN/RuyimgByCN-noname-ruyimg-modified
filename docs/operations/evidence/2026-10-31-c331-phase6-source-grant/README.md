# C331 phase6：玄影实际授予与字节恢复（2026-10-31）

私有C候选基于38ab／规则331，本切片未合入Main。两源父group仅授予xuanying_sha／xuanying_use，damage／shan只是注册的子技；原生父技却可触发Damage反击。本批从实际包asset经生产bridge与assign_general_named构造反例：旧实现使攻击者HP4→3，修后不反击、原横置状态保持；实际父技主动杀及取消仍连通。

[实现原交接](implementation.raw.txt)、[独立复审](independent-review.raw.txt)与[七文件补丁](final-freeze6/vs-phase5.patch)保留三个分开的红绿：非法Damage授予、既有xuanying_target漏入prompt字典、漏host_ask导致OpenChoiceMismatch。仅尾追加既有字符串literal，producer与恢复共用精确ChoiceSpec；编码仍UTF-8字符串，没有数字prompt ID迁移。错误options／observer／prompt／kind／min／max／cancel／stage和其它未知prompt负控仍拒绝。

确认／取消均真实encode→decode→Engine::restore，比较完整save（只归一化auth_generation）、hash及完整后续日志，包含RNG／resolution。实际注册g1_shahai组177/0/0，原生三包1749/0/75；两范围重叠。首次错Cargo target与fixture编译错误作为执行错误保留，不算规则红；全部原始失败及最后通过日志均在[复制清单](copy-manifest.json)中。

冻结从135扩为140：五个额外baseline文件先与immutable38ab字节比对，其余保留原冻结来源。父[只读核验](parent-independent-audit.py)／[结果](parent-independent-results.json)fresh核140份tar与工作树SHA、pre/post有序清单、真实argv与17条原生结果汇总；无新运行。manifest81d66d59…、tarb9cb5c97…见[冻结清单](final-freeze6/source-manifest.json)。独立复审No issues／OK with notes，仅接受这个有界修复。

完整loseEnd-use仍未实现，不能用部分CardsLost／CardsMoved／CardsDiscarded拼凑；更广chooseToUse、装备两个元组及其它C机制仍待。源排序证明维持262/290、剩28、新增0，不删除290分母，不把删除非法callback当整技能等价。未新增WASM／GUI／IPC／平台或全十包资格；整C331继续BLOCK，实际331前不得执行改命332或R05正式升号。
