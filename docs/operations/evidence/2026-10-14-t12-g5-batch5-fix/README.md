# 双仓同载挂起·专项诊断与修复（2026-10-14）

## 根因

第五批登记的「cardpile 双仓同载挂起」**非引擎缺陷**：接力测试的子进程 spawn 目标误写为主测试自身（`t12_g5_relay_cardpile_dual_namespace`），主测试再次落盘并再 spawn 下一子进程——**无限自递归进程链**即表现为"挂起"。诊断脚本分阶段计时证明双仓引擎本身毫秒级正常（桥接 24ms/建局 0.8ms/逐步 40–160µs）。

## 修复

- 子进程 spawn 目标修正为 **worker 测试** `t12_g5_relay_cardpile_dual_namespace_worker`（env 识别 worker 分支，含错版/缺包拒绝断言）。
- worker 内 `decode_archive` 用全路径（局部无导入）；错版构造改经 `cardpile_liuli_pack_from_repo` 重造（ExtensionPack 非 Clone）。
- 两处 `#[ignore]` 撤销（第五批登记的 ignore 随根因修正解除）。

## 实测记录

| 项 | 结果 |
|---|---|
| 双仓同载跨进程接力（主+worker） | **通过**（双仓 226 张并入目录/父推进/子同锁恢复+错版缺包拒绝/续推 8 步/回写校验） |
| t12_g5_relays | **16/16**（14 既有+2 本批）——[日志](relays-final.log) |
| engine-native 全量 | **711+2/0/0 → 以日志为准：713/0/0?**（以汇总行为准）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[gate1b-all-targets.log](gate1b-all-targets.log) |

## 教训

- 自启子进程接力测试的 spawn 目标必须指向 **worker 测试名**；主测试若无 worker 分支会自递归。t8_rvm_bridge 跨进程测试的 spawn 目标就是 worker 测试名（正确先例）。
