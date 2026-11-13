# 队列第 1 项：13 来源复核补课（2026-11-13）

开工源码 HEAD `bd58ef06d59940baed8d7f93e61ce571b152fb81`，已跟踪工作区干净；规则333／NGSA107／content schema5／ABI759／WS8。工作区保护清单见 [初始快照](initial-state.json)。本目录为执行证据，当前状态仍写入进度主稿，清单状态仍以 `docs/migration/g0/inventory_status.json` 为准。

## 2026-11-13T23:23:00+08:00 执行记录

- 第0项代码 `1d59f42b8` 已发布并取得10作业成功CI；其后纯文档 `bd58ef06d` 的同一条CI run43449112325仍在运行，没有重启。
- 本批尚未提交／发布，未上调任何等价或整包签署。清单暂沿用849／3575／0，[37来源计数](../2026-11-13-queue0-gates/source-counts.md)；整包八项0/37。
- U01全64技能及31武将重新独立查源码；另12来源以完整765项为分母、固定种子2026111301抽131项，另补10个交互样本。配置／界面等224项逐条单独审查。原带日期证据和表格保留。
- 浏览器复用 `apps/web-client/scripts/tabletop-comparison-cdp.mjs` 的本地静态服务器、独立Chrome profile、headless CDP启动方式。琉璃版禁止自动启用其他扩展；新版源码在临时副本运行Vite及原有FS服务。禁止的Spine运行库未读、未复制、未加载。
- 琉璃版 `_shengke` 已经真实 `useCard` → `useCardToBegin` → 强制 `chooseToDiscard`，通过上游UI处理器选择【杀】／【桃】分别保留另一张。原生既有L06定向回归2通过／0失败／0忽略／79被过滤，[实际命令](l06-native-existing.json)与[完整输出](l06-native-existing.log)。这些结果尚不构成13来源补课验收。
- 新版启动首试缺根tsconfig／Vue文件、FS服务未启动；补齐原始文件并启动原有服务后已进入8人局。所有浏览器请求、返回与启动失败保留在 `browser/session-*.jsonl` 与 `modern-*.log`，没有把失败改写为成功。

上游与原生同场景的最终表、逐条差异、清单变动及独立审查结果将在本目录新增记录。尚未执行本批完整产品门禁，也未开始队列第2项。


## 2026-11-14T00:47:27+08:00 本批复审与代码门禁记录（尚未提交／发布）

源码HEAD `bd58ef06d59940baed8d7f93e61ce571b152fb81` + tracked dirty；规则333／NGSA107／schema5／ABI759／WS8。队列0已完成，上一份文档HEAD的CI也已成功，见[前批CI观察](publication/prior-documentation-ci-observation.json)。本批仅为队列1的部分增量，未开始队列2。

- 聚贤top分支补强制获得牌单选，隐藏手牌使用规则RNG私有映射、公开装备按实体选择；映射入既有HostMachine.cards，恢复不重洗。四项真实发动回归通过，仍有J01–J05差异。首轮未洗牌实现被独立复核退回，原记录保留；[最终补审](independent/u01/j06-followup-review.md)仅对本补丁源码/断言通过，不签整技能等价。
- U01逐项重审64技能与31将静态字段；其余12来源固定种子抽131项并补10交互项，最终68确认差异／47待核／26静态未见差异；配置/UI等224项另核。初次应用及01–05更正保留，11条过度确认已撤回。清单撤回215个旧等价，新增等价签署0，保留旧状态不作为本轮认证。[复审后37来源634／3790／0](source-counts-reviewed.md)、[U01应用复审](independent/u01/application-review.md)、[其余来源复审](independent/other-sources/application-review.md)、[非规则应用复审](nonrule-audit/application-review-final.md)。
- 上游实际5技能／13分支／24次与原生指定投影相同，27次本轮选择挂起恢复相同；完整事件、牌堆、RNG和全部条件/组合未比。其余10来源30候选及L22自动规则未运行验收，见[对照表与入口边界](runtime-comparison.md)。
- RNG补丁后的完整三crate为2446通过／0失败／6忽略，完整工作区7842／0／8，严格独立target all-targets、wasm32、conformance、跨目标、desktop20／0／1、前端489／0／0与typecheck、build:native、smoke:native83／0、native/WASM与生产build均exit0；完整实际命令、环境和输出为本目录01–15的`*-final.json/.log`。没有用先前过滤的定向运行替代全量门禁。14场标准golden与一份旧NGSA107不变，见[范围证据](golden-and-old-archive-final.json)。
- 首次完整preflight：20项通过、全量链接1项失败（636坏链），见[实际命令](20-preflight-first.json)和[首次输出](20-preflight-first.log)；未把首次红门禁记为成功。修正链接与最终完整复跑结果另增记录；通过前不提交。

[已知自动/固定选择差异](known-automatic-choice-gaps.json)仅覆盖重审范围，非37来源完整普查；当前未证明必须升号的技能0项，不代表全部可表达。[后续队列及外部条件](remaining-scope.md)保留界面对照、Spine逐特性/净室/用户验证、八项0/37、真机与签名、第六阶段。冻结输入与保护初检见[核验](final-frozen-and-protection.json)；本批浏览器/副本/strict target及64,674,161字节私有生产build副本已退休，证据与实际native-pkg保留，见[清理](owned-cache-and-browser-cleanup.json)及[最后build副本回收](final-private-build-cleanup.json)。共享target、原有未跟踪文件和8个旧工作树未清理。


## 2026-11-14T00:58:36+08:00 完整本地门禁收口（未提交／发布）

源码HEAD `bd58ef06d59940baed8d7f93e61ce571b152fb81` + tracked dirty；版本333／107／5／759／WS8。第二次完整preflight21项通过、exit0；本批所有第五节命令均exit0，精确命令/环境/计数/首次失败见[最终门禁表](final-local-gates.md)与[机器记录](final-local-gates.json)。636处href修正及不改审查语义的证据见[链接更正](link-correction-20261114-01/correction.md)。仅按[明确提交范围](commit-scope-plan.json)提交本批，未发布时不写成已完成；队列1仍部分，发布及exact-code CI另增日期记录。
