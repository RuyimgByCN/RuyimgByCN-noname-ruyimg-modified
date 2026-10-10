# ADR 0732：N10 AI 只读视角会话与命令配额原型

日期：2026-09-16
状态：已实施（engine-native）
关联：[0730](0730-native-n10-player-view-isolation.md)、[0731](0731-native-n10-choice-event-visibility.md)、进度文档 N10

## 独立复核更正（2026-09-22）

set_seat_quota 实现的是每座位每轮接受命令数，不含工作线程、队列、取消或搜索预算，不能计作目标架构的有界 AI 计算池。
配额 1 时第一次推进后，下一次推进被 QuotaExhausted 拒绝，停在 round=1/Draw；轮次不能自行翻滚。
宿主调高/取消配额可以解锁，但原稿“不因配额死锁”不成立。AiSession.events 还存在 N10-a 私有应答泄露。

当前反例、优先级与证据见 [进度主稿](../architecture/native-progress.md)。以下保留当批设计/记录，不作为当前完整验收结论。

## 背景与决定

N10 剩余首批：AI 信息隔离接入与有界计算池。目标是让 AI 宿主**只能**通过
可见性投影观察对局，并对其命令频率施加可感知的有界预算（防止单座位霸占
调度或无界试探）。

### 1. AI 只读视角会话（`AiSession`）

`Engine::ai_session(seat) -> AiSession<'_>`：座位固定的只读会话——

- `view()` / `choice()` / `events()`：分别等价于 `view_for(seat)` /
  `choice_view_for(Some(seat))` / `event_view_for(seat)`；
- `quota_remaining()`：本轮剩余配额（有界池对 AI 可感知）；
- 字段私有、无权威状态入口——AI 代码以 `AiSession` 为参数即被类型系统
  限制在可见性投影内（0730/0731 契约的组合使用）。

### 2. 有界计算池（每座位每轮命令配额）

`set_seat_quota(Option<u32>)`：每座位每「轮」最多接受 N 条命令（None = 不限制，
默认）。语义：

- **拒绝不消耗配额**：只有被接受的命令（含 Pending/Suspended）计数；
- **轮次自动归零**：`turn.round` 变更后各座位池重置——回合轮转自然恢复
  公平性，不因配额死锁；
- **按座位独立**：一个座位超配额不影响其他座位的池（公平性）；
- **可感知**：`seat_quota_remaining(seat)` 供宿主与 AI 查询；
- **宿主可随时调整**：`set_seat_quota` 是逃生阀；
- 新拒绝原因 `RejectReason::QuotaExhausted`（信任边界扩展，宿主策略层）；
- 配额与计数为**会话策略数据**——不入状态/哈希/快照（与 time_slice 同层），
  恢复后需重新设置；不递增规则版本。

### 3. 回归（净增 2）

- `seat_quota_bounds_commands_and_resets_per_round`：配额 2 下第三条命令
  `QuotaExhausted`；余量递减；拒绝不消耗；座位 1 拥有独立满额池（公平性）；
  轮次翻滚后座位 0 配额重置；非当前回合玩家的越权拒绝不消耗配额。
- `ai_session_is_seat_bound_readonly_projection` /
  `ai_session_non_observer_sees_limited_choice`：会话视图与 Engine 投影一致、
  对手手牌 Hidden、观察者 Own/非观察者 Other、事件流无本座位外的摸牌 id、
  配额余量可感知。

## 影响

- 测试 156 项（native_kernel 85、view_isolation 7、boundary 59、场景 2、源码 2、
  调试 1），净增 3。规则版本不变（8）；golden 不变。
- N10 剩余：原生牌桌与输入延迟（需客户端/平台载体）。
