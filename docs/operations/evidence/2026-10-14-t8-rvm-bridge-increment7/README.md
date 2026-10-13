# T8 增量 7：跨进程恢复（同宿主双进程存档接力，2026-10-14）

## 任务来源

[T8 增量 6](../2026-10-14-t8-rvm-bridge-increment6/README.md) 登记的「跨进程恢复」——C1 验收最后一项测试面（内核侧单进程往返/挂起中存档已在增量 1–2 实测）。

## 交付物

- **`t8_vm_skill_cross_process_resume`**（自启子进程接力）：
  1. 父进程建局→授技→推进到 VM 选择挂起（rvm_choice）→NGSA 字节落盘；
  2. **子进程**（`current_exe` 自启 + `T8_WORKER_ARCHIVE/T8_WORKER_REPLY` 环境变量识别）：读档→`decode_archive`→`restore_with_extensions`→挂起态在位→应答 Tag(1)→续跑落账→存档回写；
  3. 父进程解码回写存档→恢复→断言挂起已消费、hp 2→3。
- 连带：`KNOWN_PROMPTS` 注册 `rvm_choice`（存档 prompt 固定注册表；沿用 buchun_target 等先例——纯解码侧表项追加，非格式递增面，旧档不受影响）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | **15/15** 绿（跨进程 1 + 增量 1–6 全部回归）——[日志](t8-bridge-targeted.log) |
| 全量 | **670/0/0**（669→670）——[日志](engine-native-tests.log) |
| Gate 1b | 独立 target 目录 `--all-targets` **exit 0**——[日志](gate1b-all-targets.log)（首跑抓出 1 条 unused import，已清） |

## 坑位实录

- 存档 prompt 固定注册表：新挂起提示词（rvm_choice）解码 `BadTag("prompt_unknown")`——须入 `KNOWN_PROMPTS`（纯追加，非递增面）。
- 子进程自启模式：`current_exe` + 环境变量识别（worker_paths），测试内先查 env 再走父路径。

## 登记边界（T8 收敛状态）

- **C1 验收面在本线内核侧已全部实测**：数据驱动安装（增量 1/3/4）、语义等价（增量 1）、选择挂起/恢复（增量 2/7 挂起中存档+跨进程接力）、内容锁（增量 2 漂移拒绝+快照哈希）、超限回滚（增量 1/5 矩阵）。
- 余项（按需立项）：具名组合 filter 的 engine 侧承载、剩余效果/查询变体映射（部分需内核新原语：ChangeMaxHp/AddSkill/marks 等）、`HostContext.skill_id` 反查。
