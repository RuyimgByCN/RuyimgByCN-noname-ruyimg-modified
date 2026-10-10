# ADR 0735：N12 跨进程验收——存档落盘、跨进程恢复与同哈希续跑

日期：2026-09-16
状态：已实施（engine-native/examples/n12_crossproc.rs）
关联：[0733](0733-native-n11-auth-bytes-archive.md)、[0734](0734-native-n11-session-layer-reconnect-dedup.md)、进度文档 N12

## 独立复核更正（2026-09-22）

本轮版本 24 双进程复跑退出 0，存档 679 字节；额外提取并比较 writer/reader 的 H25/H35 相等。
reader 源码只是打印 match，没有失败断言；本轮由外部比较确认。该证据只覆盖同宿主、该保存点与脚本，不证明所有挂起机器分支或跨平台一致性。
头部/长度等存档边界另见 N11-b。

当前反例、优先级与证据见 [进度主稿](../architecture/native-progress.md)。以下保留当批设计/记录，不作为当前完整验收结论。

## 背景与决定

N12「跨进程验收」的前置——字节存档——已在 [0733](0733-native-n11-auth-bytes-archive.md)
就位。本批补齐进程对验证：**进程 A 布局并存档落盘 → 完全独立的进程 B 从文件恢复
→ 同哈希 → 同策略续跑终态一致**。这证明存档保真（RNG 状态、挂起机器、预算、
去重高水位等全部影响未来的状态都在档），且不依赖进程内任何隐式状态。

### 设计（`examples/n12_crossproc.rs`，write/read 双模式）

- 两个进程各自独立：同种子 443 → 同一确定性策略（牌选项优先应答/可达出杀/
  推进阶段）独立重放 25 条命令，到达相同状态（各自计算 H25）。
- **write**：存档落盘（610 字节），打印 H25；继续 10 条打印 H35。
- **read**：从文件解码恢复，断言恢复件哈希 = 本进程重放基线 H25（match=true）；
  续跑 10 条打印 H35。

### 结果（release）

| 进程 | H25 | H35 |
|---|---|---|
| writer | `388ead82…62997b` | `37230347…155bed0` |
| reader | `388ead82…62997b`（match=true） | `37230347…155bed0` |

H25/H35 逐字节一致；原始输出存档
[writer](../operations/evidence/2026-09-16-native-n09/n12_writer.log) /
[reader](../operations/evidence/2026-09-16-native-n09/n12_reader.log)。

## 影响

- 规则版本不变（8）；golden 不变；库测试数量不变（本批为 example 验收）。
- N12 剩余：真机验证（需物理设备载体）、旧线等价负载对照（需旧线可运行）。
- N11 剩余：完整模式/内容。
