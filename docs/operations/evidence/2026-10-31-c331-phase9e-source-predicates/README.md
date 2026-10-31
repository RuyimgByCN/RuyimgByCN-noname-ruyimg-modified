# C331 C9e：有限规则反例修复、41项只读对照与独立红审

核验日期：2026-10-31。Main 仍为 `34e83f0550bb70ef4af6083697428de7be2f02f8`、rules330 / NGSA105 / ABI758 / WS6；本目录是私有 C331 分支的历史源码与执行证据，未把该源码导入 Main。私有边界仍为331/106/758/WS6，后续332/106→R05 333/107顺序保持。

独立复审确认五组目标反例的修复正确，但整体结论仍为 **BLOCK**。追加的41项只读对照发现17组实际源码缺陷；复审另指出链式子事件出生目标的存档校验P1。没有重录 golden，没有整包、37来源或完整 Damage/C331 验收。

## 封存与实际执行

- [原始51行实现报告](implementation.raw.txt)、[原始索引](artifact-index.json.gz)及[结果绑定](original/results-bound.json)保持原字节。原始索引于16:20:29Z封存，包含7,333份产物；源码／保护项封存早于该索引，不把两个时间混为一个。
- [14文件增量](original/final-freeze4/source-manifest.json)：`7e22da5716d417150e058c6f94695bffc78a64bf67f02b6a08e98deec384b8b7`；[219文件组合](original/full-source/source-manifest.json)：`578175e22f8bca7c303fcc1d4333ceaa5c680359b2f3e1d7ea52403e1e96b753`。对应原始tar、patch、完整源码和先前失败保存在同目录树。
- [最终命令及219源码绑定](original/command-source-bindings.json)、[原始结果](original/checks-final4/check-results.json)：定向18通过；全量 kernel/content/facade **1821通过 / 1失败 / 75忽略，exit101**，唯一失败为 `replay_scenarios_match_golden`。独立目标 `RUSTFLAGS=-Dwarnings cargo check --offline -p engine-native --all-targets` 通过；source9035、门禁单测2、layering0。
- [定向完整捕获](original/checks-final4/predicate-traces/)含162组JSON／完整snapshot／archive。测试源码覆盖fuel1下每次被测悬停的byte/full restore、完整最终状态、日志、随机流和成本；只使用已登记的auth_generation恢复增量。实际执行范围不包含后来新增的17组反例。
- [父独立核验](parent/noname-parent-c9e-final-audit.json)重新核对全部7,333产物、14/219源码、tar/patch、七个最终命令及162组捕获，另核362组golden配对。父解析器最初沿用旧 `vs-c9c.patch` 文件名，检查在读取新文件前失败；[原始失败](parent/noname-parent-c9e-final-audit.attempt1.raw.txt)和[准备说明](parent/noname-parent-c9e-audit-preparation-note.json)保留。纠正为实际 `vs-c9d.patch` 后通过，不是生产者重跑或产品失败。

已修复并限定复审的分支：Ciqiu Begin1／实际被准入定义移除；Ciqiu/Xixue普通与元素杀body共用准入；Chilongya/Duwen依据真实传播原因的notLink；Zuijiu真实直接CardUsed父上下文／effective kind/use_as；Hutian保持负、零与正signed量。真实chain父进度校验和实际定义别名反例也已加入。传播虚拟杀仍存在F02，不能把“元素杀反例通过”写成完整杀族等价。

## 封存后的41项只读补充

追加指导在原始封存后才被子任务消费。[单独报告](concordance-supplement/report.raw.txt)记录16:24:55Z开始，16:34:30Z完成审计；[独立补充索引](concordance-supplement/artifact-index.json)于16:35:14Z封存。精确请求送达时间未知，未猜测，也未倒写原始报告。该补充没有执行测试、构建、网络、Main／源码／设备变更。

[41项机器对照与17组缺陷](concordance-supplement/concordance.json)、[实际源码摘录](concordance-supplement/source-excerpts.txt)将状态分为10项有限相符、19项具体缺陷、12项尚未证明。41项窗口／角色元组相符，不等于完整准入、效果、成本、提供者生命周期或异步续行相符。原7,333产物、219源码和51行报告仍相同，见[原始保护记录](concordance-supplement/original-preservation.json)、[父补充核验](parent/noname-parent-c9e-concordance-supplement-audit.json)和[父41项来源绑定核验](parent/noname-parent-c9e-concordance-source-bindings-audit.json)。

## 独立复审与后续必修

[完整独立复审](independent-review-BLOCK.raw.txt)确认17组缺陷均有固定上游和封存消费者依据，并新增链式恢复P1。没有把它们作为可接受差异：

| 分组 | 必须修复的实际行为 |
| --- | --- |
| F01／F05／F17 | Jueqing、Xixue、Minwan不应错误排除实际自伤／自来源情形 |
| F02／F04 | 传播虚拟杀须保留真实use_as因果；Shuchui命中旗与准入／body必须一致 |
| F03／F06 | Zhaohuo正确Begin1自伤取消及目标臂条件；Paotai全宽数量扣减，避免256截断 |
| F08／F10／F11 | Yinzei真实he随机弃牌；Zlfanshi持续terra及随机反击；Abi真实火／雷随机流 |
| F14／F15 | Cuidui真实模式敌我与具名Mushengoumang抽牌；Duqu排除另一个实际启用Duqu的来源 |
| F07／F09／F12／F13／F16 | Qinglong、Zhangwu、Yuance、Jiding、Jidian必须有真实选择／取消／成本／判定或整张牌后续行，不能以首牌、首目标、固定选项或提前移除代替 |
| 新增chain出生P1 | 子事件Start的目标不得仍在父remaining中；实际发射器已先移除选中目标。须有typed/byte原子拒绝与真实出生正控，后续合法数量变化仍允许 |

当前同一私有writer依次处理原子资格／数值／随机流和chain出生校验，再处理五组真实调用续行。F13独立复审明确要求等待整张虚拟杀后移除实际被准入定义，且保留分组恢复的真实来源；它属于下一续行切片，不能提前删除技能。全部12项未证明分支及provider grant/remove/renew/expiry、Feiren suit/altered、Huanxia来源、Zhue异步顺序、完整分组与成本也仍必做。

随后仍须实现真正普通ChangeHp与共享DeathFlow。已修订HP合同要求正submitted `damage.notrigger` 在护甲把HP变化归零时仍运行普通ChangeHp；只抑制对应泛生命周期／damage窗口，不能抑制普通HP回调。方案复审不能替代实现和回归，见[ChangeHp v2方案证据](../2026-10-31-c331-change-hp-plan/README.md)。

## Golden与验收界限

[12场处置](original/golden-business-and-outcome-dispositions.json)复用既有精确C8正向生产者，没有重跑它。362配对capture中11场的六个已观察业务面相同，chain场保留先记录damage再执行自身解链的真实顺序差异。187边界无命令返回、139 Accepted的32字节争议哈希单独保留、32 Pending完整载荷和4 Rejected完整原因一致。未做未知字段归一化；这些结果不证明任意内部fuel、全上游规则或错误相等的隐私安全。

[复制映射](archive-map.json)记录7,351份初始归档文件及原始／归档SHA；大文件如使用gzip均可逐字节还原。额外归档器命令与输出为父后续附录，见[parent目录](parent/)。源码／tar／历史原始证据不作为Main生产代码或已交付功能。262/290、剩28及37来源范围没有因本轮计数增加，完整Goal仍未完成；当前权威状态见[进度主稿](../../../architecture/native-progress.md)。
