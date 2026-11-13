# L28 独立运行证据复核（2026-11-14）

核查 HEAD `5c73d1d9158145e33f98f2018013914667d13964`，规则333／NGSA107／schema5／ABI759／WS8。工作区有主 agent 文档改动；本 agent 只写本目录，未改产品、测试、清单、公共文档或 Git index/refs，未提交。本批不签任何新“已复核等价”。

建议撤回 `th_yeyan` 旧等价。真实原版业炎的伤害次序、三血费用时点、默认强化配置和只分配两火，均有具体偏差；补叙分类查牌不同，整经仅原生分支实跑且上游入口实际受依赖阻塞。精确 inventory 键、规则源码落点、证据等级与未测设计在 [findings.json](findings.json)。关联 `th_zhengjing2` 未作本轮运行签署，也不因主技能结果自动改状态。

[逐场表](case-review.md)完整纳入冻结后的25场：11场仅所列投影相符；3场类别结算不同；3场费用/伤害时序不同；2场配置维度不同；1场原生缺少两火分配表达；1场取消交互分段不同；4场整经原filter实际抛 `dcdAnim is not defined`。21场完成目标分支不表示21场规则通过。逐场初态、配置、实际DOM与CDP行、原生场景复用关系见 [case-review.json](case-review.json)。

先独立读实际上游、原生及内核源码，再接收冻结输入；未读采集者 README、findings、比较器来决定预期。原版 liuli 1.10.1.0.1 通过真实 phaseUse/chooseToUse 进入，用原 UI handler 和 CDP 鼠标触发原技能属性；5个技能、9个辅助函数和1个卡牌属性与上游精确重组一致。扩展 content/precontent 没执行，不能当整包装载验收。整经在原filter就实际异常，未加载官方Spine、未stub、未提前置 `zhengjingxiao`；小游戏之后仅用逐行源码替代，未声称双侧完成。

[输入回执](frozen-input-receipt.json)列精确 hash 和正式包内容锁。354个采集文件的hash/size全符；3330对请求/回包时间对应，最终cohort的711对请求覆盖146个可逐一回溯的快照，全部25场先fresh reload、再排空真实onfree/cardsAsync才建初态。没有采用旧轮终态。详见 [完整性审计](raw-integrity-audit.json)、[快照原始行索引](raw-snapshot-index.json)和[实际入口核对](entry-and-boundary-audit.json)。读取原报告文件仅用于完整manifest字节hash，不以报告文字为规则依据。

全部25场的八座HP/maxHP/生存/横置/翻面/护甲、手装判、技能、限定已用、映射扩展区和125个native-ID的牌区及顺序均核。严格逐牌首轮为0/25相符，因为两张上游闪电有固有thunder、原生face输出null；[首轮输出](audit-raw-first.stdout)保留。随后只声明这一固有属性归一化，初态25/25匹配，见 [投影审计](projection-audit.json)。它不等于原牌堆一致：21个映射牌面不在原standard/extra牌表（原两包共161张）。补叙装备/twice实际取得79黑桃1绝影，full初态89黑桃1青釭受此限制；无懈分支实际用到的70黑桃J无懈、原生1黑桃8杀以及所有业炎费用0/7/21/24均有原牌表元组。其余闲置的非原元组不用于声称完整实体一致，也不抹去已用字段上的费用/时序反例。观察区间内所有场末态无外来牌/空ID，双方物理ID守恒。

独立外部 [NGSA replay probe](native-replay/src/main.rs)读取采集者的正式冻结包和旧archive，在当前产品上实跑：39个pending、22个final恢复全部成功；39个恢复前完整Snapshot只归一化auth后相同，39次对答的hash、回包和新增日志均与原始记录相同。38次恢复后的完整Snapshot可与下一pending或final逐字段匹配；`buxu-twice`第一轮分类答复后另有第二次ActivateSkill，缺少该答复直接后的独立NGSA，因此此处只核hash/日志/回包，不编造第39次完整后态对照。22个final完整恢复Snapshot相同。原始 [重放JSONL](native-replay-first.jsonl)和[编译stderr](native-replay-first.stderr)保留，完整前后Snapshot文本及新续行NGSA位于 [native-replay/first](native-replay/first/)。这些验证仅是同原生存档续行，未比较完整上游state/events/RNG。

本机工具链为 rustc/cargo 1.99.1（aarch64-apple-darwin），独立probe使用共享默认target，已释放，未清共享target。未启动独立Chrome/服务，未创建私有target或缓存。实际命令、退出码、首轮牌表解析脚本的 `SyntaxError` 与修正分别留在 [命令与首次失败](commands-and-first-failures.json)；源规则、原始期望均未为通过而改写。采集者早轮onfree污染和首次UI/提取错误仍在原L28目录，只将其视为历史输入而非最终结果。本批未运行完整产品门禁或浏览器重新采集，由主 agent 统一整合与发布。
