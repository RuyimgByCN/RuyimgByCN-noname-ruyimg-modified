# R05-F 批：E 未尽项 3–7 kernel 回调接线（2026-11-03）

三提交：`112da572`（wave1 到期/死亡/上限）、`1a4a06cd`＋子代理 `106a2ab1`（wave2 Combs Begin2 消费＋两 baoshu 测试按新生命周期和解）、wave3（Next enabled 精确门＋首写装实例，随本报告提交）。最终三 crate **1936/1/75**（logs/f-03.log；唯余 replay_scenarios golden，失败集与基线一致）。

| 未尽项 | 实现 | 红绿 |
| --- | --- | --- |
| 3a Limit/Current 到期 | Catalog 新 `r05_role_defs` 回调（内容按 `#r05:{role}` 模板识别；标准目录空）＋phaseAfter 段（收其他 trigger 前）清全部角色两槽与 helper（含 disabled；Next/Combs 保留、不二扫） | r05f_limit_current_expire（真红于回调缺失→绿） |
| 3b Combs Begin2 消费 | D 机 Begin2（非 fixed）stored>0 → pending＋全部正值、槽清、helper 移除；≤0 不动 | r05f_combs_consumes（正消费/负保留双面）；两 E baoshu 测试经原作者会话核对为**实现正确、测试未预设生命周期**，轮序对调后更严（负条目跨两轮保留兼证新语义） |
| 3c 死亡清理 | DeathMachine step 前置＋turn_handoff 兜底双位，清 Limit/Combs/Current；Next 条目/值/实例/enabled 原样保留 | r05f_death_clears（Next 值＋实例双保留断言） |
| 5 手牌上限 | 弃牌上限在基数/机甲/蛊舞后加全部**启用** Limit 有符号值 clamp 0（修 usize::MAX as i64=-1 panic） | r05f_hand_limit（-3 → 4-3=1） |
| 7 Next enabled 精确门 | take 门＝enabled 实例存在（§6）；窗口面首写 Next 即装目标实例 | r05f_next_take（真实伤害窗写入→disable 不 take 条目保留→重启用 take 转 Current 含 0） |

未尽项 6（主 draw>0 时 End 窗被 R 子帧替代、涕泣装载面取 After 窗）为既有装载面设计选择，随 J 场景批核。
