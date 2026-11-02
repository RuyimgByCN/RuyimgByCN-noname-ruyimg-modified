# C331 C9g：真实调用与普通判定，最终源码已编译但全量仍失败

核验日期：2026-10-31。Main已发布9f4为纯文档提交，生产仍为34e代码基线／330/105/758/WS6、默认WASM。以下是独立私有331/106源码及原样验证，不是Main代码接入或全37验收。

[原始C9g报告](original/report.raw.txt)保留封存时 **PARTIAL／BLOCK、最终第6版未编译** 的事实；之后[原样验证报告](final6-validation/report.raw.txt)在相同239文件上完成编译和全量测试，结果仍为BLOCK。后续结果不回写旧报告或以前的红测。

## 实际冻结来源

- [46增量manifest](original/final-freeze6/source-manifest.json)：`9557423705b52ebdfa158004bdeee9e2ab54398f3fa13696cf43249cc225ea00`。
- [239组合manifest](original/full-source6/source-manifest.json)：`69eeebab43d3ccea5f0b48e003cecd8a339f2fe1df8cf8e8d75e118a5309087b`。这是C9f225加实际增量的overlay；其余文件使用不可变5fc源码。
- [原1858项封存索引](original/artifact-index.json)、[当轮结果](original/results-bound.json)及[12条命令绑定](original/command-source-bindings.json)保留每次真实测试对应的版本。第2版predicate98/0/1与最终版有八处差异、四个最终文件未覆盖；第3版strict和NativeSession通过，但与最终版仍有WAL、choice、native invocation、machine validation四处差异。
- [父源码／证据审计](parent/noname-parent-c9g-final-audit.json)核全部1858文件、239/46源码／tar／patch／live字节、225+增量组合和旧5fc回退、两补丁反向检查、旧C9e7333及C9f4167未变。早期两次全原生编译失败、server缺Control match、source-cap初次失败均保留，不以便宜门禁9045通过替代最终编译。
- 源码工作时钟从17:54:55.361409Z开始，实际重型命令截止18:44:55.361409Z；源码索引18:54:22封存，早于18:54:55截止。此前父只观察到子任务仍在运行，不据此认定源码封存超时。封存时无本轮active/queued producer，没有signals或后备启动。

## 有实现，但未验收完整语义

现有Frame／queue／park中加入具名InvocationData、有限Before→Begin→Body→End→After及Before取消→Omitted，持久化进度接入archive／hash／restore／A0。两个C7入口保留实际选中定义，direct body取消仍完成End／After；成本、logBegin及after-log以真实子事件排布。没有增加第二套scheduler／registry。

Zhangwu已改为可选变长he成本并等待log／loss，再重新取得来源方exact-n支付候选或造成n伤害；Yuance有两语义选项、取消与四颜色HP分支；Jiding等待CardUsed／闪／伤害子树后移除实际定义，具名Rushou恢复检查实际Invocation／CardUsed因果；Jidian有距离目标／取消、普通判定后log及自然触发，不保留假递归静音。普通Judge复用JudgementProgress、改判队列、held-card和loss续行，迟滞锦囊上下文改为可缺省。以上实现不等于全部取消／死亡／provider／恢复分支已通过。

内部Control追加在原枚举末尾，公开仍为Custom／字符串／索引，native_kind缺省；没有新公开枚举／必需字段或VM上下文解释。公开source_skill仍只限裁决27的普通OptionalGate Own/referee，私有调用来源不扩大公开元数据权限。静态载荷形状及NativeSession单例不替代旧WS6／真实WASM／IPC／WS／UI验收。

## 独立源码复审：六项P1

[复审全文](independent-review-BLOCK.raw.txt)检查46文件增量、连通消费者、全部新增测试和固定上游，结论 **BLOCK**；未自行执行测试／构建，完整性重算引用父审计。

1. **付款／参与者／跨effect续行伪造**：多数Body progress未绑定实际effect及admission参与者，正数count可改为其他值后被用于来源索款／伤害。须绑定真实选择／支付及子事件收据，保留合法provider移除后的续行。
2. **稳定定义ID**：JidingRecover插入旧helper中段，导致现有Jidong等定义后移。须追加至末尾并验证旧ID和旧pending helper恢复，不归一化哈希消差。
3. **GuiCai资格**：普通Judge暴露单独GuiCai在非国战仍可使用黑装备的错误。运行及restore ask须按真实provider、模式、颜色和区域共享资格；GuiDao／Tiandao独立验证。
4. **Jiding虚拟牌**：当前空牌堆会返回NativeVirtualCarrierUnavailable，即使弃牌区仍有唯一实体牌；上游虚拟雷杀没有实体牌堆前置。须修实际虚拟牌表示和CardUsed子树，不能把panic改fault当源行为修复。
5. **F07仍未实现**：Qinglong仍直接随机获得he，无真实可取消隐牌选择、私有槽绑定、装备撤销／loss或成功提交后的跨座位隐私证据。
6. **F14仍未修**：不同已明置魏／蜀的实际反例仍返回None而源为Some(true)，回归被明确ignored。unknown／ye和mutual yexinjia_friend真实事实及writer缺失，不用阵营取反或身份猜测替代。

Omitted、forceDie／includeOut、log精确顺序、成功来源支付／self-source、黑Jidian嵌套、paid-loss死亡／provider移除／TianDu／多改判／耗尽牌堆／伪造进度等控制仍未齐。全部12项provider／分组／expiry／async行也未因结构阶段新增而获得证明。

## 原样最终第6版验证：编译通过，九项全量失败

[新1917项索引](final6-validation/artifact-index.json)、[实际命令](final6-validation/command-results.json)、[结果](final6-validation/result.json)和[逐项诊断／忽略名](final6-validation/diagnostics-and-counts.json)覆盖19:17:09至19:18:39执行的九条命令。每条命令前后完整239文件哈希相同，没有source／test／fixture／catalog／lock编辑。

| 命令范围 | 实际结果 |
| --- | --- |
| kernel/content/facade完整原生 | **1848通过／9失败／76忽略、exit101** |
| 独立target `RUSTFLAGS=-Dwarnings cargo check --offline -p engine-native --all-targets` | 通过 |
| game-server all-targets | 通过；345条warning行含重复／汇总保留 |
| NativeSession controls | 1通过；18条诊断及warning汇总保留 |
| nc_rounds `c331_` | 98通过／1忽略F14／153 filtered |
| 独立原期望golden | 失败，12原争议case |
| source-cap unit／caps／layering | 2通过／9045通过／F7零 |

计数属于不同且重叠的命令，不累加为唯一测试总数。完整失败不因随后focused／strict通过而改为成功。

[九项原始断言](final6-validation/full-native-failures.json)包括：append-only期望230／实际231；Qizuo play未Accepted；Yuance目标HP3／期望2；Zhangwu play未Accepted；Mingxingzhu Jiding IllegalTargets；Jidian HP4／期望3；Leiji应答未Accepted；GuiCai旧判定牌所有权旧断言；golden12不符。须依源行为和新调用序列分类修复，不能只改期望值。

[父独立验证登记](parent/noname-parent-c9g-final6-validation-audit.json)重新核1917材料、九条命令完整前后239绑定、原日志计数及失败名、362组完整Core／events／RNG／views／choices／referee和命令outcome。11已观察轨迹相同；chain0007-after仅events变化，仍是先记录自身伤害再自身解链。12组actual／expected哈希与旧C9f相同，187无命令返回／139Accepted争议哈希32字节单列／32Pending完整载荷／4Rejected原因均保留，未重跑C8生产者、未归一化或重录golden。

现有hook导出202组JSON／完整snapshot／NGSA，其中15份含新Invocation；新C9g控制测试只有内存restore／same_final，没有完整分支dump导出。[捕获能力界限](final6-validation/capture-capability.json)明确登记，未在“只验证、不改测试”阶段加 instrumentation 冒充原证据。

## 追加P2关闭与后续边界

[追加引用更正](original/golden-dispositions-c9f-reference-correction.json)只改12处final-freeze4→真实final-freeze2路径，原e3703abf…不变，真实19／225清单分别f19c…／ab88…；父审计及本次复审接纳该引用P2关闭。下一比较器引用实际final6，原C9f报告／索引／比较器不改。此关闭不等于golden业务或源码P1关闭。

原始31867 Main保护未刷新；原样验证另核Parent18:42带时间戳4406项收据、Own6／I02／CI／source37、18生成模块／37默认dist／四schema及hooks／旧reader。后续本目录与进度更新属于Parent正常文档写入，以新带时间戳逐路径收据界定，未暂存／提交／推送；不当作此前CI输入或私有源码漂移。旧4258动态收据未另存完整原件的限制见[版本保全说明](parent/noname-parent-post9f4-receipt-versioning-note.json)，不以当前副本冒充旧原件。

同一写入者继续C9h，先处理以上源码P1和真实红测，再完成五组续行／取消／成本／普通Judge／隐私／兼容及12未证机制。之后真正ChangeHp（正notrigger／护甲HP0仍普通回调）、共享DeathFlow／provider／LossEnd仍必做。源码262/290、剩28、37来源与完整Goal4/27均未关闭。PA08真机仍未具备可执行条件，见[19:20只读摘要](parent/mobile-readiness-1920-summary.json)，未执行设备设置／配对／安装／签名／听感或30分钟功耗验收。

复制范围见[逐文件归档映射](archive-map.json)；该映射只覆盖当时3792份复制件，后写README／此设备摘要及进度稿由新增工作区收据枚举。
