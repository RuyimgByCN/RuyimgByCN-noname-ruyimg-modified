# 下一批协议与规则变更审批稿（2026-10-30）

**提议，尚未获批、尚未实施。** 源码核对基准为 `38789f54`；main仍为规则328/存档105/原生ABI753/WS5，J2的329/758还在隔离树修复和验收。以下只在前置实际通过、版本未被占用时生效，不能提前抢占J2的329。若版本、格式或ABI前提改变，重新提交具体方案。

依据[规范第0节第10条](../../../../../NONAME_NEXT_AGENT_SPEC.md)：“新立 ADR、改协议（含 VM 提交协议与选择／命令协议）、升规则版本／存档格式／ABI 之前，先查裁决登记与剩余台账……超出范围的先向用户提交方案并取得明确答复。”既有[裁决14、15、17](../2026-10-28-user-decisions/README.md)已定玩家选序、超时目标和J2范围；本次不重新请示这些目标，只请示下面的具体新增契约与版本。

## A. 超时：J2之后独占规则330，并交付WS6

批准按[超时原始源码审查稿](timeout-proposal-raw.txt)形成正式ADR；J2实际329通过并入库后，单独使用规则329→330，预计保留存档105/ABI758。核心新增 `TimeoutAction { choice_id: u64, selection: Vec<ChoiceOption> }`，复用Answer校验，记录服务端实际选值，恢复只回放记录、不重算AI；网页、WS客户端、旁观和裁判都不能调用这个特权入口。

超时配置仍是已定的10/15/30/60/90秒、默认30秒。配置、托管座位、当前选择截止和实际命令WAL放入现有房间JSON/同一持久化事务，不建第二套存储；人工/托管动作一起记录真实actor/revision/dedup/选值/hash。写盘失败回滚所有候选，提交不确定沿用现有重启恢复规则。

停机时间计入截止；重连不延长仍挂起的选择；本人新合法操作被接受或有效重连才清托管，历史收据重放/非法报文不清。进程内用单调计时等待，重启用保存的服务端Unix截止恢复剩余时间；时钟异常明确处理，不将读取失败当作零而自动接管。同一调度边界已到期的timeout先于尚未取出的消息，已经开始处理的人工命令先完成。

WS5→WS6，明确拒绝旧版本/未握手/0版本绕过。新增字段：CreateRoom与RoomRoster的 `choice_timeout_secs`；SeatPresence的 `managed`；授权ChoiceRequest的 `deadline_unix_ms: Option<u64>`。**父审查收敛：截止放在已有ChoiceRequest旁的timeout_policy语境中，与choice同对象绑定，省去原稿在JoinOk/Snapshot/CommandOutcome三处重复字段。** 离线/旧线为None，旁观没有私有choice也没有私有deadline。同步全部构造者、客户端及UI，保留旧字段含义，不新增Worker命令。恢复时v329及更早原生规则档明确拒绝，保留原字节/冻结包/旧锁，不改头和校验和伪造迁移。

## B. 完整裁判与普通观察者元数据：兼容的可选字段

批准[原始字段方案](referee-wire-proposal-raw.json)的精确字段：SeatView尾加 `generals: [Option<GeneralSlotView>; 2]`，顺序主将/副将，槽含 `def/name/faction/revealed`；ChoiceRequest尾加可选 `prompt` 与 `native_kind`（confirm/select_cards/select_players/respond_card/pick_tag），保留旧 `character_def` 与 `kind` 回退，不新增ChoiceKind变体。

所有数据只来自共享授权投影；未公开暗将为null，裁判来自服务端可信成员而不来自客户端role，裁判无命令/应答权限。同步实际WASM/Worker、IPC、服务端和UI双将/选择展示，不读取隐藏原始状态补字段。旧服务缺字段→新默认、新载荷→旧Rust/TS读者、同档各权限投影、保存恢复/真实WS重连都要实测。

这些字段可以先作为WS5可选兼容追加，必须证明旧读者忽略新字段、新读者接受缺省；随后随A的WS6整体发布，不另占WS7。若兼容证明失败，暂停这条先行路径，将字段与A同批WS6明确拒旧；这不授权新的枚举变体、必填字段或另一个协议号。

## C. 玩家选序：A之后独占规则331

批准[触发选序原始源码审查与无编号ADR稿](trigger-order-proposal-raw.txt)，A实际330通过后单独使用330→331；预计保留存档105/ABI758及当时已发布的WS6。以当前noname的arrangeTrigger为主规则基准：普通触发按玩家顺座桶→有效优先级，去掉强制/可选作为排序优先级，每次执行前重验filter；同人同优先级的强制/可选技能都可选“下一个”，First/Last及silent/无译名自动执行。liuli分支差异单独逐来源对照与记录，不能据此宣称两仓无条件等价或豁免来源功能。

复用Answer/ResolveChoice与PickTag，新增类型化 `ChoiceOption::Skill(SkillDefId)` 和静态 `trigger_order` 键；内部WASM选项JSON新增skill名，外部仍用既有token/label字段，wire可使用已有OrderTriggers值。新增小型可恢复TriggerOrder机器；所有新状态同步codec/hash/恢复校验/AI与产品消费者，不截断为u8，不用卡牌Kind冒充技能，不一次锁死整排列。选择本身不抽RuleRng，真实技能效果仍可消耗随机。

包触发索引补 `timing/effective priority/order_selectable`，纳入逻辑hash/冻结身份及旧读者拒绝测试；按真实来源元数据生成，不依据当前语言或optional猜测。新机器/选项只用新的尾追加tag，旧规则档拒绝并保留原件；如实际需要archive106、ABI759或新schema版本则另报，不能自动升。

57组只是已有静态清单，逐组真实filter/effect、双向选择、嵌套选择/恢复、各适用模式、实际WASM/服务端/产品都须验收；forceDie/out、indexed、隐藏技能等现存缺口仍为未完成，不能批准本稿就把缺失功能改称已接受差异。

## 验收与边界

三项都保留先红后绿、全状态/事件/选择/RNG/续行、幂等、越权、持久化失败与进程恢复检查；前端、当前源码WASM/Worker、原生三包及受影响依赖、workspace/conformance、严格Gate1b、preflight和源码限制按影响面执行。旧golden先解释版本与行为原因再决定重录。原始审查稿中的测试清单都是未来要求，没有在本审批批运行。

本稿不批准push、新remote、NC10的VM provenance变更、额外版本号或省略真机/CI/37来源验收。既有本地提交授权不变。

本审批文档的链接、严格引用、旧值和行数门禁核验命令/结果见[review-checks.json](review-checks.json)；这些仅证明文档可审阅，不代表上述实现或验收已发生。原始审查文本与JSON摘要见[字节清单](proposal-artifacts.json)。
