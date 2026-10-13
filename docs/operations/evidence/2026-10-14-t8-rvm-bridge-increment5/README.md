# T8 增量 5：恶意字节码拒收矩阵（2026-10-14）

## 任务来源

[T8 增量 4](../2026-10-14-t8-rvm-bridge-increment4/README.md) 登记的「恶意字节码拒收矩阵」——C1 验收安全面：对抗样本全部经**数据路径**注入（DSL/载体字节篡改），断言「无效果落账、对局不 panic、机器回滚收尾」。

## 矩阵（t8_rvm_bridge.rs，增 4 测）

| 向量 | 注入方式 | 防线 | 结果 |
|---|---|---|---|
| ① 未映射效果 | DSL `effect change_max_hp`（tag5，桥映射面之外） | `EngineVmHost::emit_effect` InvalidRequest → 模块 trap → 回滚 | 无落账、max_hp 不动——`t8_rejection_unmapped_effect_rolls_back` |
| ② 字节码载体篡改 | 编译模块 `code[0] ^= 0xFF` 后入包 | `verify_module`（execute 首道闸）→ Err → 回滚 | 无落账——`t8_rejection_corrupted_bytecode_rolls_back` |
| ③ 能力越权 | `caps_mask \|= 0xFFFF_0000`（未知能力位） | `check_caps`（execute 第二道闸）→ CapabilityDenied → 回滚 | 无落账——`t8_rejection_caps_escalation_denied` |
| ④ 存档篡改 | NGSA 载体字节中部翻转 | 魔数/校验和 → decode 拒绝（Rvm 载体字节的针尖级篡改由 check_machine `from_bytes`+哈希核对承担，auth_archive 负例族同口径） | 解码拒绝——`t8_rejection_tampered_snapshot_module_rejected` |
| （既有）fuel 耗尽 | 无限循环 DSL | DEFAULT_FUEL 截断 → 回滚 | 增量 1 `t8_vm_skill_fuel_exhaustion_rolls_back` 保持绿 |
| （既有）continuation 内容锁 | 包版本漂移 / 换模块续跑 | 恢复指纹核对 + `resume` 哈希核对 | 增量 2 两测保持绿 |

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | **11/11** 绿（矩阵 4 + 既有 7）——[日志](t8-bridge-targeted.log) |
| 全量 | **666/0/0**（662→666）——[日志](engine-native-tests.log) |
| Gate 1b | 独立 target 目录 `--all-targets` **exit 0**——[日志](gate1b-all-targets.log)（首跑抓出 1 条 unused mut，已清） |

## 登记边界（后续增量）

- DSL filter 表达式映射（现恒 owner 域）；`HostContext.skill_id` 反查；剩余效果/查询变体映射（MoveCards/UseCard/AddSkill/marks/RequestChoice、TargetLegal/CardUsable/HandLimit 等）。
- 跨进程恢复（跨进程=同宿主双进程存档接力；内核侧单进程往返已实测）。
- 栈溢出/heap 上限向量经 fuel 面等价覆盖（DSL 无无界递归构造；手工字节级栈攻击由 verify 的 max_stack 校验承担，未单列针尖测试——登记）。
