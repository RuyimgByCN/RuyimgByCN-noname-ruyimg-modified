# 队列1：U06、L29、U07 实际运行补课与酒诗恢复修复

## 2026-11-14T02:05:03+08:00 执行中，尚未提交／发布

源码HEAD `605201f3f49e10cea61c31d52a61bbb66c85599d` + tracked dirty；规则333／NGSA107／schema5／ABI759／WS8。队列0完成、队列1部分，未开始队列2。清单634／3790／0、新增等价签署0、整包八项0/37保持。

酒诗真实确认窗已复现 `OpenChoiceMismatch`。仅将恢复期两处询问定义与既有运行时 `ChoiceSpec::confirm` 对齐，未改结算、随机、序列化格式或严格校验。新增真实主动／受伤路径测试包含5种确认／取消组合和篡改候选拒绝。首次测试编译失败、真正红测试、首次绿测试及L29定向5通过均保留于 [restore-fix](restore-fix/)。完整三crate和独立target严格all-targets检查已通过；完整workspace与其余第五节命令仍在执行，尚未宣称本批门禁全绿。

U06已逐场重载上游页面实际运行3技能13场景；13初态指定投影相同。最终角色牌区／技能／弃牌与是否结算同时相同11场景；猴子取消存在桃消耗差异，写轮眼第二步取消存在拒绝并继续挂起差异。15次原生回答前存档恢复及续行hash/log比较相同，含3次刻意非法空回答。它们是运行探针，不是新增回归测试或整技能等价。[机器对照与边界](U06/comparison-fresh-nodes.json)，[原始浏览器记录](U06/browser/case-results.jsonl)仅 `fresh-nodes` 的13条用于对照；`first` 的13条受旧装备残留污染，完整保留且排除。

L29与U07各由独立文件所有者收集实际上游／原生证据；旧存档复跑与独立复核仍需逐项核验后另增记录。界面、Spine逐特性／净室／用户验证、设备签名、第六阶段保持待。保留旧日期记录、原始失败、原有未跟踪文件和8个旧工作树；不清共享target。

## 2026-11-14T02:14:51+08:00 独立复核与本次提交边界

仍为HEAD `605201f3f49e10cea61c31d52a61bbb66c85599d` + tracked dirty，版本333／107／5／759／WS8不变。清正/酒诗真实行为差异已由独立agent确认，清单2项旧等价撤回，现632／3792／0、新增等价0；[变更前后原文](inventory-runtime-update-01.json)、[37来源快照](source-counts-runtime-01.md)。U06最终投影连同结算11/13相同，L29最终指定投影22/22相同仍有中间时点与伤害来源差异。独立复跑修复前binary为58/68恢复成功、10次OpenChoiceMismatch；当前为68/68成功；68份旧档与136次续行均逐项相同，22直接路径连最终archive字节前后相同。

本批先提交酒诗恢复修复和已冻结U06/L29证据，[提交范围](commit-scope-plan.json)明确排除仍在写的U07/L10及其后续独立报告。完整三crate2447／0／6、严格编译与wasm32通过；workspace及其余门禁尚未全部收口，未预称可提交。此前纯文档HEAD的同一CI run43475076340 attempt1已经10/10success，[原始终态](../2026-11-13-queue1-review/publication/documentation-exact-ci-first/README.md)不能代替本次代码CI。

## 2026-11-14T02:17:53+08:00 载体卡牌条目补充裁定

同一HEAD + tracked dirty、333／107／5／759／WS8。按[独立定稿](independent/u06-l29/review.md)及[卡牌补充裁定](independent/u06-l29/card-entry-adjudication.md)，再撤回U06猴子/写轮眼卡牌2条旧等价；现[630／3794／0](source-counts-runtime-02.md)，本批共撤回4、新增等价0，U06为5／6／0、L29为1／8／0。所有此前带日期记录及变更前原文保留。苦无双CardDiscarded已确认为原生日志问题，但上游完整弃牌事件未采，不把它冒充本次上下游完整事件比较。

## 2026-11-14T02:33:04+08:00 首轮全门禁通过及U07提交范围补充

源码HEAD `605201f3f49e10cea61c31d52a61bbb66c85599d` + tracked dirty；规则333／NGSA107／schema5／ABI759／WS8不变。 三crate2447／0／6、workspace7843／0／8及其它代码命令通过，完整preflight首轮21项通过；[最终门禁记录](restore-fix/final-local-gates.md)。[范围修订](commit-scope-amendment-01.json)纳入冻结U07与独立报告，保留原计划不覆盖；L10/U08及其独立报告仍不在本次范围。现[628／3796／0](source-counts-runtime-03.md)、本批撤回6旧等价、新签0、八项0/37。U07三技能24场的指定投影12同／12异，43原生挂起检查点；复仇清算与实体诱敌旧等价分别按实跑和源码确认撤回。[独立逐场表与边界](independent/u07/review.md)。原672未跟踪路径与8旧工作树保留，门禁输入零漂移，未清共享target。尚未提交／发布本批；最终日期文档复核另记。

## 2026-11-14T02:45:30+08:00 最终本地检查可提交

HEAD `605201f3f49e10cea61c31d52a61bbb66c85599d` + tracked dirty，版本333／107／5／759／WS8。最终完整preflight21项通过，[完整门禁及所有已知边界](restore-fix/final-local-gates.md)。当前628／3796／0、新签0、八项0/37；产品仅两自有路径、3个采集来源及其独立证据冻结。L10/U08/L28及independent/l10不纳入本代码提交。[阶段性stage范围](stage-owned-prefixes-01.json)、[原始ZIP与26份Windows解包文本换行归一](staged-normalization-01.json)均已核验。准备提交，尚未发布/取得本代码CI。

## 2026-11-14T02:47:05+08:00 代码已发布，首次CI进行中

代码 `5c73d1d9158145e33f98f2018013914667d13964` 已正常推送native-convergence，原钩子及持久配置不变；提交后tracked clean，当前仅本条及公共文档/后续证据dirty。规则333／NGSA107／schema5／ABI759／WS8。精确代码[run43485811587](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/43485811587) push事件/attempt1已登记，初始queued、10作业，未重启或补跑，终态另增记录。628／3796／0、新签0、八项0/37；本批发布不代表队列1或整技能等价完成。

## 2026-11-14T02:51:05+08:00 L10独立报告整合（文档尚未提交／发布）

源码HEAD `5c73d1d9158145e33f98f2018013914667d13964` + 文档/证据dirty，333／107／5／759／WS8。L10原318及独立572文件哈希全符，[主agent冻结输入核对](l10-integration-freeze-check.json)。[独立26案/正式续行](independent/l10/README.md)本批退回，新增等价0，原/正式各102和追加4个挂起档恢复通过；8个已观察结算反例与其它取消/退出/源码差异分列。L10数量8／9／0未变，全清单628／3796／0；[差别栏变更前后](inventory-runtime-update-04.json)。本段不改产品字节，不把CI或恢复相同当规则签署。

## 2026-11-14T03:16:45.676673+08:00 后三来源已冻结，代码CI通过，文档批待发布

HEAD `5c73d1d9158145e33f98f2018013914667d13964` + 文档/证据dirty；规则333／NGSA107／schema5／ABI759／WS8。酒诗修复已发布且同一[精确代码CI](publication/code-exact-ci-first/README.md) attempt1为10成功／0失败／0取消，未补跑，两平台性能内部首试exit0。本次仅整合L10/U08/L28及各自独立报告：[L10](independent/l10/README.md)／[U08](independent/u08/README.md)／[L28](independent/l28/README.md)，无产品/测试变化。两辅助补充[独立裁定](independent/u08-auxiliary-adjudication.md)在已冻结U08目录外单列，不篡改旧报告。主agent冻结字节核验见[L10](l10-integration-freeze-check.json)、[U08](u08-integration-freeze-check.json)、[L28](l28-integration-freeze-check.json)。

当前[625／3799／0](source-counts-runtime-04.md)、新增等价0，本纯文档批再撤回3旧等价；[清单前后原文](inventory-runtime-update-05.json)，本目录累计撤回9。源夹具限制、首失败和源码/运行证据层级全部保留；存档续行通过不签规则等价。[原未跟踪路径/工作树/21211产品门禁输入核验](documentation-product-protection-before-01.json)、[外部条件实际探测](external-prerequisites-01.json)。文档全preflight与发布尚待，队列1仍部分；四来源12交互候选、L22自动规则、已知差异及[其它门槛](remaining-scope.md)继续待。

## 2026-11-14T03:29:13.975149+08:00 本批完整文档preflight通过，尚未提交／发布

HEAD `5c73d1d9158145e33f98f2018013914667d13964` + 文档/证据dirty；333／107／5／759／WS8。完整preflight21项通过／0失败，[命令、原始输出及未验范围](documentation-local-checks.md)。当前625／3799／0、新签0、队列1部分、八项0/37；酒诗代码首次精确CI10/10成功。最后明确stage/字节/链接审计、提交及正常发布随后分别记录，不以准备状态冒称已发布。
