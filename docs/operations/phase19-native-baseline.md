# phase19：原生 runner 实跑与性能验收边界

独立实跑：**2026-09-23，第 14 轮，规则版本 28，release，engine-native**。
本轮 counting runner 退出 0，FFA/身份各 20/20 终局；未完成旧线同负载、跨平台或发布性能验收。

## 命令与原始记录

```bash
N09_SAMPLES=/tmp/native-samples.csv cargo run --offline --release -p engine-native --example n09_runner -- 20 9000
cargo run --offline --release -p engine-native --example n10_table -- 5 7000
cargo run --offline --release -p engine-native --example n12_crossproc -- write /tmp/native-archive.bin
cargo run --offline --release -p engine-native --example n12_crossproc -- read /tmp/native-archive.bin
```

[本轮日志/复现入口](evidence/2026-09-23-native-review-14/README.md)、
[runner 日志](evidence/2026-09-23-native-review-14/runner.log)、[CSV](evidence/2026-09-23-native-review-14/samples.csv)、
[环境](evidence/2026-09-23-native-review-14/environment.txt)、[源码摘要](evidence/2026-09-23-native-review-14/source-hashes.json)。
CSV 无表头，四列 load,game,seq,ns，共 **11758 条**。
macOS 27.0（26A428）、arm64、Homebrew rustc/cargo 1.98.0；本轮未固定记录设备型号/内存/功耗/温度。
每配置只一次运行，不计算稳定开销比例或跨实现提速倍数。

## 规则版本 28 数据

| 负载 | 命令 / 接受 / 拒绝 / 事件 | counting P50/P95/P99（ns） | 结束条件 |
|---|---|---|---|
| FFA 4 人 × 20 | 5341 / 5302 / 39 / 13926 | 2458 / 3042 / 3542 | 20/20 终局 |
| 身份 5 人 × 20 | 5932 / 5866 / 66 / 15421 | 2750 / 3250 / 3583 | 20/20 终局 |
| 判定专项 × 10 | 485 / 485 / 0 / 1371 | 2417 / 3000 / 3250 | 有界脚本结束，outcome=None |

- 拒绝均为 sha_limit；分位混合接受/拒绝，CSV 无结果列，不能拆两类延迟。
- wall_us 是被测调用耗时之和，不含完整建局/策略/输出；不能作整局端到端耗时。
- 判定专项恒 10 组，不随 games 参数改变。每组 Finalized=6、Gained=2、Rewritten=4–6（含 5）。
- save/restore 均值 **136/1146 ns**，各 2000 samples，仅该快照的克隆/恢复。
- 累计分配 **55934930 bytes / 359648 次**；峰值存活 **464494 bytes**，打印时存活 **335036 bytes**。
  含 runner、样本和日志，不是纯内核或 OS RSS。
- 旧版本与本轮规则状态哈希不同；命令/事件数相同也不能倒推全部新增技能已被 runner 覆盖。

## 宿主与 AI 示例

| 项目 | 本轮结果 | 解释与限制 |
|---|---|---|
| n12_crossproc write/read | 687 字节；writer/reader H25、H35 额外比较相等 | 同宿主两进程，该保存点恢复成立；不是跨平台或全部技能恢复 |
| n10_table 5 7000 | 5/5，14207 样本，P50/P95=709/2916 ns；日志有 12628 次 ShaLimit | 仅视图组装/策略/submit 路径，约 88.9% 样本是拒绝；恢复性 fallback 提交未计入样本，无输入设备或渲染帧提交 |
| n11_kof seed 11（临时诊断副本） | 三轮：2048/53/125 命令；首轮 1998 次 ShaLimit 且 outcome=None，后两轮 Player(1) 终局 | 0:3 包含一局驱动超时判负；不能写三局自然终局 |
| KOF 会话解析 | 当前解析器空首池往返成功；损坏格式/越界返回 None；重复将/矛盾场次仍接受 | 空池原反例关闭；一致性和真正从恢复件续跑未完成 |

`n10_table` 输出虽然标记 `metric=input-to-frame-submission`，计时到 run_policy 返回即结束，
没有 frame 或 GPU 提交；当前数字只能称 **CPU 策略提交调用延迟（混合拒绝）**。
与早期仅 submit 的计时范围不同，不能直接相除比较。
原生 host_assembler、render-host 本轮未重跑；明置授予技能/体力由工程测试验证，六技能已接线。
同宿主跨进程示例的保存点通过，不覆盖本轮复现的护驾/离间/决斗/AOE/无懈等恢复问题。

## 旧线与历史证据

- [第 13 轮版本 26](evidence/2026-09-22-native-review-13/README.md) 保留当轮性能与缺陷证据；其六技能接线、流离内存恢复、KOF 空池问题已在本轮更新状态。

- ADR 0773 的 [5 局新线日志](evidence/2026-09-22-n09-n12-load/native-n09-load.txt) 是他方批次记录；
  [补充索引](evidence/2026-09-22-n09-n12-load/README.md) 标明原对照 README 当时缺失。
- ADR 0767 的浏览器完局及 ADR 0773 的选将边界是历史集成记录，本轮未复验。
  UI 操作不等于规则命令，DOM 观察不等于事件总数；157 张终态总数不证明逐牌全过程守恒。
- [上轮版本 24 证据](evidence/2026-09-22-native-review/README.md) 保留 counting/plain 各一次、旧版示例与反例。
  该轮的令牌/日志/头长度反例已在后续修复，不继续作为当前故障；性能数据不混作版本 28。
- [2026-09-20 版本 5 失败日志](evidence/2026-09-20-native-review/runner.log) 与
  [早期 N09 日志](evidence/2026-09-16-native-n09/runner.log) 仅作历史。
- [phase17](phase17-test-perf-infrastructure.md) 中 Rust 旧参考线数据是第三种执行线，不能混排提速。

## 后续验收

1. 修复当前信息隔离、恢复与规则反例，固定相同模式/技能/牌序/动作及信息条件。
2. 记录接受/拒绝/事件实数，拆延迟类别，并计入 fallback、存档、恢复、中止等完整路径。
3. 固定设备和运行条件，重复采样，补长局内存、持续负载、冷启动。
4. 旧线采同负载；新线跨目标执行同脚本并定位首个差异。只有本机 writer/reader 不足以关闭 N12。
5. 原生输入到帧提交、GPU、正式平台和真机依原稿另轨；范围调整与技术通过分别记录。
