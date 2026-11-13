# U08 英雄杀三技能实际运行证据（2026-11-14）

本批仅收集医心 `yixin`、慧眼 `bolehuiyan`、鬼斧 `guifu` 的上游真实交互与原生对照。收口 HEAD 为 `5c73d1d9158145e33f98f2018013914667d13964`；规则333／NGSA107／schema5／ABI759／WS8。开工时为 `605201f3f49e10cea61c31d52a61bbb66c85599d` 加主 agent 的酒诗恢复定义/测试变更。两次完整 tracked 状态保留在 [开工输入](preflight.json) 和 [收口输入](freeze-environment.json)；收口尚有主 agent 的四份公共文档改动。U08相关产品/正式包字节与开工哈希一致，本 agent 未改产品、清单或Git index。

最终上游 cohort 为 [browser-round4](browser-round4/results.json)：31场每场重新加载独立Chrome页面，待onfree/牌堆稳定后重建初态，30个事件结束、1个故意停在强制弃牌窗，runner异常0。每场八座HP/maxHP/技能/实体牌ID与牌面等核验通过，共248座；关键storage、stat和控制关联另见 [初态审计](initial-validation-audit.json)。上游真实导入原技能源码，磁盘字节与原始archive相同，见 [源加载核验](source-load-verification.json)。没有创建不存在的卡牌定义，也没有替代技能content、卡牌结算或过滤规则。观察包装仍调用原方法，并在询问时切换到该座控制以手动回答；脚本与每窗轨迹保留这种控座操作，不把它当完整自然对局。

原生使用独立外部 [probe](native-probe/src/main.rs)，装载正式英雄杀包并调用正式上游规则附加入口，109个host技能、0个VM技能、没有占位CardDef或catalog修改，输入锁为 `9187435935481107005`。详见 [包与锁](native-round2/native-input-metadata.json)、[31场原始结果](native-round2/native-results.json)、[直接命令/回包/日志](native-round2/native-direct-commands.jsonl)、[恢复命令/回包/日志](native-round2/native-restored-commands.jsonl)。仅用debug接口设初态；发动、用杀、闪响应和回答都走真实命令/事件，未用debug_execute_effect。

76个pending点各有NGSA并实际decode/restore，恢复错误0；76个哈希及除会话认证代数外的完整Snapshot相同。31场直接/恢复续行请求、回包、日志及最终完整Debug快照（只归一化auth_generation）相同。138个原始NGSA及哈希在 [存档审计](native-validation-audit.json)。恢复会增加auth_generation，因此21场两条路径最终NGSA原始字节不同，10场没有pending的原始字节相同；没有把这种差别隐藏为字节一致。

[逐场表](cases.md) 与 [机器对照](comparison.json) 给出实际分支和投影范围。指定投影27场相同、4场不同，不能把27解释为技能等价签署。慧眼真实DOM取消残留本回合标记，后续第二张杀与原生的预测次数、摸牌不同；受控目标场的控制关系原生没有对应维度；强制失败弃牌挂起时源杀还在ordering、原生已在discard。医心选两卡后只有“结束回合”可见控件，原生则是独立可取消目标窗。具体证据及未验分支在 [findings.json](findings.json)。本 agent 是本轮场景实现者，未进行独立复核，没有新等价签署，也没有改预期消掉差异。

初始弃牌堆中21个原生泛型装备没有真实上游名称，未伪造映射；其完整清单保留在每场initial/skippedInitialDiscard。所有手牌、装备和保留牌堆均使用真实源实体并按ID/牌面核验。闪电固有雷属性的显式归一化写在对照投影中。最终所有31场原生全实体、上游已映射实体分别守恒，没有延迟新加的无ID牌。未比较双方完整事件栈、随机流、手牌/弃牌顺序和完整自然阶段流转，详细范围见逐场表。

原始首错按轮保留：[原生首编译](native-first.raw.txt)、[原生首错源码](native-first-source.rs)、[浏览器首轮](browser-first.raw.txt)、[第二轮代表正例](browser-second.raw.txt)、[第三轮取消假设失败](browser-third.raw.txt)、[最终轮](browser-fourth.raw.txt)。各轮runner/setup/control快照都在本目录，不以最终补跑覆盖早轮失败。实际命令和工具链入口见 [commands.md](commands.md)、[runtime-inputs.json](runtime-inputs.json)。

给独立复核者的原始入口是 [review-input-manifest.json](review-input-manifest.json) 和 [raw-transcript-index.json](raw-transcript-index.json)，可据此直接读取源、probe、cohort、请求/回包、NGSA及全日志；无需先读本README、findings或比较脚本。原始CDP共1072请求/1072回包，覆盖4/4/31/31场四轮；首轮前的探测错误也保留。源、原生和脚本输入副本在 `source-inputs/`。

独占Chrome92575/CDP50185/control50210/profile已清理，只读共享Vite5173/FS51215仍开启，见 [cleanup.json](cleanup.json)。没有清共享target，没有创建私有Cargo target。精确旧二进制依 [preserved-probe.json](preserved-probe.json) 保留给独立复核，待主 agent允许后再删除。Spine路径由独占CDP屏蔽；记录中Spine网络请求0，未读取、复制或加载官方/上游Spine运行库。本目录完成本轮收集后冻结，完整文件哈希列在 `manifest.json` 与 `manifest.sha256`，后续独立复核写其它目录。
