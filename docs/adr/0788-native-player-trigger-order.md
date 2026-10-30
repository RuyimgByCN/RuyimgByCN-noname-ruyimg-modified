# ADR 0788：原生同窗触发由玩家选择下一个技能

日期：2026-10-30　|　状态：**用户已采纳，实施与验收待完成；须在超时规则330实际验收接入后实施**

授权：[裁决22](../operations/evidence/2026-10-28-user-decisions/README.md)、[获批方案C](../operations/evidence/2026-10-30-next-version-proposals/README.md)。详细源码、两仓规则和未来验收清单见同目录trigger-order-proposal-raw.txt；不是实现证据。裁决14已确定玩家选序目标，本ADR确定其具体契约和版本边界。

## 背景与决策

现有队列采用全局优先级、强制/可选、座位和def的确定性顺序；两同人技能只能分别确认，不能选择谁先发动。已保存的counterexample通过是锁定这个旧差异，不是目标B验收。互动答案可以作为命令回放，确定性不要求删除互动。

保留串行结算栈、Frame.queue、park、现有OptionalGate及内容机器，不建立第二个调度器。以当前noname arrangeTrigger为主基准；liuli的silent优先等差异逐来源记录，不能据此无条件宣称两仓等价或豁免功能。

1. First → 普通玩家顺座桶 → Last。普通桶先seat、再有效priority降序，去掉optional排序优先级；First/Last保持自动。effective priority按来源真实元数据解释，比较降序时不对i32::MIN盲目取负。
2. 收集订阅实例时不因当时filter为false就永久丢弃；每次分派前共享复核实例存在/启用、当前可用性、触发上下文、限次和filter。同优先兄弟效果可以使另一个技能变得可用或不可用；降到更低优先级后不复活已跳过的高优先项。
3. 当前最高可用优先级内，有silent/无真值技能名译文的项先自动执行；其余按稳定技能身份去重，0/1个无需询序，2个以上强制/可选/混合都让该玩家选择“下一个”。不是一次锁死完整排列。可选技能仍在选定之后经过OptionalGate，选序不代表同意发动。
4. 复用Answer/ResolveChoice及PickTag，新增 **ChoiceOption::Skill(SkillDefId)** 与静态 **trigger_order** 键。技能ID不截断为u8，不用卡牌Kind冒充技能。内部WASM选项JSON携带稳定技能名，外部仍是原token/label字段，服务端可复用已有wire OrderTriggers值。
5. 一个小型TriggerOrder机器绑定frame/owner/有序唯一候选/asked，使用既有Trigger slot与WaitChoice。答案通过当前observer/id/revision/dedup/accepts校验，再复核选择仍合法；只移除选中项并走已有发动路径，不提前消费限次或发SkillTriggered。效果/子帧/可选确认结束后重建下一组。
6. 选序无客户端取消，空/多选/未知skill拒绝；实际可选发动仍可拒绝。状态变化使旧候选不再可用时不执行旧候选，必要时发布新choice id，不暗改同一token的含义。托管/AI以已有明确Answer选首个当前候选，选序本身不消耗RuleRng。

## 包元数据、状态与兼容

TriggerMeta和PackageTriggerEntry补timing、effective priority、order_selectable的必要信息；普通作者默认Normal/0/true。来源迁移按真实silent和技能名译文真值生成，不依据optional、当前语言或tooltip猜测。编译、合并、装载和冻结恢复保留这些字段，非法值拒绝。

元数据进入规范logic_hash/内容身份。默认值可哈希中性仅限旧读者测试证明非默认语义必被拒绝；直接ExtensionPack路径也须核验，不把name/version锁当作全部行为校验。真实修正版使用新不可变内容版本，原件/锁保留。若现有package format1/schema3无法安全表达，另提具体schema方案，不擅升。

新机器和Skill选项使用新的尾追加tag，不重解释旧payload；trigger_order尾追加现有静态白名单。codec/hash/ExpectedAsk/恢复/检查点同步验证frame、owner、候选顺序/唯一性、选项/asked和当前队列绑定。每个新增语义字段变动要么改变规范hash，要么作为非规范快照被拒绝；不得靠全局去重隐藏非法状态。

超时批实际规则330验收接入后，本批独占 **330→331**；预计维持存档105/ABI758和届时WS6。旧规则档按门禁拒绝、保留原字节/冻结锁及匹配旧执行途径，不伪造迁移。若需archive106、ABI759、新schema或版本被占用，先提交具体方案重新确认。该版本不能提前占用J2的329或超时330。

## 验收与剩余边界

先把旧反例改成能红的选序预期，再验证两个非交换真实效果、反注册序同名选择、3技能逐次选择、强制/可选/混合与拒绝确认。覆盖同优先filter启用/失效、跨玩家优先顺序、First/Last、silent/无译名、负/边界优先级、死亡/禁用/嵌套、真实DSL及Host+VM组合。

存档在开窗前、待答、答后效果前、OptionalGate、嵌套VM/J2、子事件和yield处跨字节/进程恢复，比较全状态/事件/选择/RNG/续行。恢复篡改、越权、重试、人工/timeout竞争、耐久各故障点都要拒错且只发动一次。实际当前WASM/Worker、服务器和产品按同一答案序列对照，技能按钮有真实名称、焦点/键盘、忙状态、恢复重绑定且第257项可达，不能只依赖256上限legal_actions。

57组是既有静态命中清单，每组都须真实来源filter/effect、适用模式、各实质选择方向和恢复证据，不用两个通用摸牌技能替代。forceDie/out、indexed、hidden、filterStop及既有阶段窗拆分等缺口仍要实现/另获必要授权并验收，不能因本ADR采纳将缺失功能改称已接受差异。动态授技、装备/全局和模式组合也不在57计数中被自动覆盖。

原生三包、schema/compiler/loader/WASM/server、前端、workspace/conformance、严格Gate1b、preflight及源码门禁按影响面执行；golden先解释版本域与真实行为变化再录。本文不声明任何未来测试已运行，当前结果只在进度主稿及剩余台账维护。
