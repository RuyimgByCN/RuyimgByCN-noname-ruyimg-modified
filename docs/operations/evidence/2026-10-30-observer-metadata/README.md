# 批准B：双将与精确选择元数据整合（2026-10-30）

**状态：31路径隔离候选独立复审通过；已在`44982f4`后接入，并与J2共同进行主树验收，尚未提交。** B自身不升规则/存档/ABI/WS；当前合并树因J2为329／105／758／WS5，WASM桌面默认保持。不能把本页隔离统计当作当前合并主树通过。

## 实现与权限

SeatView.generals是主/副顺序的两槽可选元数据，包含def/name/faction/revealed；def0有效，未配置或未授权暗将为null。旧character_def保留。ChoiceRequest的prompt/native_kind可选、缺省兼容，已有ChoiceKind及命令/索引token保持；五种native kind用同一映射，旧kind仍回退。字段贯通原生共享投影、NativeSession/WASM、Worker、Tauri IPC、服务端及实际BattleView/SeatCard。

只从已授权视图取值；国战暗将不泄露给其他玩家/旁观者，自己及受信裁判可见，普通非国战武将保持公开。裁判权限由成员/恢复令牌决定，角色或seat报文字段不能提权；裁判没有legal_actions/submit/answer/replay权限。现有真实WS测试覆盖双将def0及元数据的恢复/重连/旧连接fanout/重启与只读拒绝。

JSON边界拒绝出现但畸形的新字段；旧载荷缺字段继续正常。UI显示两槽并保留旧字段回退，图片使用原有定义号和资源查询，不由名字拼路径；未授权槽不显示图片/皮肤控件。已知prompt采用确认过的语境文案，其余按canonical kind给可读提示，避免显示内部程序键；恶意字符串按普通文本转义。69注册prompt加252源码字面名的282去重项是呈现守卫，不能证明每项可达或恢复完整。

## 兼容及既有恢复修复

真实WS5 Snapshot字节由冻结旧Rust结构读取，旧结构重序列化后由新结构读取；旧TS读者不是空类型声明，而是实际运行旧版转换逻辑，消费98条新WASM投影并与移除新增字段后的新转换结果对照。新读者处理缺字段、真实UI旧载荷回退及畸形值拒绝均有测试。这支持B先保留WS5，再随批准的超时批整体发布WS6；没有擅增WS7或必填字段。

真实反间PickTag的fanjian_suit曾在解码时报BadTag，按父明确授权仅尾追这一既有静态键，并改正“白名单索引是线上tag”的旧注释；编码器仍写字符串、未知键仍拒绝。恢复本身轮换一次auth generation，所以字节不可变检查比较恢复后的状态；直接/恢复续行的完整snapshot和events也比较，不通过假设auth不变来放松安全行为。原始red/初次错误断言和后续修正分别保留。

## 隔离证据与复审

[独立审查原文](independent-review.raw.txt)覆盖31路径及连接调用方，无可执行P1/P2发现。纯补丁SHA256为`8b2a0c80e0ffe0eb4180ac10387cd3fb8c8f0e1dfab9607f74b185594b943ff4`，见[补丁](approved-b-checked.patch)、[候选清单](b-checked-manifest.json)。候选以100489f5加父精确52路径前置为基准，不包含后来的NC10源码。

| 隔离检查 | 结果 |
| --- | --- |
| 原生三包完整重跑 | 1665通过／0失败／73忽略 |
| API/WASM/server/desktop影响包 | 483通过／0失败／1忽略 |
| 元数据/WS5/续行事件定向 | 4通过 |
| 前端 | 153通过／0失败，typecheck/生产build通过 |
| 当前生成WASM/server/IPC | 14实际档／98比较，五kind、十实际prompt、每座位两次恢复 |
| 实际Chrome Worker UI | 16流程通过 |
| 实际WKWebView原生IPC UI | 16流程通过 |
| workspace all-targets及原生三包严格检查 | 通过 |
| 旧core构造/恢复 | 6通过／0失败 |
| 源码与分层 | 当时8954文件在上限内，9分层测试，上行引用0 |

原始命令/环境/退出与失败见[57条命令索引](isolated/b-command-index.json)；[字段矩阵](b-all-field-matrix.json)、[隔离全量](isolated/b-native-three-full-fixtures.log)、[影响包全量](isolated/b-affected-full.log)、[实际跨入口](isolated/b-cross-entry-final.log)、[Chrome](isolated/b-ui-chrome-readable-report.json)、[WKWebView](isolated/b-ui-wkwebview-readable-report.json)均保留。初次822/6/1是缺少隔离树上游路径的失败；父批准只读查找链接后独立完整重跑1665/0/73，不能用后者改写前者。14真实档之外另2条UI兼容/恶意文案流只是兼容测试，不假称引擎生成了任意扩展prompt。

## 父整合与剩余

父逐一核对31个最终源hash并正常apply；J2接入前31/31匹配，见[接入核验](parent-applied-b-source-check.json)。之后仅重叠Cargo.lock及prompt表与J2共同合并；其余B内容仍与复审源一致。主树结果及人工块合并见[J2/B共同整合](../2026-10-30-j2-main-integration/README.md)。25个WASM根绑定和本地胜者规范化保持，未覆盖主树护驾修复。

结束的独占Referee target已在lsof确认空闲、保留桥接/桌面二进制及实际WASM且SHA逐一匹配后回收，见[父回收](parent-completed-target-retirement.json)。隔离源码/原始日志/包/补丁仍在，复制证据时只去掉父创建的重复浏览器profile和非本31路径的完整源码副本，原文件逐一hash确认仍保留，见[复制清单](isolated/parent-copy-manifest.json)。没有删源工作树或共享/活跃target。

本批不关闭NC03/NC09全部目标、所有Referee边界、恢复工厂普查、37来源、PA09性能切换或真机/CI；超时330、玩家选序331、改命332仍按前置实际验收顺序执行。
