# 2026-10-10 T1：host.rs 事故恢复空分支收尾

对应主稿待做 **T1**。核验日期 2026-10-10，源码基线本地 git `f0bd0af9`（产品源码与 `72583b2b` 相同，此前只改文档），规则 v192、NGSA 存档格式 v41。

## 结论

`crates/engine-native-content/src/content/host.rs` 里 14 个 `TODO(事故恢复)` 空分支（`let _ = ctx; Flow::Done`）**不是丢失的实现**：逐项对照上游定义与规则基准后确认，效果都由内核在固定位置按技能键内联，宿主机器本就没有分段。本轮把 14 处占位注释改写为按上游语义的有据 no-op，并补上元宝肉缺失的上限负例。**没有改任何行为代码**。

## 逐项对照

| 行为 | 上游定义 | 原生效果位置 | 结论 |
|---|---|---|---|
| MoshouImmunity 墨守 | 玩点论杀/费诗，仅 `mod.targetEnabled` | `host.rs` `blocks_delayed_trick` + 延迟锦囊目标过滤 | 上游无 trigger，宿主无步骤 |
| GujianFoodYuanbaorou 元宝肉 | 古剑，`cardUsable num+1` + mark 回合递减 | `engine.rs` 2339/2501/3329 | 效果内联；补上限负例 |
| WudianCiqiu 刺酋 | 玩点论杀/韩龙，`damageBegin1` forced | `kernel.rs` 142（+濒死 3568） | 效果内联 |
| WudianChezhen 车阵 | 玩点论杀/田豫，仅 `mod.globalFrom/globalTo` | `state.rs` 700 | 上游无 trigger，宿主无步骤 |
| WudianTuji 突击 | ow/士兵76，仅 `mod.globalFrom` | `state.rs` 663 | 上游无 trigger，宿主无步骤 |
| WudianGuangshuSpade 光塔·♠ | ow，`guangshu_spade` trigger `phaseEnd` | `kernel.rs` 768（阶段结束直推） | 效果内联；TriggerMeta 仍注册 PhaseEnd 供过滤 |
| WudianDuwen 毒吻 | ow/黑白合，`damageBegin` forced | `kernel.rs` 167 | 效果内联 |
| WudianFeiren 飞刃 | ow/源氏，`damageBegin` + `mod.targetInRange/selectTarget` | `kernel.rs` 121；`engine.rs` 2377/2447 | 效果内联 |
| WudianXiandan 霰弹 | ow/死神，`shaBegin direct` + `xiandan2` | `kernel.rs` 3152（询问）、2222（黑牌加伤） | 效果内联 |
| WudianJijia 机甲 | ow/D.Va，`changeHp` 拦截 + `mod.maxHandcard` | `kernel.rs` 197、2006 | 效果内联 |
| WudianLongxiang 龙翔 | xianjian/温慧，`shaBegin`（有 check） | `kernel.rs` 3047（内联不询问，v146 已登记） | 效果内联 |
| XianjianTannang 探囊 | xianjian/韩菱纱，`viewAs` + `mod.targetInRange` | `engine.rs` 2165、2623；usable 走命令层 once_used | 效果内联 |
| XianjianXuanyan 玄炎 | xianjian/玄霄，仅 `group:[xuanyan2,xuanyan3]` | 子技在 `host.rs` 10930/10939 | 壳分支无步骤 |
| WudianLueying 掠影 | xianjian/慕长兰，`shaBegin usable:1`（可选） | `kernel.rs` 3087 | 效果内联；上游差异登记见 T11 |

事故前会话回放版本 `~/host-recovery-20261010/replayed_pre_incident_16128.rs` 里，刺酋、元宝肉与其它被动状态行为合并为同一个 `=> Flow::Done`，墨守也是 `=> Flow::Done`；其余 11 项在该版本之后才加入。可佐证“无机器步骤”是原有设计。注意 `host.rs.pre_assembly` / `host.rs.recovered-v192` 仍有 76 个 TODO，是恢复中间产物，不含未恢复实现。

## 补充的测试内容

`crates/engine-native/tests/g1_shahai_gujian.rs::food_effects_batch_two` 的元宝肉段原先只断言“额外一张杀放行”，且 `def_ybr` 取出未用。本轮：

1. 断言元宝肉状态授予（`skills` 含 `(0, def_ybr)`），消除未用绑定；
2. 放行第一张额外杀后，再用第三张杀，断言返回 `RejectReason::ShaLimit`（每回合仅一张额外杀）且不额外摸牌。

## 验证

```
cargo test --offline -p engine-native --no-fail-fast
```

结果：**525 passed / 0 failed / 1 ignored**（24 个测试目标），`replay_scenarios_match_golden` 与 `scenarios_are_registered_in_coverage_manifest` 通过，14 个 golden 未变。完整日志见 [engine-native-tests.log](engine-native-tests.log)。

改动仅 `host.rs` 注释（62 插入 / 30 删除）与上述测试；无规则版本或存档格式变化。
