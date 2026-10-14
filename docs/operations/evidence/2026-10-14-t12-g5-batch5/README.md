# T12 G5 接力·第五批：cardpile 双仓同载——**发现挂起缺陷并登记**（2026-10-14）

## 任务来源

[第三/四批](../2026-10-14-t12-g5-batch3/README.md) 后的双 namespace 深化：cardpile 双仓**同载**（noname+liuli 两包并入同一引擎，226 张合计）的跨进程接力。

## 发现：双仓同载挂起（真缺陷）

- 父进程 `Engine::new_with_extensions(seed, 3, &[&pack_noname, &pack_liuli])`（226 张并入目录）→ 推进 6 步 → 落盘 → 自启子进程双锁恢复 → **>90s 无完成（挂起确认，单测直跑同样复现）**。
- 挂起点：父进程建局/推进或子进程双锁恢复（`--nocapture` 下无 panic 输出，纯挂起）。
- **对照**：单仓接力（第三批 noname、第四批 liuli）与双仓 G2–G4 通道均绿——挂起仅出现在**双仓同载**路径。
- **处置与后续（2026-10-14 当日闭环）**：ignore 登记后专项诊断完成——根因=子进程 spawn 目标误写为主测试自身（自递归进程链，非引擎缺陷）；修复后解除 ignore，见 [诊断修复证据](../2026-10-14-t12-g5-batch5-fix/README.md)。

## 实测记录

| 项 | 结果 |
|---|---|
| 双仓同载接力 | 挂起（>90s）——**登记缺陷，ignore** |
| 回退后套件 | t12_g5_relays 14/14（+2 ignored）；engine-native 全量 **711/0/0**——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[gate1b-all-targets.log](gate1b-all-targets.log) |

（全量数字以 [engine-native-tests.log](engine-native-tests.log) 汇总行为准。）

## 登记边界

- 双仓同载挂起为**新发现的内核/装载缺陷**（疑点：双 cardpile 同 package_id 不同 namespace 的目录合并或发牌路径），**专项诊断批待领取**——单仓路径全部绿排除大面积回归。
- 本批价值：缺陷发现与隔离（ignore+证据），不是功能达成；双 namespace 深化在缺陷修复后解锁。
