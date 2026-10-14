# T12 运筹武将面批 3：纵火 + 烧营（diy_luxun 整将转正，2026-10-14）

## 任务来源

[T12 运筹武将面](../../migration/upstream-extension-reconstruction.md)批 2 续批：
diy_luxun（陆逊，吴 3 血）两技全部转正——**第二个整将完成**。

## 上游语义与承载

| 技能 | 上游 | 原生承载 |
|---|---|---|
| 纵火 zonghuo（可选，priority 10） | 你造成非火属性伤害时弃 1 张手牌将该伤害改为火属性 | `HostBehavior::YunchouZonghuo`（tag 245）：**DamageBefore 触发**（nature 写回须在 commit 前）+ 帧直写 `dm.nature=Fire`（乱剑增伤 amount 直写同机制）+ 弃首张手牌（登记差异：上游玩家自选） |
| 烧营 shaoying（可选，direct） | 你造成火属性伤害后弃牌堆顶牌亮出，红色则受伤者距离 1 内一名其他角色受 1 点火伤 | `HostBehavior::YunchouShaoying`（tag 246）：Damage 触发 + 座位序首合法目标（登记差异：上游玩家选目标——水攻先例）+ `deck.pop()` 牌堆顶（摸牌同端）+ 红色则 `damage_data_with` 蔓延 1 点火伤；**链式再触发与上游一致不挡**，堆空自然终止 |

nature 写回面勘察结论：`ctx.res.frame_mut(ctx.current_frame_id())` 直写
`EventData::Damage` 载荷在 DamageBefore 窗生效（commit 读最终帧载荷）——乱剑
增伤（v187）已验证同机制，本批扩展到 nature 字段。

## 测试设计要点（回归驱动）

- **纵火火性证明用铁索传导**：`LogEvent::DamageDealt` 无 nature 字段，改为
  p1/p2 预置横置——火伤经铁索传导 p2 掉血即火性证明；无手牌负例不传导。
- **烧营链式受控**：蔓延火伤会再触发烧营（上游同语义）——牌堆排空
  （`debug_drain_deck_to_discard`）后压受控顶，链在堆空自然终止。
- **负例堆顶按谓词压黑桃**：闪有红花色变体，按 kind 挑选不可控
  （`debug_set_deck_top_by`）。

## 实测记录

| 项 | 结果 |
|---|---|
| 反例实证 | Zonghuo/Shaoying filter 桩化恒 false → 批 3 测试 **FAILED**（0/1） |
| 定向 | g1_shahai_yunchou **16/16** |
| engine-native 全量 | **730/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| 装载/包管理/服务端 | **84/0** |
| Gate 1b | exit 0（拦截 2 处 `let mut` 冗余绑定，已修）——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 运筹武将面进度 **4/10 名、6/17 技**（陆逊整将；暂缓仍 2：奸雄·变体/护驾）。
- 批 4 候选：diy_yuji（diyguhuo+diychanyuan）与 diy_caiwenji/diy_lukang/
  diy_zhenji/old_majun 逐名勘察。
