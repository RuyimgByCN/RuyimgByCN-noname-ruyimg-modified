# NC12 联机往返时延探针：**未取得有效测量**（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC12**（[方案 §NC12](../../../architecture/native-convergence-plan.md#nc12)）「联机 RTT」
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**探针未取得「成功命令」的往返时延**——30 次提交全部被服务端拒绝（`result_rejected=30`），故**不作为 RTT 结论**；根因待查（登记）

## 1 做了什么

新增 `apps/web-client/scripts/nc12-online-rtt.mjs`：Node 原生 WebSocket 直连 game-server，
按协议手工发送 `Hello`（`network_protocol=5`）→ `CreateRoom` → 第二个连接 `JoinRoom` →
`SubmitCommand(EndPhase)`，测「发出 → 收到 `CommandOutcome`/`Error`」的墙钟时间。

## 2 实测（[rtt.log](rtt.log)）

```
STEP hello ok
STEP room 1
STEP join2 JoinOk
STEP startgame Error {"code":"Internal","message":"room error","room_id":1}
STEP first_rejection {}
RESULT scenario=online_rtt tier=loopback n=30 p50_ns=100541 p95_ns=174166 max_ns=339917
RESULT scenario=online_rtt tier=loopback result_accepted=0 result_rejected=30 final_revision=0
```

**判读（不得当作 RTT 结论）**：
- 30 次提交**全部被拒**（`Accepted=0`）→ 测到的是「错误回包的往返」而非「成功命令的往返」；
- 服务端日志：`client handler error conn=1 error=Internal: room error`；`StartGame` 也返回
  `room error`（原生会话的 `submit` 不支持 `StartGame` 形态，属预期；但随后 `EndPhase` 仍报
  `room error`，**根因未定位**）；
- **对照事实**：真实浏览器路径（`nc09-browser-online.mjs`）中同样的「结束阶段」命令**被接受**
  （`rev 0 → 1`，见 [联机闭环证据](../2026-10-18-nc09-browser-online/README.md)）→ 说明差异在
  **本探针的手工报文**（头字段/幂等键/命令体与 `networkClient.ts` 的构造口径不同），不是服务端缺陷。

## 3 未完成 / 待办

- [ ] **修正探针报文**（对齐 `apps/web-client/src/engine/networkClient.ts` 的 header/`Command`/
      `IdempotencyKey` 构造）后重测，取得**成功命令**的 p50/p95
- [ ] 真实网络（非回环）RTT、丢包/重连抖动、慢消费者背压下的时延
- [ ] ≥30 分钟持续负载与温控；其余平台档位；其余扩展包逐包性能；旧线对照
