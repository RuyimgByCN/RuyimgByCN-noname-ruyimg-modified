# 第 0 项独立静态复核首轮

- 日期：2026-11-13；复核人：`queue0_independent_review`；HEAD `3a6e398485db6ef0729abe3013c3c4b7705a5d6d`，tracked dirty；规则 333／NGSA 107／schema 5／ABI 759／WS8。
- 范围：Fz 选序登记、正式安装 U01 的技能构成断言、提示词守卫测试及暗织修复。只读上游、原生源码、实际 diff 和测试；未读本轮实现者说明或证据 README。本轮尚未运行 Cargo，依主 agent 要求等待生产修改稳定。
- 本文保留首轮发现，不代表最终生产版本；当时 diff 与文件哈希见 [static-source-round1.txt](static-source-round1.txt)。

## 三项小修复

1. Fz 按 `(Normal, 1, false)` 补登记及同步 C331 守卫不会改变既有 tuple；上游仍为 `(Normal, 0, true)`，详情见 [边界复核](fz-selection-boundary.md)。不得据本次登记提升为等价。
2. 独立将 `decision34_pack9_data.rs` 的 U01 64 名集合，与上游 `3D精选/character/skill.js` 的 64 个顶层 `ddd*` 技能键比较，双向差集均为空。真实装载优先 `lookup_upstream_host_behavior`，缺少绑定时才落 VM；更新安装路径测试为完整 64 名 host 集合与 0 VM 符合预期。此项只证明名称构成及安装路线，未审查全部 64 技能规则等价。
3. 提示词测试首版新正则 `^    "([^"]+)",` 只取一行中的第一个登记值。即使合并生产 `prompt:` 字面量扫描，仍漏 206 个唯一登记值，例如 `dddyeshen_confirm`、`dddlianer_confirm`、`dddguiying_confirm`。向主 agent 报告后，新版改为扫描所有专用 `known_prompts*.rs` 中全部逗号结尾字符串，并断言非空；独立对数组内容解析确认 6 个模块共 3,136 个唯一值，遗漏 0。

第 2、3 项静态对照可复跑 [static-comparison.py](static-comparison.py)，结果见 [static-comparison.json](static-comparison.json)。独立执行 `node --import ./scripts/ts-register.mjs --test scripts/approved-b-metadata.test.mjs`，7 passed／0 failed／0 skipped；原始输出见 [approved-b-metadata-test.raw.txt](approved-b-metadata-test.raw.txt)。

## 暗织首轮退回点

已逐项反馈主 agent，需在最终实际源码与执行结果中复核，不把本条消息当作修复证据：

| 发现 | 具体反例与源码依据 |
| --- | --- |
| 无暗织时也改变五谷／桃园目标语义 | `kernel/tricks.rs` 新 `retain` 只判断当前用牌 targets 非空。既有 `rvm_ops/commit_ext.rs` 的 `UseVirtualCardOn`／`UseVirtualCardOnSelectedTargets` 可发非空子集的虚拟五谷或桃园；旧机器作用全部存活／其他角色，新机器在无暗织时也缩为子集。需保留无暗织旧行为回归。 |
| 装备末牌色及黑装备免疫遗漏 | `engine/play_card.rs` 的装备分支直接发 `Equipped`，绕过本次仅处理 `CardUsed`／`CardResponded` 的共同末牌历史与目标移除。上游 `game/index.js` 的装备默认 `toself`、`selectTarget=-1`，仍经 `useCardToTarget`。需要区分实际 PlayCard 使用和技能强制／直接装备。 |
| 旧 CardUsed 追色分支倒退嵌套历史 | 外层红用牌 Commit 记 false；先执行的合法扩展／VM触发发出黑用牌或黑打出子帧，记 true；随后外层 `u01_batch_d.rs` 的旧 `machine.card==Some(1)` 分支又以外层红牌回写 false。共同历史与旧按持有者写入同时存在，末牌不再是最后发生的事件。 |
| 免疫到期早于回合末窗口 | 上游 `player.js` 的 `addTempSkill` 默认到 global `phaseAfter`／`phaseBeforeStart`；当前 `VmAux::clear_turn` 新移除两暗织旗，而 `commit_turn_ended` 先调用 clear_turn，再发 `TurnEndWindow(after:false)`。因此 phaseEnd 中发生的黑用牌已无免疫。 |
| 目标移除早于追加目标 | 只在 CardUsed Commit 移除目标，后续 AfterTriggers 的既有 VM `ChooseAddCurrentShaTarget` 等可追加免疫目标，新增目标未再次检查。反方向也成立：原唯一目标被移出时已置 `intrinsic_done=true`，后续追加未受免疫目标仍略过整牌固有效果。需覆盖两种方向。 |

五谷挂起的新增 `machine_held_cards` 分支将已离开牌堆、只在 `Wugu.cards` 中的亮牌列入归属检查；既有 `NullifyWindow` 递归会覆盖包裹态。此分支本身不新增序列化字段；仍须运行普通无暗织五谷与黑色转化五谷的恢复正例，并核对机器恢复分派。

本轮未启动上游浏览器，也没有上游实际触发记录；以上是源码复核。暗织触发仍使用 Prepare 的 PhaseBegin，命名元数据仍为 priority 1／不可选序，与上游真回合开始及 priority 0／可选序不同；这两项保留到下一队列，不应升为「已复核等价」。
