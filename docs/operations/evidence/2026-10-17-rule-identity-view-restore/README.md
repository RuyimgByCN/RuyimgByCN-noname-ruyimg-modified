# 2026-10-17 身份可见视图与身份局恢复（RULE-IDENTITY-VIEW-RESTORE）

**核验日期**：2026-10-17。**待做 ID**：`RULE-IDENTITY-VIEW-RESTORE`（规则基准「当前声明『身份』验证边界」）。
**性质**：视图投影 + 测试（无状态/版本/golden 变化；规则 v295 / 存档 v80）。

## 结论：身份可见视图补齐 + 身份局逐提交恢复一致

1. **完整身份可见视图**（新）：`PlayerCellView` 新增 `role: Option<Role>`（`crates/engine-native/src/view.rs`），由 `Engine::build_view` 纯投影导出（`engine.rs`）：
   - 非身份局恒 `None`（不改其它模式视图）。
   - 身份局：**本人**（自己身份可见）、**主公**（身份公开）、**已死亡**（上游 `dieAfter` 遗言公开身份）三者可见；其余**存活非主公**成员的他人身份保持 `None`（信息隔离）。
   - 纯函数导出，**不入状态/快照/哈希**——不引入版本次，golden 不变。
2. **身份局逐提交恢复**：`non_prototype_script_parity`（[T11 事件/RNG/恢复扩面](../2026-10-17-t11-event-rng-restore-parity/README.md)）对身份 5 人局在每个命令点 save→restore 后跑完剩余脚本、终态哈希与基准一致；本批另加「死亡公开后逐座位视图在恢复件上一致」断言。

## 测试

新增 `identity_role_visibility_and_death_reveal`（`crates/engine-native/tests/t13_info_isolation.rs`）：
- 5 人身份局：主公对所有人公开；本人可见自己身份；其余存活非主公他人身份为 `None`。
- `debug_deal_damage` 击杀 1 号后，其身份对所有存活座位公开。
- save→restore 后逐座位视图（含身份可见性）完全一致。

命令：`cargo test --offline -p engine-native --test t13_info_isolation`，结果 **11 passed / 0 failed**（[targeted.log](targeted.log)）。engine-native 全量 **876/0/0**、Gate 1b exit 0。

## 边界

- 身份**可见性规则**采用「本人/主公/死亡公开，其余隐藏」，与上游 `dieAfter` 遗言公开及主公明置一致；未建模上游「主动 `showIdentity`」技能面（当前技能集不依赖），有内容需求时另批（涉及状态字段与版本次）。
- 「所有终局逐提交恢复」在身份局已由逐命令恢复对照覆盖；其余模式与扩展长局的终局恢复按各自批次验收。
