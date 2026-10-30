# ADR0787 A：规则330／WS6隔离候选

核验日期2026-10-30，基线 `255bbfbd7f6916ee77bd29799826a225bd10871f`。本目录描述隔离候选；不表示主树已接入或NC09整体完成。授权为[裁决20](../2026-10-28-user-decisions/README.md)及[ADR0787](../../../adr/0787-native-choice-timeout-durable-control.md)，不包含C331、改命332或新的ABI／存档格式。

规则329→330，存档105、原生宿主ABI758保持。`TimeoutAction { choice_id, selection }`复用Answer的观察者／候选／数量／revision／dedup／故障事务校验，计时器不进入规则核。所有实际动作以穷举服务器DTO保存真实actor／expected_revision／dedup／选值／接受后revision及hash，和托管／deadline／收据在同一个房间耐久事务提交；恢复逐条重放并比较完整归档，唯一归一字段为既有恢复令牌轮换的auth_generation。显式Recover仍轮换认证代次，磁盘恢复不多轮换一次。

原生三包首轮因14个golden版本域变化失败；只将版本常量临时退回329、保留修复时，14个旧golden全部匹配。恢复330后按实际输出重录。实际329服务端生成的原始房间JSON在 `apps/game-server/tests/fixtures/adr0787-room329.json`；新读者保持其原checksum且按RuleVersionMismatch拒绝，未重写头／冻结包／旧锁。已有WS5字节读者样本保留header5，当前房间协议明确拒绝WS5、未Hello及header0绕过，不向旧服静默降级。

本次运行的原始命令／退出码、31秒actor红、330真实WS与SIGKILL重启、WAL前后字节、当前WASM字节对照、浏览器截图和候选patch保存在隔离工作区相邻的 `../evidence/a330`。两次provider SSE解析中断及恢复单独记录，不算测试结果。完整执行索引是该目录的 `commands.jsonl`；所有初次失败保留，不能以定向补跑改写初次全量失败。

已实测的边界包括：默认30秒actor红→绿、10秒真实actor与磁盘WAL、五档／非法配置、恢复不延当前窗、人工／timeout两序及历史收据不解除托管、客户端伪造控制拒绝、逐条WAL篡改拒绝，以及既有Prepared／Committed和补偿失败路径。后继托管只用授权PolicyInput最少合法项的Answer或NextPhase，每条归还actor调度，不代表NC10强AI质量或永久录像历史完成。

本候选须由父任务独立复审后集成，并在合并后重跑组合验收。真机、CI、37来源整包、玩家选序331与改命332均不由本批宣称完成。

最终隔离回归：原生三包1706/0/75；完整workspace最终6889/0/77、exit0；strict native3/all-targets、分层0、源码门禁通过。前端155/0及typecheck、最终fresh330 WASM／Worker82/0、五窗75/270/35/40、J2的36/189/114及完整归档相等、正式跨目标和旧线cross-target均通过。实际Chrome创建五档／10秒倒计时／托管与旧329服务端反向拒绝均留证。完整preflight首跑19/21仅因隔离仓外lookup缺失，父批准临时只读链接后完整补跑21/21；所有首次失败、夹具修正和provider中断保留，不回写原退出码。最终纯patch及每file SHA在运行证据中，待父独立源码审查和合并后的组合验收。
