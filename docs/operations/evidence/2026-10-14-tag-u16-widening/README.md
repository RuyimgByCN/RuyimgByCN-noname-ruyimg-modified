# HostBehavior tag 值域 u16 扩容（2026-10-14）

## 任务来源

[玩点论杀批 10](../2026-10-14-t12-wudian-generals-b10/README.md) 登记边界：u8 值域
已满（256 变体全占用，96 为历史空洞回收）——本前置批扩容，泛化解锁后续所有新
host 行为。

## 改动

| 面 | 改动 |
|---|---|
| `host.rs` | `tag()` → u16（编号只增不改——ABI 兼容对包数据无影响：包内为 `host:*` 字符串键，tag 仅内部编号与存档快照）；`from_tag(t: u16)`；`_` 兜底臂恢复（u16 值域有空洞语义） |
| `snap.rs` | `MachineSnapshot::Host.behavior: u16`；`active_ask_host` 签名 u16；两处 `from_tag` 消费（一处 `u16::from` 升位兼容） |
| `archive.rs` | Host 快照 enc `w.u16` / dec `r.u16`；**存档 v65→v66** |
| `engine.rs` | 规范哈希 push_u8→push_u16（Host 挂起机快照编码） |
| `a0_tag_registry.rs` | 守护测试扫描域 u8::MAX→u16::MAX（既有守护：tag 唯一/往返/可达数=ALL） |

## golden 与兼容性核验

- **14 golden 不变**：Host 挂起机仅扩展技能挂起时存在，标准场景不可达——哈希面
  不受编码扩宽影响（存档版本不入 canonical hash）。
- **旧存档兼容**：NGSA 版本门 v66 拒绝 v65 及更早存档（既有版本门语义，非本批
  引入）；版本内往返（save/restore/跨进程接力 t8 套件）全绿。

## 实测记录

| 项 | 结果 |
|---|---|
| engine-native 全量 | **738/0/0**（含 a0 tag 守护 2/2 与 14 golden 不变）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |
| 装载/包管理/服务端 | **84/0** |

## 登记边界

- 反例实证形态：a0_tag_registry 守护测试即本面回归（唯一性/往返/可达数），批 10
  的 u8 溢出编译错误为扩容必要性的编译期实证。
- HostStage 不扩（值少）；后续若扩展同样循 u16。
