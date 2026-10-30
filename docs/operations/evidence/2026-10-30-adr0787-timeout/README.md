# ADR0787超时330／WS6：隔离复审与主树集成（2026-10-30）

**当前：本地主树组合验收与最终门禁已完成，按裁决18收口入库。** 验收基线`9dbc655`保留已验收B/J2、资源及CI入口；53个源码/测试/fixture路径逐字节等于已复审A候选。规则330／存档105／ABI758／WS6，WASM仍为默认。验收命令执行时HEAD为329的`9dbc655`，提交身份以本批Git记录为准。

## 已实现、已复审的隔离范围

按裁决20/ADR0787，内部TimeoutAction共享Answer校验，房间actor使用公开最小合法PolicyInput，不伪造玩家收据；每条自动动作真实actor/revision/dedup/selection/接受revision/hash写入既有耐久事务。配置、deadline、托管及WAL一起提交再广播。重连/Reveal不续当前窗口，停机用持久Unix期限、进程内用单调等待，一轮仅一个跟进。WS6拒旧、未Hello、0版本及外部伪造系统动作，普通观察者/Referee权限按授权投影维护。真实329档checksum先核，再拒RuleVersionMismatch，不伪造旧档迁移。

初版独立复审指出Recover暂停未清、客户端直接相减Unix/本机时钟两个P2。原writer均保留实际红再修复：仅成功Recover耐久commit后清暂停；既有Heartbeat/Pong的server_time_ms绑定当前socket/protocol/conn_seq/room，RTT/2校准，performance.now推进，未知显示校准中。没有新WS字段。原reviewer确认两P2关闭、无新P1/P2。

P2修后隔离结果：workspace6891/0/77、server/protocol451/0、frontend158/0/typecheck/私有Vite、strictnative3、完整preflight21/21。真实actor故障→失败Recover→成功Recover原窗恰一次重试；实际Chrome±60秒、+1小时跳钟、原窗断连重连约6秒、deadline不变且每局一条TimeoutAction。恶意Pong/RTT负值/重复/旧socket分支来自Node模拟传输配实际SFC计算，未冒称全部浏览器实测。

这些是隔离树结果；该树不含主树资源增量，不能当作本地主树前端或完整组合结果。首次19/21规范lookup失败、所有中间编译/codec/版本/golden失败、两次provider SSE中断、原P2红及警告/ignored保持原始记录。14 golden只在回退规则常量能匹配旧值的归因探针后重录。未变化的WASM输入按原构建来源区分，不把actor P2修复记成重编WASM。

## 主树入口与余项

[主树接入审计](parent-source-admission.json)记录53项精确SHA及纯补丁3ca2d2c777c3736377cdceb923b5be824678c94fe989c0334ca50733aea4f26f。候选旧进度文本未覆盖主树状态；原源码基线255及全部raw证据单独归档。主树Phase1已完成原生三包1706/0/75、影响六包6097/0/77、完整workspace6891/0/77、strict/all-targets、conformance、前端161/0/typecheck、分层0和行数门禁；31808维护输入前后无漂移。Phase2当前330WASM/正式Worker82/0、仓库外Vite、正式轨迹与旧线crosshash、J2 36/189/114、五窗75/270/35/40、B14/98和ordinary5/40/32通过，直接Planner、空Hujia、真实WS/SIGKILL/WAL/校时页面与当前资源代表例通过。标准11份header330/105；3个包场景保留当前content-lock派生的真实u64规则身份，非把所有header强制改330。见[主树Phase1](main-phase1-handoff.raw.txt)、[主树Phase2](main-phase2-handoff.raw.txt)及两阶段完整命令/指纹。Phase2原39次尝试34项exit0/5次私有驱动失败与一次手动清理路径guard拒绝均保留，不能写成原尝试全通过。

最终[独立组合复审](main-combined-independent-review.raw.txt)结论为OK with notes、无新P1/P2；[独立完整JSON比较](main-acceptance/independent-review/formal-object-check.json)核对全部已记录键、类型和值，仅formal的已核实WASM产物locator例外，轨迹自身不含完整engine state。Planner输出、Referee操作前后完整state不排除字段。最终[四门禁](main-acceptance/final-gates/index.json)全部exit0：sourcecaps自测2/2、8989文件行数门禁、strict docrefs1065引用/越界0、[完整preflight21/21](main-acceptance/final-gates/04-full-preflight.log)（锁内328.674秒）。docrefs的六类未定位文件提示保留，不冒称全部引用clean。[最终指纹](main-acceptance/final-gates/summary.json)确认53源码、CI4、四份冻结输入、guard、HEAD与空index均不变；本轮未重跑31808全量指纹，不将既有Phase2摘要算成第三轮证明。未重跑已经通过的Cargo/WASM/UI链路。

本批完成的是本地A330范围，不替代9dbc首轮真实CI的5成功/5失败；Linux/Windows修正及下一轮真实CI另行归属，不冒跨平台全绿。首次暂存被精确字节核验阻断：Git将Windows原始CI日志CRLF转为LF，原始SHA与该次index SHA见[保留的失败记录](parent-first-staging-byte-failure.json)。根目录新增`.gitattributes`，仅对此精确原始日志路径禁用text转换；保留原字节，不改全局Git设置或原始日志。该归档配置是最终门禁之后唯一新增的维护输入，53项运行源码不变，不能继续宣称当前整树与Phase2的31808份输入完全相同。

共享stage3只在原writer明确结束且lsof确认无人使用后交给本批验收；CARGO_INCREMENTAL=0、jobs2、全局串行锁，每编译核盘并清本批无后用增量/临时产物。语音隔离任务原先只读链接主树329WASM，先保持该输入字节不变并私有冻结，确认后才覆盖主树ignored模块。默认dist不作为本批构建目录。

C331的A330本地验收前置已满足；新增包schema兼容补充尚待用户批准，不能据本批自动升schema4。改命332仍排在331实际验收之后；NC10持久标量反例、真机、HID/池收益、37来源整包及原生默认切换均不由本批关闭。GitHub发布许可见裁决25，实际CI状态独立登记。
