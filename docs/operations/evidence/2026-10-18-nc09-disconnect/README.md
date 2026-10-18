# NC09 断线中途（服务端崩溃）可见性 + 重启后行为（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC09**（[方案 §NC09](../../../architecture/native-convergence-plan.md#nc09)）「错误展示 / 重连」
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**发现并修复「中途断线不可见」缺陷**；服务端重启后房间不持久（客户端正确报错）——登记为待办

## 1 方法（可复跑）

```bash
(cd apps/web-client/dist && python3 -m http.server 8123 &)
node apps/web-client/scripts/nc09-browser-disconnect.mjs http://127.0.0.1:8123/ \
  "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" <outdir> ./target/release/game-server
```

新增脚本 `apps/web-client/scripts/nc09-browser-disconnect.mjs`：**脚本自身启停 game-server**——
入座 → `SIGKILL` 服务端 → 断言界面出现可见错误 → 重启服务端 + 重新导航 → 断言恢复行为。

## 2 实测结果

| 阶段 | 修复前 | 修复后 |
|---|---|---|
| 入座 | `房间 #1` | `房间 #1` |
| **服务端被 SIGKILL 后** | **无任何错误提示**（界面保持旧状态，`hasErrorText:false`） | **`连接已断开（服务端不可达）`**（状态行 + 战斗日志，`store.connected=false`） |
| 重启服务端 + 重新导航 | `房间 #1`（页面重连到**已不存在**的房间，仍显示旧状态） | `hasRoom:false` + **可见错误**（房间在服务端重启后不持久，客户端如实报错） |
| 控制台 | `[]` | `[]` |

证据：[disconnect-drive.log](disconnect-drive.log)（修复前）、[disconnect-drive-after-fix.log](disconnect-drive-after-fix.log)、
[disconnect-report.json](disconnect-report.json)、[disconnect-screenshot.png](disconnect-screenshot.png)。

## 3 修复（真实缺陷）

- `src/engine/networkClient.ts`：`handleClose` 新增 **断线监听者广播**（`onClose`，监听者异常不影响他人）；
- `src/views/BattleView.vue`：注册 `onClose` → 状态行写「连接已断开（服务端不可达）」、`store.connected=false`、
  战斗日志追加一条。

## 4 未完成 / 待办（NC09）

- [ ] **房间/对局跨服务端重启持久化**：服务端重启后房间不存在（内存态），客户端只能报错；
      需要房间恢复（WAL/快照落盘 + 启动恢复）——原生 `NativeRoomPersist` 已有快照/WAL 能力，
      但**未接线到房间生命周期**（登记）
- [ ] 断线**自动重连重试**（当前需用户重新进入/刷新；已有自动回房间逻辑但依赖房间仍在）
- [ ] 限流（`max_cmds_per_sec`）与慢消费者的用户可见反馈
- [ ] 联机存读档/录像、锦标赛面、平台与性能档位、服务端 SPA fallback
