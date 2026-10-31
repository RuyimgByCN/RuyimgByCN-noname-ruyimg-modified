# I02公开对局呈现／语音：具体待裁决合同（2026-10-31）

**待用户答复；没有据此实施产品或分配版本。** 本页将[完整v3合同](proposal-v3.raw.txt)与[独立复审](review-v3.raw.txt)整理为一次明确决策。v1三项P1（发生时公开权限与最终视图校验冲突、资源身份桥接不足、公开原始跨度查询缺少提交约束）及一项P2（非终态拒绝丢失暂停前缀），以及v2后续本地批次消费问题的原始失败见本目录各稿；历史评审结论保留。v3复审为No issues／OK with notes，仅设计可申请授权。

当前核验源码为Main `5fc0f77403d90fa4555156bf01722450cbe7bf3b`，规则330／NGSA105／ABI758／WS6，API1／package format1，桌面transport默认WASM。此基线是分析快照；实施必须继承届时已验收版本，不回退C331、改命332或R05。全I02、全部37来源及SDK作者资源能力保持范围。

## 请求批准的具体变化

| 范围 | 批准后实施的具体合同 |
| --- | --- |
| 响应元数据 | 在现有S→C CommandOutcome和本地EngineEnvelope新增一个可缺省的 `presentation`。批含现有revision、公开顺序ordinal、complete/incomplete/overflow、公开能力缺口及七类实际事件：用牌、响应、技能真正激活、实际伤害、防伤、实际回血、死亡。保留现有粗事件码／文案；真实响应日志补齐会增加code3的实际条数，须专门验证。无新C→S消息或Hello字段。 |
| 唯一Rust权威 | 复用并修正现有presentation projector；在真实提交点复制有效view-as／属性、实际数量及当时已公开的武将／技能／provider身份到host LogEvent。隐藏身份不因本人／裁判权限而公开；前端信任既有权威传输的公开声明，只做结构、资源绑定与路径验证，不用最终武将槽反推历史可见性。 |
| 提交与一次消费 | Engine只维护现有日志上的私有易失范围，不新增正式EventId、RNG、规则队列或收据台账。Suspended／开放BudgetExhausted保留前缀，SegmentSuspended拒绝不丢前缀；真正提交才seal，fault／abort丢弃整个未提交段。NativeSession命令只release，Worker或可变桌面composer同步take一次，放顶层presentation；WASM新增无参数take方法，旧结果字符串保持。服务器耐久提交后take一次并克隆给回复／广播。重连、快照、恢复、重试、观察者／来源切换建立静音基线。 |
| 真实资源身份 | 在实际General／Skill／Card定义上附可缺省、不可变的qualified presentation binding，通过现有包／资源系统构造；保留原始来源pack/raw_symbol或SDK包内definition_id，拒绝中文名／运行时索引／来源标签代替绑定。桥接载荷可加definition_id；真实PackageCharacter/Skill/CardEntry在现有表达不足时加可缺省voice_assets，复用既有音频asset-id验证及skin映射。原始catalog／manifest与包资产须按可信pin／原字节／完整锁验证，不增加来源registry或包管理器。 |
| 资源哈希与兼容 | 新资源声明对asset/content hash和冻结锁的影响必须明确；新字段缺省须保持旧正例与逻辑hash语义，不能把新包字节不变当假设。若当前catalog版本生成导致规则／存档／ABI或不兼容schema门禁必须改变，先提交具体影响和号码方案再请示；本批准不分配任何新号码、不自动迁移旧档。 |
| 播放与全范围 | 一个BattleView媒体owner、一个active元素、待播队列最多8；原resolver／backing校验与SDK冻结资产解析复用。手势解锁、mute／音量、断线／卸载／迟到回调／错误恢复须真实运行；命令不因声音失败回滚。一次消费指一次live admission／尝试，不能保证可听。234动态声明、2487缺backing、18其他资源及全部37来源不删除或改称已完成。 |

## 既有授权与必须保留的依赖

C29批准的真实SkillInvocation／成本／provider／隐藏／救援等Core消费者仍按[裁决29](../2026-10-31-c331-runtime-contract/README.md)实施和验收；R05原始包字节／完整锁／受控factory仍按[裁决28及29顺序](../2026-10-31-r05-runtime-contract/README.md)推进。这里复用那些已批准能力，不重新申请，也不把尚未实现能力当现有事实。没有真实成本承诺的技能不能拿Source27询问、SkillTriggered开始或VM Halted当激活。

VM `KillPlayer`提前改alive／记死亡与DeathMachine取消、替代／重复死亡之间的正式语义问题，**不由本呈现合同批准修规则**。实施前仍须有明确源码语义方案和相应授权；未解决时该domain保持明确未完成，不用前端去重掩盖。

不存在新裁决30；在用户选择批准并记录精确审批稿SHA之前，本页及v3均只是请求。R05已分配759／WS7、规则333／NGSA107，C331／改命332顺序保持，不借用或擅自再升号。

## 具体选择与验收门槛

推荐批准上述限定合同，以便在C／R05真实依赖就绪后实施。也可暂缓全部新presentation字段／binding，继续已授权的C和NC10。暂缓不缩小自动语音目标。

批准后仍须通过：冻结旧Rust／TS／IPC正读者与旧／新匹配JS+WASM组合、坏present元数据原子拒绝；公开／隐藏配对及同段变身历史身份；所有真实提交／暂停／恢复／故障／耐久／重复广播；完整资源身份与作者安装启停升级卸载；hash／存档／正式事件／RNG／Plan及恢复轨迹对照；原生全量与独立Gate1b、当前WASM及实际浏览器／桌面／IPC／WS；真实播放生命周期与逐来源完整验收。测试／平台／听感未执行部分如实保留，不因合同获批标完成。
