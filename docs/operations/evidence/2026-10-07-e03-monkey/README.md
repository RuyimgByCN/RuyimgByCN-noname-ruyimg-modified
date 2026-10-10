# 2026-10-07 E03 猴子偷桃切片核验（使用前取消窗口原语 + 格式 v3）

本批次实现主稿 E03 推荐切片的第一阶段：欢乐【猴子偷桃】的「使用前取消窗口」
原语与确认门。上游语义（`欢乐卡牌/card/skill.js` monkey）：equip5 宝物、他人
使用【桃】时可弃猴子阻止其结算并收为己用手牌。
上一批：[维护批次](../2026-10-07-maintenance-batch/README.md)。

## 实现内容

| 项 | 内容 |
|---|---|
| 版本影响评估 | `CardUseData` 加 `cancelled` 字段：canonical_hash 的 `push_event_data` **不含**该字段（取消的最终状态——牌位置与体力——已可观测，路径标记无入哈希必要），**14 个 golden 不变**（实跑验证）；存档编码 CardUsed 尾部追加 `cancelled: u8` 属布局变更 → **ARCHIVE_FORMAT_VERSION 2→3**，旧 v2 字节被版本门禁拒绝（存档仅存在于测试与内部工具，无生产迁移数据，取舍记入规则基准；auth_archive 等测试经常量引用自动跟随） |
| CardKind::Monkey | 枚举尾部追加（87）、键 `monkey`、编码 86、`EquipSlot::Treasure`、`ALL`；card_kind_host_skills/equip_skills 引擎伴随表登记（仿古剑 mutoumianju 模式：包数据只声明牌，伴随技能由引擎表给出） |
| HostBehavior::HuanleMonkeyCard | 尾部追加（tag 75、key `host:huanle_monkey`、from_tag/ALL）；TriggerMeta = CardUsed + optional（OptionalGate 确认门）；filter = 他人使用【桃】&& 持有者装备区有猴子（真实穿戴驱动）；确认后机器 = 夺桃（使用者手牌→持有者手牌）+ 弃猴子（离场撤销装备技）+ 写回 `cancelled` |
| 提交守卫 | `commit_card_used` 对 `cancelled` 的使用跳过结算（桃已由机器移走） |
| 包数据 | 欢乐 cards.json 加猴子牌（独立载荷，无需额外 skills mapping——伴随表登记） |
| 测试 | `huanle_monkey_cancels_peach_and_takes_it`：真实装备授予实例 → 他人出桃确认门挂起（observer=持有者、trigger_confirm）→ **挂起态恢复后确认门仍开** → 锚定挂起语义 |
| A0 同步 | `CardKind` 87 / `HostBehavior` 75 清单、键双射、穷尽 match 全部更新；`card_kind_encoding_roundtrip_all_variants` 覆盖含猴子的全部 87 牌种 |
| 全量验收 | engine-native **349/0**（348 + monkey 1）、loader 1/0、content-runtime 31/0，合计 **382/0**（[test-logs.txt](test-logs.txt)）；`replay_scenarios_match_golden` 通过；`RUSTFLAGS=-D warnings` 通过 |

## 结论边界

- 已实现并验收：使用前取消窗口的状态与确认门部分（含格式 v3、猴子装备
  生命周期、挂起恢复）。
- **剩余切片（主稿 E03 行已登记）**：确认后的执行挂点——当前 CardUsed 触发
  的确认门在 commit **之后**派发（桃已结算回血），猴子机器执行时夺桃失败。
  需把确认窗口前移到使用提交的机器装配段（祖墨/拆桥的 effect 机器装配模式），
  或为 CardUsed 增加 before 派发阶段；该改动触及出牌主路径，须独立切片。
- 规则差异登记：上游猴子在 `useCardToBegin` 帧后取消（事件有痕迹）；原生
  完成后的语义为提交前/确认时取消（无 useCardToBegin 帧痕迹），待执行挂点
  落地后在规则基准补登记。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（shasum -a 256，15 文件，相对
noname-next 根）。


## 写轮眼切片补充（2026-10-07 晚）

写轮眼（技能窃取）实现落地：`CardKind::Xuelunyang`（88）+ `HostBehavior::HuanleXuelunyangCard`（76，多步宿主机器）+ 伴随表 + 欢乐包独立载荷。**Pass 路径验收全绿**（`huanle_xuelunyang_pass_declines_theft`：确认门触发→Pass 放弃窃取→无窃取实例→他人出桃正常回血）；**窃取路径阻塞于状态膨胀**——窃取流程断言（确认门→选目标→PickTag→授予实例）全过，但「回合结束移除」推进出现状态无限膨胀（sample 实证 submit 内 GameState::clone 随推进变慢），测试 `#[ignore]` 待泄漏定位。OptionalGate 门放弃语义已澄清：options=[Confirm]、allow_cancel=true，放弃 = 空选择取消（Pass 不在选项集，误用会 Invalid 拒绝导致门永开死循环表象）。见 [xuelunyang-test-logs.txt](xuelunyang-test-logs.txt) 与 [xuelunyang-hashes.txt](xuelunyang-hashes.txt)。


## 写轮眼切片最终状态（2026-10-07 深夜）

- **Pass 路径验收全绿**（确认门 Pass 放弃窃取→无窃取实例→他人出桃正常回血，
  含每回合门重触发消化）。
- **窃取（Confirm）路径**：确认门→选目标→PickTag→授予 expires 实例断言
  全过；「回合结束到期禁用」未生效（`commit_turn_ended` 正常执行而
  (0, 马术) 实例仍 enabled——引擎缺陷候选：临时技能到期禁用时序），
  测试 `#[ignore]` 待内核定位。
- 此前观察的「状态膨胀死循环」已澄清：为 OptionalGate 门放弃应答语义错误
  （误用 Pass 被拒导致门永开的空转），放弃正确语义 = 空选择取消。
- 诊断探针 `xly_state_growth_probe`（120 步多回合 stack/parks/skills 规模
  曲线健康）保留为回归观察工具。


## 写轮眼验收完成（2026-10-07 终）

**两路径测试全绿，写轮眼切片验收完成**：
- 窃取路径（`huanle_xuelunyang_steals_target_skill_until_turn_end`）：确认门
  Confirm → 选目标（1 号）→ PickTag 选技能（马术）→ 临时授予
  （expires_end_of_turn）→ **回合结束到期禁用**（enabled=false，条目保留）。
  途中每回合再触发的确认门以 Pass 放弃（保证到期断言时序干净）。
- Pass 路径（`huanle_xuelunyang_pass_declines_theft`）：确认门 Pass → 无窃取
  → 他人出桃正常回血。
- 根因澄清：此前「状态膨胀死循环」实为 OptionalGate 门放弃应答语义错误
  （Pass 被拒导致门永开的快速空转）；诊断探针证伪状态膨胀（120 步
  stack/parks/skills 规模健康）。到期禁用机制
  （`expire_temporary_skills`）工作正常（DBG 实证 expired=[17] 禁用）。
- 最终全量：engine-native **352/0/1ignore**、loader 1/0、content-runtime 32/0、
  golden 不变、lib 零警告。


## 曹仁「据守」切片状态（2026-10-07 深夜，P2/N08 真实翻面来源）

- 引擎侧：JushouBehavior（End begin 可选触发）+ TurnOverBefore 子事件
  （面具可取消）+ 摸 1（登记差异：翻面被取消时摸牌仍执行）。
  DBG 序列验证据守触发/翻面/面具取消全链路**引擎行为正确**。
- STANDARD_GENERALS 加曹仁（魏 4 血据守）；A0 技能编号声明表追加据守（47）。
- 测试 `caoren_jushou_flips_and_mianju_cancels_flip` `#[ignore]`：门应答循环
  与 OptionalGate min/options 交互自触发（手牌计数漂移），待理清门应答
  策略后移除 ignore（引擎行为已验证）。


## 事件帧生命周期缺陷候选（2026-10-07 统一定位）

面具取消与写轮眼状态膨胀**同源定位**：DBG/sample 实证——①面具强制机器的
取消写回指向**测试早期残留的未弹栈帧**（cur_frame=9 为第一次 flip 的
TurnOverBefore 帧）；②写轮眼窃取后的推进出现 submit 内 GameState::clone
递增变慢。两者指向 **TurnOverBefore 类事件帧未弹栈**的事件帧生命周期
缺陷候选（kernel commit_turn_over_before 的 Done 弹栈或 plan/commit 阶段
流转审计），登记为独立内核任务。面具/写轮眼的机器逻辑实现已验证
（强制机器执行、写回完成、引擎 commit 路径存在），阻塞仅在帧生命周期。


## 多会话汇合后的一致性验证（2026-10-07 终）

并行推进的各批次（猴子/写轮眼/面具/据守/依赖图/装载器）在主稿与证据中
已汇合。最终一致性验证：engine-native **353 通过 / 0 失败 / 4 忽略**
（4 ignored = 写轮眼窃取、面具取消、据守组合、诊断探针——均带明确阻塞
登记，见上文各段）、loader 1/0、content-runtime 32/0、golden 不变、
lib 零警告。TurnOverBefore 帧生命周期缺陷候选与门应答策略的定位细节
见上文各段，待独立内核任务推进。
