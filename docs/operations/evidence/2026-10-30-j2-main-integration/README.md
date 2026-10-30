# B元数据与J2规则329主树整合（2026-10-30）

**状态：隔离候选复审、主树组合修复、全部运行验收及父最终完整preflight均通过，按裁决18准备本地入库。** 接入基准`44982f4`，其上应用B纯31路径及J2纯72路径（去重101路径），另纠正5路径，共106个唯一接入/纠正路径；当前源码为规则329／存档105／原生宿主ABI758／WS5，桌面仍默认WASM。

## 隔离依据与接入

J2最终独立复审确认原四项P1、一项P2及后发现的胆守拿牌误赠牌P1全部关闭，见[最终审查原文](independent-final-review.raw.txt)、[隔离交接原文](isolated-handoff.raw.txt)。隔离最终workspace6757/0/74、原生三包1670/0/73、严格检查、conformance及preflight21/21，实际WASM36场景／189检查点／114重试及真实Worker三条拿牌/装备/赠牌语义流程通过；这些计数**仅属于隔离源**。父不能拿它们代替含NC10、PA09、Referee及B的新主树结果。

原始72路径补丁SHA256为`96f8ec3be3a3a37903a1623956bbae0c3b5bbe4386b0eee2a09fafd80070e8f9`；父对69路径正常应用，另外3处以共同基准`2b3348ef`做逐块合并：

- 静态prompt表保留Referee五键、B的现存fanjian_suit及字符串编码勘误，再加入J2七键，各键仅一次；未修改编码器或吞掉未知键拒绝。
- Catalog装入J2的vm_active/vm_validate/vm_commit/vm_ask，同时保留Referee九参数HostAsk及Option<u32>花色标量，NONE默认闭包也为九参数。
- misc登记保留Referee两组和J2四组，不替换已有模块。

父另更新J2原生矩阵的choice输出，加入B的kind/native_kind，与实际WASM继续做完整对象deep-equality；没有删掉新增字段比较，也没有重录golden掩盖规则差异。原主树Worker武将定义检查比旧J2基准多五行，同样保留（原主树82项、隔离J2为81项）。具体合并差异、逐文件指纹及九个保留/适配文件见[合并补丁](parent-three-seam-merge.patch)、[合并说明](parent-three-seam-merge.json)、[101路径准入](parent-combined-source-admission.json)。源行数门禁已通过，8980文件在上限内，见[接入后门禁](parent-source-lines-after-merge.log)。

B31路径在应用J2前与复审候选hash逐一相等；接入J2后只改变重叠Cargo.lock及prompt表。B的详情与兼容证据见[B整合说明](../2026-10-30-observer-metadata/README.md)。后续另修复了真实NC10/J2组合P1及五窗trace的B字段消费者，具体如下；初始101路径准入仅描述这两项追加前的源。

## NC10/J2组合反例及修复

[集成初审](independent-integration-block.raw.txt)阻断J2新效果窗进入AI搜索的私有缓存泄漏。真实作者DSL先查询对手红牌数量，再执行胆守及向自己造成reds+1伤害。seed99107868、自己4血，在rvm_danshou_discard保留桃52/闪44；两份状态只有对手四张暗牌全黑/全红不同，完整PolicyInput/choice/events相等，真实待提交伤害为1/5。旧AI预算48时分别选Card52和Card44，nodes3/3、faults0且源archive不变，注册回归实际exit101；见[原始注册red](combined-acceptance/parent-registered-nc10-j2-red.log)及[完整反事实记录](combined-acceptance/nc10-j2-paired-plan-result.json)。不能以114个helper状态可恢复代替规划不泄漏。

父在共享vm_choice_leaf一次补齐七个J2静态prompt，根选择和lookahead同用此公开边界；Rvm的committing、非空commit_state/following连同旧paused/commit_pick均走保守采样，不泛化u32牌号。注册真实DSL回归后预算0/48完整Plan均选Card52、nodes/faults0；保守helper保持players/piles/resolution且字节恢复后可应用，源archive不变。见[注册green](combined-acceptance/parent-registered-nc10-j2-green.log)、[五路径修复指纹](combined-acceptance/parent-nc10-j2-fix-source-check.json)及[修复复审](independent-integration-correction.raw.txt)。复审关闭P1，源码结论OK with notes；不代表后续全量运行完成。

实际native/生成wasm32的同一共享harness直接调用Engine::plan_action：上述反事实每目标4次规划，另对七个真实J2窗口每目标14次规划，预算0/48、policy1、depth2，完整输出一致、nodes/faults0、原archive不变、恢复应用有效，见[实际WASM报告](combined-acceptance/phase-B-direct-planner/actual-wasm-report.json)。没有用NativeSession.auto_step的首选项捷径替代规划证明。范围仅此缓存案例和七类公开选择叶；保守状态仍留私有信息，完整NC10的no-choice/Host/标量来源等缺口保持。

五窗的维护trace_frame也补B的kind/native_kind：native kind复用既有函数，wire kind由测试按五种枚举独立匹配，无新增依赖；原全部choice/hash/revision/visible/deckCount字段和Node完整对象比较保留。见[测试消费者适配](parent-five-window-choice-adaptation.json)。新trace下原生18测及实际WASM75状态比较/270座位检查/35命令/40字节重启通过，对应日志在combined-acceptance/phase-B-machine-*。AI修复不升规则/存档/ABI/WS，仍为329/105/758/5。

## 版本与包边界

329和758由裁决17仅授予J2；存档格式105和WS5保持。Rvm的following、commit_state、committing按既有尾部机制入档/入摘要。旧规则328及更早明确拒绝，真实原包/旧档/冻结锁保留；不制作虚构迁移。B的可选字段已通过旧/新WS5读者兼容，随已批准超时批再整体发布WS6。

三类宿主原语为756冯河交牌/重铸、757胆守四分支与序号弃牌、758平寇；均使用真实作者DSL与选择/恢复/子事件时序验证。冯河真实交牌仍发CardsGiven；胆守从手/装备获取走take_owned_card、失牌和普通CardsMoved，装备撤销附带技能，不触发被拿牌者的赠牌收益。完整shiji仍有phase_prepare/game_started等未支持分支，onlyOL整包、R03全部边界和N01 AI等不因此完成；真实旧shiji0.2.4和0.1.0原件保留，新包0.2.5，不伪造未发现的中间版本。

超时330/WS6、玩家选序331和改命取消332均有各自明确授权，仍须等上一批真实验收接入后依次实施。本批没有提前改这些版本、协议或规则；PA09切换条件、真机/CI、37来源与整体目标均保持未完成。

## 验证、磁盘和产物

修复后的phase B：原生三包1706/0/75、六个影响包6035/0/77、workspace6829/0/77；workspace全targets及native三包-Dwarnings通过，分层0、8981文件行数门禁通过。当前生产WASM/Worker82/0、前端153/0/typecheck/build、conformance、正式native/WASM轨迹及旧线crosshash通过。B重新生成14档/98跨入口对照、Chrome/WKWebView各16条实际组件流程；J2实际WASM36场景/189检查点/114重试、作者SDK/theft及Chrome Worker三例通过。普通投影5状态/40对照/32隐私通过；忽略项为native75、desktop1、旧core1，逐项记录保留，不能当已验收。完整命令/产物与历史阶段见[验证交接原文](combined-main-final-handoff.raw.txt)、[最终运行摘要](combined-acceptance/final-acceptance-summary.json)、[63条命令退出记录](combined-acceptance/command-results.json)。

首Chrome runner因4186端口占用在启动浏览器前失败，只改独立端口后复跑三例通过，没有杀别人的进程。完整preflight320.84s实际19过/2链接失败，原因是父将standard.rs:162当相对URL；修成#L162后两项补跑均过，core7/all1900零坏链。失败原文保留；父修正后独立完整重跑21/21、exit0，用时317.65s，两项行数门禁亦通过；运行前后20752维护文件指纹一致，见[父最终门禁](combined-acceptance/parent-final-gates.json)及[完整重跑日志](combined-acceptance/parent-final-preflight.log)。原失败不改写。所有已完成命令与失败尝试分别保留，不以后补通过改写原exit。

最终runtime源码审计：维护20752文件聚合SHA256为`e671b4d6418c9f0c12606320405990d02fcb57c65b546f952015845b20cbab3e`，phase B前后相同；101接入路径与5纠正路径全部精确相等，无非Markdown漂移。37个tracked dist在保留当前实际产物与SHA后恢复HEAD字节。见[源码审计](combined-acceptance/final-source-audit.json)及[产物SHA](combined-acceptance/artifact-hashes.json)。

验证worker原run触及45分钟硬限时；完成命令均有独立exit，未结束项不计通过，[限时记录](combined-acceptance/parent-run-45minute-timeout.json)保留。父确认无活跃相关编译/队列后，从同一保留会话恢复，不切换CLI、不改源码或重录期望。修复前20条命令归phase A，后续新源码归phase B；完整workspace/严格门禁等按新源码复验，旧绿不冒充新绿。

编译使用CARGO_INCREMENTAL=0/jobs2及全局串行锁；stage3私有缓存仅为当前连续接入验收复用。结束的B缓存约12.2GiB、J2两个缓存约7.4GiB已在复制真实二进制/WASM并验SHA、确认lsof无使用者后退休；保留源码、补丁、日志、冻结包和实际产物，不删共享或活跃target。后续实际验收和最终产物指纹随验证交接登记，功能耗时不作性能收益结论。
