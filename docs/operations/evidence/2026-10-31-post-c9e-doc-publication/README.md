# 9f4 文档提交：正常发布与精确HEAD CI登记

核验日期：2026-10-31。本批只发布文档／原始证据，不导入私有C331生产源码。最新文档提交为 `9f4b74236f97b0b9dc72388b6e08d310d2f26f64`，parent `34e83f0550bb70ef4af6083697428de7be2f02f8`，tree `88dfb549ab937d23b3632371eb77f387a566cb7c`。生产代码基线仍为34e、rules330／NGSA105／ABI758／WS6，桌面transport默认WASM；37个代码／配置保护输入与两提交中的字节均相同。

## 入库与发布

父按既有裁决18对[9,905项枚举计划](parent/noname-post-c9e-doc-stage-plan.json.gz)逐blob／字节／mode复核，合计257,498,242B；[实际暂存审计](parent/noname-post-c9e-doc-stage-audit.json)与此前独立临时索引预览得到同一tree。临时索引从未复制回Main；实际入库使用普通git add及commit，见[提交收据](parent/noname-post-c9e-doc-accepted-commit.json)。独立文档复审只接纳叙述和归档范围，见[C9e文档复审](../2026-10-31-c331-phase9e-source-predicates/docs-scope-review.raw.txt)，没有接纳C代码或整个Goal。

正常推送唯一目标为已批仓库的 `refs/heads/native-convergence`。新guard精确绑定仓库URL、目标ref、new9f4及expectedOld34e，八项真实正负控制通过，原 `.git/hooks/pre-push` 字节保持 `e349af1e…`。没有force、no-verify、永久hooksPath／remote设置、备份ref推送或历史清理。

- [发布scope与八项控制](push/original/push-stage/scope.json)、[结果](push/original/push-stage/result.json.gz)、[36份材料的原始复制清单](push/push-copy-manifest.json)。原始push结果可能使用lossless gzip，实际路径见[复制映射](post-release-archive-map.json)。
- [租约释放](push/original/push-stage/lease-release.json)：push exit0，一次普通全历史推送；18:04:45.794689Z明确释放Main只读租约，CI观察不持有租约。
- [父独立发布核验](parent/noname-parent-9f4-publication-push-audit.json)：36项SHA／gzip CRC核对、八控制重新执行、before／immediate／after原始索引／逻辑项／flags／未跟踪清单相同，另以[独立远端读取](parent-live-remote/remote.stdout)确认native-convergence=new9f4、远端main仍为47347c66。旧bdaf错误叶的本地保护引用保留，不是新提交祖先，552本地排除物保留。
- Git显示打包2965对象、**12.26MiB为四舍五入文本**；new-revset2152对象、56,806,619B未压缩是另一口径，不能等同精确wire字节。push含启动／队列的12.6376秒不是纯队列时间，也不是性能收益。

父重放accepted守卫时最初使用父目录cwd，被真实worktree守卫正确拒绝；[原始失败](parent/noname-parent-9f4-publication-push-audit.attempt1.raw.txt)及[说明](parent/noname-parent-9f4-push-control-cwd-note.json)保留。改为实际Main cwd后八控制核验通过，没有在错误调用中执行网络或push，也未绕过守卫。

## 原始索引变化的已知与未知

正式暂存前，守卫观察到原始index从16:36记载的 `fb7832…` 变成 `626e19…`，mtime为17:34:34Z。守卫在任何暂存变更前停止；写入者未知，旧fb原始副本没有保留，**不能声称差异仅为ctime或TREE缓存**。

父保存[626原件](pre-stage-index-observation/observed-index.raw.gz)与[观察记录](parent/noname-parent-doc-index-guard-observed.json)，并通过[逻辑审计](parent/noname-parent-doc-index-logical-audit.json)证明当前全部53,864项恰为HEAD加原2,540项文档，路径／OID／mode／stage及旧文档blob字节完全匹配，所有项为正常stage0、无assume-valid或扩展标志，Git SHA1 footer有效。删除TREE缓存的候选哈希也不等于fb，未以该假设解释原变化。

随后只对明确列举的9,905文档执行授权的正常add／commit，626→cfc是另行登记的有意变更；没有复制、恢复或归一化原索引来掩盖未知。此事实发生在发布租约前，与已证明稳定的短push窗口分开。不能归咎源码worker，也不能用本次逻辑正确倒推未知原始统计字段的全部变化。

## 新CI最终结果与已保留检查点

新运行实际于18:27:46Z结束为 **10/10 success**，130步骤=128成功／2跳过／0失败／0null，见[最终封存summary](ci-final/original/verified-publication-ci-summary.json.gz)、[126项原始manifest](ci-final/copy-manifest.json)和[封存seal](ci-final/artifact-index-seal.json)。126项加manifest／seal合计128主要文件，十日志保留4,848,793原始字节和Windows CRLF，3,628条匹配warning含重复／忽略输出均保留。

[父独立最终登记](ci-final/parent/noname-parent-ci41227614131-audit.json)重新核SHA／CRC／gzip原字节、130步骤、FIFO167个workspace header/result、实际API/protocol12和原生／前端／Own／conformance结果，并用[六项独立API](ci-final/parent-live-API/)再次核run/jobs/artifacts/commit/native-ref/main-ref。workspace6929/0/77、native1830/0/75、三OS lib各853、前端441、Worker82、资源17、conformance19及crosshash按重叠套件记录，不合并唯一总数。两Windows真实checkout／build及desktop installer/upload成功；三安装器仍只有API元数据，未下载或安装。[独立登记复审](ci-final/independent-registration-review.raw.txt)已给出OK with notes／No issues，仅接纳本次文档发布和精确HEAD CI证据；[追加闭环收据](ci-final/registration-review-closure.json)保留其哈希与范围。复审抽读实际日志／API／生成器／错误来源，不冒称全文重算128文件，父完整SHA／CRC／六live核验另列。

### 原检查点（当时未完成）

精确新HEAD [CI41227614131／attempt1](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/41227614131)由push触发。18:15:24Z父用两次独立只读API核对run、attempt、event、branch、HEAD及十作业，见[检查点summary](ci-checkpoint-1815/summary.json)和原始[run](ci-checkpoint-1815/run.stdout)／[jobs](ci-checkpoint-1815/jobs.stdout)：**6success、4in_progress，conclusion=null**。Windows build已成功，Windows desktop仍运行但checkout step2成功；其他checkout均成功。运行中的null不是失败，更不是成功，不把该检查点重写为最终结果。

发布者在启动任何CI观察进程前，私有observer生成器 `ast.parse` 因替换命中内层checkpoint函数而SyntaxError；没有observer文件／进程／watch启动。父仅批准保留原失败并修正自有生成器、完整静态校验及离线结构自检后，再启动原定唯一一次watch，未授权CI rerun／dispatch／换后备CLI。最终封存确认修正后只启动原定一次watch并自然exit0，30次生产者只读CI API均exit0；原SyntaxError、自检、精确tool命令／结果及无后备启动记录保留。[早期发布指针](push/observed-push-and-running-ci-1805.json)的“尚未启动watch”仅属于18:05当时快照。

旧34e的CI41214274772真实10/10与旧d07的8/2失败仍按各自源码保留；它们不作为新9f4 CI结论。本页也不把成功job／installer API元数据当下载安装、实体签名、真机整局、听感或30分钟温度功耗证据。当前仍未全量完成，最终状态见[进度主稿](../../../architecture/native-progress.md)。

## 后续归档范围

[发布租约释放后的复制映射](post-release-archive-map.json)统一记录本目录、C9f和调用方案的原始／归档SHA；gzip可逐字节恢复。它们是本次发布后的工作区证据，**不属于已提交9f4的CI输入**。C9f的225/19私有源码及已知P1/P2见[C9f](../2026-10-31-c331-phase9f-atomic-source/README.md)，真实调用／opaque选择／判定／来源与兼容契约见[下一方案](../2026-10-31-c331-invocation-plan/README.md)。当前没有新增生产源码、包、默认、版本、生成产物或设备权限写入。
