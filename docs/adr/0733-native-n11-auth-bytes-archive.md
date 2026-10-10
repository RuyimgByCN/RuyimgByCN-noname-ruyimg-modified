# ADR 0733：N11 权威服务第一批——认证 actor 与版本化字节存档

日期：2026-09-16
状态：已实施（engine-native）
关联：[0732](0732-native-n10-ai-session-bounded-pool.md)、进度文档 N11

## 独立复核更正（2026-09-22）

同种子、同签发顺序的两个 Engine 令牌相等，跨实例提交被接受；保存恢复后重签也会让旧令牌重新有效（N11-a）。
现有 auth_tokens_do_not_survive_archive 未尝试旧令牌，不能证明旧凭证持续失效。
NGSA 规则版本头 24→999 仍解码/恢复成功；payload_len=u64::MAX 在 debug 构建溢出 panic。
Box::leak 每次解码选择提示词都会分配，不能按“一次性小规模”解释反复恢复。字节往返已实现，外部边界尚未完整验收。

当前反例、优先级与证据见 [进度主稿](../architecture/native-progress.md)。以下保留当批设计/记录，不作为当前完整验收结论。

## 背景与决定

N11 权威服务首批交付两项基础设施：命令发起者从宿主自报升级为**经认证的会话身份**；
快照升级为**版本化确定性字节存档**（此前内存快照无法落盘/跨进程传输）。

### 1. 认证 actor（`issue_token` / `submit_authed`）

- `issue_token(seat) -> AuthToken`：为座位签发 opaque 令牌（引擎种子混合计数器，
  跨实例不重叠）；重复签发覆盖旧令牌——旧令牌立即失效（重连语义基础）。
- `submit_authed(token, cmd)`：令牌不存在/已失效 → `NoSuchSession`；
  令牌绑定座位 ≠ 命令 actor → `Impersonation`；校验通过后走与 `submit`
  完全相同的主通道（配额/去重/修订/预算全部生效）。
- 令牌为**宿主会话数据**：不入状态/哈希/快照（与 time_slice/quota 同层），
  存档往返后失效，需重签（契约化并有测试锁定）。

### 2. 版本化字节存档（`src/archive.rs`）

`StateSnapshot` 的确定性序列化 + blake3 完整性校验：

- 布局：`magic(NGSA) + format_version(u16=1) + ruleset_version(u64) +
  payload_len(u64) + payload + checksum(blake3, 32B)`；
- **确定性**：小端定长整数、显式变体标签、BTreeMap 键序遍历、无哈希随机性
  ——同一快照两次编码逐字节相同（有测试）；
- **全覆盖**：core（玩家/牌库/弃牌/桌面/RNG 原始状态/回合/技能表/模式/胜负）、
  resolution（帧树含 13 类 EventData/15 类 EventKind、挂起机器全 18 变体、
  开放选择、应答、预算）、last_dedup、segment_open、递归段检查点；
- **防篡改**：payload 翻转一字节即 `ChecksumMismatch`；坏魔数 `BadMagic`；
  未来格式 `UnsupportedFormatVersion`；
- 选择提示词（`&'static str`）解码经 `Box::leak`（一次性小规模泄漏，契约化）；
- `ruleset_version` 载于头部供宿主快速核验，规则绑定仍由 `Engine::restore`
  强校验（跨规则版本存档不保证可解码——SkillDefId/CardId 按目录数值编码）。

### 3. 回归（新测试目标 `tests/auth_archive.rs`，5 项）

- `authed_submit_rejects_impersonation_and_unknown_token`：有效令牌接受、
  冒充拒绝、未知令牌拒绝、Answer 通道同受约束。
- `auth_token_reissue_revokes_old`：重签吊销旧令牌。
- `snapshot_archive_roundtrip_rest_and_window`：静止点与改判窗口两个快照——
  解码后逐字段相等（StateSnapshot 本批补齐 PartialEq/Eq derives）、
  恢复出同哈希引擎、恢复件可继续应答改判窗口。
- `archive_detects_tampering_and_version`：防篡改三断言 + 编码确定性。
- `auth_tokens_do_not_survive_archive`：存档后令牌失效、重签可用。

## 影响

- 测试 161 项（新增 auth_archive 5 项），净增 5。规则版本不变（8）；golden 不变。
- 结构性派生补充（GameStateCore/Frame/ResolutionSnapshot 等补 PartialEq/Eq）：
  纯 derive，无编码影响。
- N11 剩余：重连/乱序/去重的会话层（去重已在引擎，服务层待接）、完整模式/内容；
  N12 跨进程与真机验证（字节存档为其前置，已就位）。
