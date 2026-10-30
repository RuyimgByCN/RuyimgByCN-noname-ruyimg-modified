# ADR 0787：原生房间选择超时、托管与耐久动作记录

日期：2026-10-30　|　状态：**用户已采纳，实施与验收待完成；须在J2规则329实际验收接入后实施**

授权：[裁决20](../operations/evidence/2026-10-28-user-decisions/README.md)、[获批方案A及父审查修订](../operations/evidence/2026-10-30-next-version-proposals/README.md)。完整只读源码审查与未来用例见同目录timeout-proposal-raw.txt；不是运行证据。既有超时策略来自裁决15，不重新改变。

## 问题与既有边界

原生actor没有选择计时器，既有31秒反例仍未关闭。当前NativeRoomPersist只有内存动作段，耐久房间恢复主要依赖最新快照；不能将内存record称为磁盘WAL。CreateRoom没有可复用的房间options通道，必须正式接线配置和托管反馈。

保留单写actor、现有NativeRoomState/RoomDurability/StoredRoom/checksum/journal及成员收据，不另建存储或协议入口。规则核心不读时钟。

## 决策

1. 房间可选10/15/30/60/90秒，默认30秒。actor到时构造内部 `GameAction::TimeoutAction { choice_id: u64, selection: Vec<ChoiceOption> }`，记录实际选择。它与Answer共享观察者、选择数量/候选、revision、去重及故障回滚校验；客户端/Worker/旁观/裁判没有该特权入口。已有公开CommandKind::TimeoutAction不因此获得原生调用权限。
2. 仅首次接管使用TimeoutAction；后续托管输入仍是明确Answer/NextPhase。使用授权PolicyInput/Own ChoiceView按公开候选取最少合法项的既有基线代答，不查询真实暗牌或搜索未知VM续行，不伪称强AI。每次至多推进一条然后归还actor调度，实际动作全部记录。
3. 配置、托管座位、当前choice id/observer/Unix截止和WAL进入现有房间JSON、session checkpoint与同一耐久事务。当前选择不因快照、断线、重连或同窗RevealGeneral而加时。停机计时；进程内单调计时等待，重启按保存的服务端绝对截止恢复剩余时间，已过期只为当前窗口排一次候选。时钟异常不能被当成零时间自动接管。
4. 本人新的合法操作被接受或有效令牌重连才清托管，历史成功收据重放、快照、心跳、非法报文不清。断线/重启保留托管。actor调度边界已到期的timeout先于尚未取出的消息；已开始处理的人工命令先完成。不以客户端发送时间判顺序。
5. 耐久WAL复用已有快照/命令段策略，保存真实actor、expected_revision、dedup_id、动作、接受后revision/hash；服务器私有穷举DTO编码所有实际GameAction/ChoiceOption，内核不因JSON增加serde依赖。候选中先追加动作、托管、截止、玩家收据，再commit再广播；失败整组回滚。系统动作不伪造玩家令牌/收据key。
6. 恢复只回放保存的实际命令，不再使用人造revision*2去重号或重新调用AI；逐条核对结果及最终完整语义快照。assign_general等不推进revision的既有状态变更重置WAL快照基准，不能留下重放缺口。段压缩保留确认检查点，不声称已经交付永久录像历史。
7. rename后不确定沿用现有Prepared/Committed及requires_restart规则，计时器不能绕过持久化不确定状态继续写。内核拒绝不标托管、不写WAL、不消费确认dedup；稳定故障暂停该窗自动重试，不能零延迟空转。

## 协议与版本

- J2确实以329/758/105验收入库且无冲突后，本批独占规则 **329→330**。任何前提或版本占用改变先重新确认，不自动换号。预计保留原生存档105和ABI758：新增动作不是StateSnapshot字段，托管/时钟/WAL在外层房间JSON。
- **WS5→WS6**：CreateRoom/RoomRoster加入choice_timeout_secs，SeatPresence加入managed，授权ChoiceRequest加入可选deadline_unix_ms（离线/旧线None）。截止与choice同对象，省去三种外层回复重复字段。实际同步Rust/TS构造、创建UI、倒计时、托管反馈及权限投影。
- 严格会话握手及各入口拒绝WS5、未握手和0版本绕过，客户端也校验Welcome，不只是改常量。裁决21的兼容可选元数据随后并入同一个WS6，不再分配WS7。
- v329及更早原生规则档按门禁拒绝；保留原字节、冻结包/锁和匹配旧执行/回放途径，不通过修改头或checksum假造迁移。外层旧JSON的可省略字段必须证明旧校验和不被重序列化改变；失败不发布该房间。

## 验收与取舍

先红后绿关闭真实actor31秒反例；五档及默认、非法配置、人工/到时两序、RevealGeneral不续时、托管让出调度、无人入座/断线/重连、越权和幂等均覆盖。存储故障覆盖PartialWrite/FileSync/Rename/DirectorySync/Prepared/Decision及补偿双故障，核对全部候选状态和最终一次结算。

实际服务器进程/SIGKILL跨截止恢复、正式WS与UI反馈、记录选值在native与当前重建WASM Answer路径的全状态/事件/选择/RNG/续行对照都须执行；简单代答不代替NC10质量验收。对应原生三包、协议/服务端/WASM、前端、workspace/conformance、严格Gate1b、preflight和源码门禁按影响面完成，保留所有初次失败。golden先区分版本域和行为变化，再决定重录。

代价是新增服务器DTO和耐久命令段；收益是实际动作可重放且时钟不进入规则核。绝对截止依赖可靠服务端时间，生产校时异常须明确处理。本ADR只记录已采纳契约，不是实现/测试通过或NC09完成声明；当前结果以进度主稿和剩余台账为准。
