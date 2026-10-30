# C331 phase4：限定修复与默认并行证据（2026-10-30 UTC）

本页只登记私有331增量，不接入Main330、不关闭整C或37来源。父先核134源SHA，manifest `6f8eabd872feb5719e3bb76cfa7f813273b81a2cddefa8924fb4d626d6d3d20e`／tar `9866a12f604b8eba8af3c5da9304892e2abd12544efc1ecac56a365a4fe094c4`；[权威恢复交接](checkpoint.raw.txt)保留60分钟worker超时及随后只读封存。源／工具chain未在本页重编。

[核心复审](review-1.raw.txt)关闭trigger-only模块通过NO_FRAME／Intrinsic逃过实际窗口校验的P1；真实双反例红→绿、合法active路径保留，本增量未加Frame／RvmSnapshot／wire字段。Normal前缀4→5旧指控继续正式撤回，不新增历史ledger。HlssYueyin／HlssYueyin2／Tianxiang2三条排序行为修正有实际源码和两项真实红绿／恢复对照。

核心复审另发现证据P2：原[18项档案](original-host-source-batch1.json)中NewMeibu／NewZhixi计算器误写不存在的get/index.ts且缺SHA，实际为get/index.js。源码公式本身支持当前tuple，不改成源码P1；原件保留，补正尚待下一私有批。18项不能一并宣称全部身份链闭合。

[产品复审](review-2.raw.txt)接受原WAL variant修正、真实timer和旧reader限定证据，No new product finding／OK with notes。三段结果分开：freeze3为2897/0/76，中间2901/2/76，final4为 **2903/0/76**。父[parser](parent-wire-counts.py)和[全部103条汇总结果](parent-wire-counts.json)绑定原十包defaultparallel argv、exit0、六C331fixture全ok；外层锁不将cargo改成serial测试，fixture内部锁仅隔离六个昂贵257模块初始化。原中间两失败和截断wire不抹去。

最终wire 11,841,224B、SHA `6c14fab323749b67a1ac6e37148ba032986fd6f6c192cbea023a5008ce857ca7`，3875行外JSON及内wireJSON全解析；含127条server Error及其他故障用例，不能宣所有请求成功。父[第二parser](parent-manual-oldreader.py)／[完整结果](parent-manual-oldreader.json)核manual accepted／越过旧deadline／final的完整StoredRoom.state逐字段相等，时间严格manual<old≤observed<new。timer保留choice3的TimeoutAction与choices4–7的四Answer、逐条hash和10→15 revision。WAL base是plain NGSA105、revision10且前后未变，不是压缩archive、未测阈值compact/rebase。完整Native语义比较由绑定fixture实际断言支持；父没有另跑Engine或重新解码全部Native字段／BLAKE3。

旧reader的276个当前生产文件与实际38ab Git blob及原audit相同；负控直接取原record字节区间[316877:606507]，两份输入byte-equal。现存reader61469816B／SHA0ee5e0b4…和server76926256B／SHAd08ff4af…均重hash匹配；自然四条旧WAL正控生产scan／restore通过，actual旧server正控恢复一房，负控报unknown variant Skill后继续自有监听。未修改旧生产serde／header。1,187,184,293B旧缓存清理在取消复核前自然完成，恢复阶段没有新清理。

[前36来源审计](review-3.raw.txt)和[后36来源审计](review-4.raw.txt)分别给22条proven-order，合计44个静态排序候选，其中17条明确tuple差异；另16条译名上下文未闭合、12条来源冲突／非trigger桥／生命周期歧义。两份[完整A](laneA-ordering.json)／[完整B](laneB-ordering.json)含body／公式／译名链／caller身份；部分caller只读回退Main330，后续私有实现须重核，不能自动计入proven数。当前仍登记218/290，234sourceRows中16项非current；尚未将44候选加为262。phase5已限定一个私有writer核44、修17差异及证据P2，结果另登记。

整C继续BLOCK：72来源正式证明未收口、PhaseBegin VM-before／Host-after、dieBegin／dieAfter合并、indexed、forceDie／out／hidden／filterStop等实际机制仍缺；不以defaultparallel绿代替规则等价。改命332仍待实际331验收。

[复制映射](copy-map.json)逐字节绑定14份报告／来源／父parser结果。未复制原11.8MB wire、可执行程序和全部产品目录；其绝对原路径／SHA在报告及结果中保留，不能声称本页是自包含全量实验包。
