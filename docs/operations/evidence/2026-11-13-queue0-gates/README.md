# 2026-11-13 队列第 0 项：门禁、已知偏差与补发布

## 2026-11-13T23:03:20+08:00 第0项代码发布后核验

代码 `1d59f42b8b493e1fbac7a0e8b8c781ccef1ea61c` 已提交并非force推至 `native-convergence`，同一SHA的[CI 43442212867](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/43442212867) attempt 1全部10作业success；[完整日志、平台计数和证据](publication/code-exact-ci-first/README.md)。本地门禁、独立复核及首次失败均保留；规则333／NGSA107／schema5／ABI759／WS8不变。清单849／3575／0，上游实际触发0，整包八项0/37；[后续范围](remaining-scope.md)尚待，不进入下一项。记录时HEAD为代码提交加本次文档dirty；后续仅发布本次证据回填。以下未注明其他时刻的开工段属于执行起点，旧记录不覆盖。

- 执行起点：`3a6e398485db6ef0729abe3013c3c4b7705a5d6d`（main，最后代码提交 `afa72d46b`）。起点已跟踪文件干净，历史未跟踪证据及全部临时工作树登记保留；详见 [precheck.json](precheck.json)。本目录为本批新增证据。
- 版本：规则 333／NGSA 107／content schema 5／ABI 759／WS8；不新增 ADR、协议或版本号。
- 状态：执行中，尚未提交、发布或登记本批 CI。队列第 1 项及后续尚未开始。

## 本批范围

1. 补齐 `FzLiegong` 的现有选序登记和守卫条目；保留现有 `(Normal, 1, false)`，不暗改对局调度。该值与上游 `(Normal, 0, true)` 的现存差异由独立复核登记，清单降为「简化近似」，见 [选序边界](independent-review/fz-selection-boundary.md)。
2. 核对 U01 迁移后正式安装路径的 64 个 host 技能／0 个 VM 技能，修复服务端过期的 2＋62 断言。实际映射与来源逐技测试须同时成立，不能仅按本次报错改计数。
3. 修复 dddanzhi 的黑色历史条件（含打出）、非空弃牌堆误触发、目标独占的一次性免疫及实际消费路径；补真实事件正反例、取消、选择分支、挂起 NGSA 往返及修复前真实旧档恢复。触发时机与选序的额外差异留待队列第 1 项，不能因本次定向测试通过声称整技能等价。
4. 修复失效源码引用和拆文件后的真实链接；只替换原有证据的引用，其余历史内容保留。详见 [引用修复](doc-refs/README.md)。
5. 完整执行用户目标第五节门禁，首次失败与补跑各自保存，全部通过后才提交并按裁决 25 补发布。

## 检查记录的读取方式

本目录每项根级检查由同名 `.json` 保存实际 argv、工作目录、环境覆盖、开始／结束时间、源 HEAD、tracked dirty、退出码及耗时；同名 `.log` 保存未压缩的完整合并输出。命令中的定向过滤仅用于反例／回归，不能替代不带过滤的完整门禁。

- `01-server-before`：服务端旧断言定向首跑。
- `02-line-check-tests`：行数脚本自检。
- `10-frontend-tests`：前端首跑 488 通过／1 失败；原因是测试从已拆分的 `resolution.rs` 中读取 `KNOWN_PROMPTS`。
- `10b-frontend-tests`：改为读取现有 `known_prompts*.rs` 注册表后，489 通过／0 失败／0 跳过。
- `11-frontend-typecheck`：通过。

## 保留的验收边界

本批不进行 13 来源的完整复核补课，也不增加界面、Spine 或平台签署。37 来源整包八项签署保持 0/37；真机、签名和第六阶段保持未验收。标准 14 场 golden、旧档兼容、完整门禁和发布结果必须以实际完成后的本批日志为准，不能沿用历史全绿。

## 2026-11-13 21:20 +08:00 补充执行记录（未提交）

- `dddanzhi` 旧代码的修正夹具反例为 0 通过／5 失败；修复后首轮 7 通过／2 失败，第二轮 8 通过／2 失败。首次输出均保留在 [专属证据目录](dddanzhi/)。后两轮暴露了普通五谷挂起恢复的既有遗漏，正在补现有机器的持牌归属、问询派生和恢复分派，不新增快照字段或版本。
- 修复前的真实 `pending.ngsa` 为 2,149 字节；在新代码下已通过精确解码／编码、两包内容锁和恢复快照（仅归一既有授权代次）的逐字节断言。完整结果仍待最终回归与独立复核记录，不能据此宣称所有历史档均已验收。
- 独立复核发现前端首版注册表读取漏掉同一行的后续提示词，已修为读取专用注册表中所有逗号结尾字符串，并断言注册表非空；`10d-frontend-tests` 全量 489 通过／0 失败／0 跳过。此前 10／10b／10c 原文保留，不用它们替代修后结果。
- 独立复核发现 `tricks.rs` 初稿会改变无暗织时虚拟五谷／桃园的子集目标行为；当前退回修复并增加兼容反例，尚未进入完整门禁或发布。

时间勘误（2026-11-13 21:08 +08:00）：上节标题的「21:20」为录入错误，实际追加于 21:08。命令的实际起止时刻以各原始 `.json` 及专属执行日志为准。

后续各队列、上游实际触发样本、界面／Spine／真机／签名与第六阶段的本批边界已单列于[剩余范围](remaining-scope.md)。独立首轮退回的具体反例见[复核记录](independent-review/static-review-round1.md)，修后必须复核，不能用首轮记录当最终通过。

## 2026-11-13T21:38:41+08:00 完整门禁补跑记录（仍未提交／发布）

- 三 crate 首跑 [04 日志](04-native-three-complete.log) 失败于旧 Adv21 诊断：该测试强制要求五谷恢复失败。首次输出及[旧断言原文](04-wugu-diagnostic-before.rs.txt)均保留。生产恢复路径修复后，改为固定种子 10148 的 40 步 NGSA 恢复回归，必须实际达到五谷挂起，比较完整快照及未中断轨迹；仅归一恢复约定的 auth_generation。
- [04b 完整补跑](04b-native-three-complete.log)：2442 通过／0 失败／6 忽略，命令无过滤。随后按独立复核建议，增加每步 Accepted/Pending 断言，防止两个会话同时拒绝而空转；[最终定向](04c-wugu-restoration-regression.log)为 1 通过／0 失败／0 忽略、112 filtered。最终完整门禁另跑 04d，不以中间运行替代。
- 最终输入冻结见 [v3 清单](validated-final-inputs-v3.json)：25 个文件，原 24 输入未再变化；新增的唯一变化是旧 Adv21 测试。标准 golden 测试源码未改，完整结果以最终门禁为准。
- 独立源码复核与 20 项回归已完成，见[最终复核报告](independent-review/final-report.md)；上游浏览器实际触发样本仍为 0。37 来源清单现状见[逐来源快照](source-counts.md)，总计 849／3575／0，没有上调等价项。

## 2026-11-13T21:46:44+08:00 最终源码门禁与工具链边界（产品验收执行中）

- [04d最终无过滤三crate](04d-native-three-complete-final.log)：2442通过／0失败／6忽略；[05c独立目录严格all-targets](05c-strict-all-targets-final.log)：exit0。当前25输入以[v3哈希](validated-final-inputs-v3.json)冻结。独立复核最终Adv21的Accepted/Pending断言、Wugu实际到达和完整快照／连续轨迹比较，见[增量审查](independent-review/adv21-final-review.json)。
- [06首次WASM检查](06-wasm-check.log)为exit101：实际调用Homebrew rustc1.99.1，其sysroot没有WASM core/std。[工具链核对](wasm-toolchain-before.json)显示rustup stable1.95.0已安装wasm32目标；[06b补跑](06b-wasm-check-rustup.log)只覆盖本命令PATH，exit0。没有改持久工具链配置；新增测试和严格all-targets已在1.99.1下验证，WASM实构建版本分别记录。
- 严格编译私有目录已按realpath／前缀／非符号链接校验后清理，见[清理证明](strict-final-private-cache-cleanup.json)，共享target未清。原门禁私有目录也有独立清理记录。
- [黄金与旧档核对](golden-and-old-archive.json)：14场标准golden源码及期望与HEAD逐字节一致，最终完整测试通过；修复前NGSA107挂起档与fixture的2149字节相同，三内容锁断言与续行通过。此证据不涵盖全部旧档、全部扩展轨迹。

## 2026-11-13T21:49:08+08:00 发布准备与未验收范围

- [发布前远端实查](20-remote-before-publication.log)：native-convergence仍为 `99d9091bb28b55e28b620be16d1b7f343682783f`，是当前HEAD的祖先；[待发布历史](unpublished-history-before.json)逐条列出必要可达提交。没有named remote，暂存区为空，原无条件拒推钩子保留，见[发布环境](publication-preparation.json)。
- 按裁决25准备[命令级精确发布守卫](exact-publication-runner.py)：只允许批准URL、精确SHA、目标ref及预期远端旧SHA，自检允许与各拒绝路径；实际提交、push与CI成功后另增日期记录，本节不表示已执行发布。
- 未完成范围与外部条件见[剩余范围](remaining-scope.md)；其中上游实际触发0、13来源复核补课、全源自动结算／确需升号盘点、界面并排图、Spine逐特性／净室／用户验证、真机与签名均不能由本批门禁代替。八项签署0/37，第六阶段未开始。

## 2026-11-13T22:00:31+08:00 产品门禁与preflight首失败

- [完整workspace](07-workspace-complete.log)：**7838通过／0失败／8忽略**，实际命令为 `cargo test --offline --workspace --no-fail-fast`，未加过滤。后续conformance、WASM构建／冒烟、跨端与desktop门禁继续串行运行，结果分别登记。
- [preflight首跑](18-preflight-first.log)：3项失败——全量链接、规则登记表、脚本单测（8失败、1跳过）；其他18项通过。该首跑保留为红，不能以定向补跑替代最终preflight。脚本单测的完整原始输出另保存在 `18b-script-unit-full-detail`。
- [坏链明细](18c-local-links-all-detail.log)为本批剩余范围文档指向历史目录不存在的README，已指向真实并排图；[补跑](18f-local-links-all-rerun.log)2060文件／0坏链，不表示本批新拍图。
- 规则登记表原32行与实际37行相差5个分支：[源码增量](phase-registry-before.diff)为BossChuanyun End、BossTianyun End、DddXiaxing Prepare、KMowang Prepare及End。保留原2026-10-19历史表逐字节不变，将当前表迁至[规则阶段登记表](../../../architecture/native-phase-filter-registry.tsv)，原32行及其状态／依据逐字保留，新增5行均“未核”。[迁移自检](phase-registry-migration.json)和[定向9项测试](18d-phase-registry-tests.log)通过，[当前表检查](18e-phase-registry-check.log)一致。该扫描只覆盖既有显式阶段过滤臂，不是所有HostBehavior或上游规则等价清单。
- 两个脚本与当前／历史表的最终输入见[工具登记冻结](validated-tooling-inputs.json)；本批25个Rust／前端测试／fixture输入未变。其余完整产品门禁可沿用同一冻结输入，最终preflight必须在本次工具修复后无过滤补跑。

执行时序说明（2026-11-13T22:02:43+08:00）：`18b-script-unit-full-detail`在工具修复前启动，运行期间两个阶段登记脚本发生了已记录的修改；它仅用于展开诊断，不能作为冻结输入的最终通过证明。原preflight首跑在改动前完整结束，8个失败的首结论保留；最终完整preflight为工具v2冻结后的 `19-preflight-final`。工具v2仅把注释“夏行”勘误为“侠行”，见[冻结v2](validated-tooling-inputs-v2.json)。

## 2026-11-13T22:08:54+08:00 脚本完整失败展开后的第二次收口

- [18b完整诊断](18b-script-unit-full-detail.log)实际为192项／8失败／1跳过。失败逐项为旧阶段枚举6项，单技能检查器1项、伤害臂枚举1项；首preflight的链接失败属于另一个门禁，不是该8项之一。没有删除失败用例。
- `audit_upstream_skill_flags.once_allowlist`仍在寻找已拆走的 `let limits = match behavior`，实际约束已在 `host_skill_limits` 函数。现限定读取该函数内直接声明 `SkillLimits.once_per_turn=true` 的臂，保留OR行为名；排除false、按技能名委托、注释及邻接函数，并加一条独立解析反例。[5项回归](19a-audit-skill-flag-tests.log)通过。该事实检查器不据缺席判断不限次，命名委托限制仍须另查。
- `test_report_event_arms`旧计数88未包含既有提交 `14b2b43f7` 的酒诗MbcmJiushi Damage分支，当前真实89，三个扫描源码文件均与HEAD相同，见[当前枚举](damage-arm-current-scan.json)。同步既有计数并要求酒诗行存在，[15项回归](19b-event-arm-tests.log)通过；没有改任何原生触发实现或提升等价项。
- 先前名为 `19-preflight-final` 的运行启动后，发现上述两处剩余失败并修改了工具输入；该运行保留为中间诊断，不能作最终冻结证明。最终完整补跑为 **`21-preflight-final-complete`**，在[工具v3七输入](validated-tooling-inputs-v3.json)冻结之后启动；25个产品／fixture输入仍逐字未变。前两个冻结清单与原失败均保留。

## 2026-11-13T22:12:20+08:00 独立增量复核退回后的工具冻结

- 独立复核构造块注释中的假行为、false字段旁的true注释和被注释的同名函数，均能令此前解析器误报；原反例保存在[首探测](independent-review/once-block-comment-initial-probe.json)与[假函数反例](independent-review/once-commented-function-initial-probe.json)。修复复用既有 `rust_effective_lines.strip_comments`，先清理嵌套块／行注释再定位函数，没有新增解析依赖。[新增回归](22a-once-comment-regression.log)为6通过／0失败。
- `21-preflight-final-complete` 启动后工具输入再次变化，故也保留为中间诊断，不作最终冻结证明。最终采用 **22b-preflight-frozen-v4**；[工具v4](validated-tooling-inputs-v4.json)含8个输入（新增冻结既有注释清理器），25个产品／测试／fixture输入仍与[v3清单](validated-final-inputs-v3.json)逐字相同。独立增量复核仍须通过。
- `pnpm build` 改写的5个已跟踪dist路径已备份并恢复到起点，保留生成产物供查验；[恢复记录](generated-dist-restoration.json)。没有恢复产品源码、清理共享target或修改历史未跟踪证据。

## 2026-11-13T22:21:49+08:00 最终本地验收（尚未提交／发布）

全部本地门禁已通过，完整命令、准确计数、工具链及首失败边界见[最终表](final-local-gates.md)和[机器汇总](gate-summary.json)。产品25输入／工具8输入哈希未变；独立暗织、选序／安装／提示词及两轮脚本增量均无剩余阻断。此时源码仍为起点HEAD加本批dirty；提交、远端与exact-head CI结果另行追加，不把本地通过写成已发布。

## 2026-11-13T22:23:35+08:00 代码批首次发布成功，精确SHA的CI观察中

代码提交 `1d59f42b8b493e1fbac7a0e8b8c781ccef1ea61c` 已按裁决25非force推送到批准分支，实际[发布元数据](publication/code-batch-first.json)及[push原文](publication/code-batch-first.log)。10项精确目标守卫自检通过，原无条件钩子／持久config不变、临时守卫已移除。当前HEAD为该提交加后续文档dirty；CI仍待终态，不用历史99d CI代替，不进入下一批。
