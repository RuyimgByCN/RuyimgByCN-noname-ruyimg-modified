# 2026-10-17 T11 横切：事件/随机流/恢复轨迹对照扩面（T11-EVENT-RNG-RESTORE-PARITY）

**核验日期**：2026-10-17。**待做 ID**：`T11-EVENT-RNG-RESTORE-PARITY`（进度主稿 T11 完成条件末段；[T11 开放行清单](../2026-10-16-t11-open-rows/README.md) 记缺口为「扩场景夹具 + RNG 序断言 + 非 Prototype 模式中间恢复」）。
**性质**：测试扩面（无规则语义/版本/golden 变化；规则 v295 / 存档 v80）。

## 已覆盖（本批补齐所列三缺口）

新增 `identity_versus_boss_replay_restore_parity`（`crates/engine-native/tests/native_kernel.rs`）+ 通用夹具 `non_prototype_script_parity`，对**身份 5 人局 / 对决 2v2 / Boss 关卡**三种非 Prototype 模式各跑：

1. **扩场景夹具**：由 `next_script_command` 生成同一确定性脚本（≥20 命令，最多 80 直至终局）。
2. **RNG 序断言**：`canonical_hash` 含 `rng.state()`（`engine.rs`）与牌堆/弃堆/牌桌（`engine.rs`），故**状态哈希一致即随机流消费序与牌堆顺序一致**；重放线逐命令比对哈希 + 全量日志事件序列（事件轨迹对照）。
3. **非 Prototype 模式中间恢复**：恢复线在**每一个命令点** save→restore 后跑完剩余脚本，逐点断言恢复态哈希与基准一致、终态哈希与基准终态一致。

既有的 `replay_and_restore_per_command_parity`（Prototype）与 `restore_at_every_choice_point_continues_identically` 覆盖 Prototype 路径；本批把同口径扩到三个模式入口。

## 命令与结果

```
cargo test --offline -p engine-native --test native_kernel -- \
  identity_versus_boss_replay_restore_parity replay_and_restore_per_command_parity
```

结果：**2 passed / 0 failed**（[targeted.log](targeted.log)）。engine-native 全量与 Gate 1b 另记。

## 边界（登记）

- 覆盖**引擎模式入口**（Prototype/Identity/Versus/Boss）的逐命令重放/恢复；含扩展包内容（swd 等）的长局、跨进程接力与联机录像不属本项（分别见 T8 G5 切片、T9 钩挂、E05）。
- 不声明「所有上游技能组合的事件/RNG 逐项等价」——具体技能的事件序差异仍按覆盖清单行为差异表逐行验收。
