# I02 phase1：类型与协议入场候选（2026-10-31）

[实现报告](implementation.raw.txt)、[29文件清单](manifest.json)和[完整补丁](full.patch)已封存，父重新核29源SHA。**此处保留首轮私有候选复审BLOCK两P1与两P2，后续[入口修复已限定复审通过](../phase1-ingress-repair/README.md)，未合Main或宣完整I02验收**；[原复审](source-review-v1.raw.txt)指出组合坏IPC提前清pending、网络保留项无限增长、Rust／TS的FEFF策略差异和C→S错误variant导致空洞负例。Main现有NC10 Own六文件接入是另一批，不能据此算I02已集成。

裁决30批准范围内实现engine-api leaf DTO、可缺省presentation响应和严格Rust／TS入场。所有九个Rust生产构造器均None并省略序列化；没有真实公开事件producer、Engine提交range、资源loader binding、WASM新export或播放行为。本阶段不能证明身份披露权限、真实提交或冻结资源membership。

旧5fc Rust／TS正读者已复制并核原SHA，执行旧读者接受新可缺省字段、新读者接受旧缺省，C→S保持严格；共享七事件正例及47变异在Rust wire／TS入口运行。Worker／IPC／SAB经EngineClient校验前不删除pending，network在store／持久收据／ACK前校验；bad-present只通知匹配调用结果未确认，保留原幂等收据。共享请求入口复用固定8上限，第九个在发送前拒绝，避免保留不确定项导致无限增长；这项超额并发行为变化明确记录，不能写旧行为全同。

已报API／协议11通过、server／desktop受影响调用方check通过；完整前端330通过发生在最后conn_seq0 guard之前，最后受影响入口190通过／typecheck。**不能将330记为最终全量**。原编译／fixture／诊断／artifact生成错误均留日志，见[原验证索引](evidence/validation-index.json)。匹配的旧Main WASM SHA225f…仅为既有回归输入，没有从新DTO源重建WASM。

项目3GiB下限指可用磁盘；原worker另外尝试macOS RLIMIT及owned进程RSS监测，是本轮额外执行限制，不是项目验收要求。失败与监测记录保留，不作为产品性能或必要策略依据。未改其他worker进程、编译profile或Main。

后续独立ingress-repair阶段的30文件／最终Unicode证据及限定接纳另见[修复目录](../phase1-ingress-repair/README.md)；原29源、patch、日志和首轮BLOCK结论不改。完整producer／qualified factory／提交生命周期／真实隐私与各端产品仍依C29／R05顺序推进；全37、18其他资产、234动态／2487缺backing及作者生命周期仍保留。批准及当前状态见[上级合同](../README.md)和[进度主稿](../../../../architecture/native-progress.md)。
