# ADR 0758：N08 官方迁移第五十九批——刚烈/突袭/裸衣

日期：2026-09-22（第五十九批）
状态：已采纳
上游：[0755](0755-native-n11-faction-kof-m2.md) / [0756](0756-native-guozhan-m3-faction-victory.md) / [0757](0757-native-guozhan-reveal-gating.md)

## 背景

N08 官方武将迁移按既有模板（五类触发 + 主动技 + view-as 声明牌种齐备）
继续扩面：标准包魏国武将夏侯惇（刚烈）、张辽（突袭）、许褚（裸衣）。
三者分别覆盖新原语组合：受伤后判定反伤、摸牌阶段改写（弃固定摸牌换取
他人手牌）、摸牌阶段减量与本回合伤害加成。

## 决策

1. **刚烈（夏侯惇，可选触发）**：`EventKind::Damage`（target=owner、
   非防止、source 存在且 ≠ owner）→ 确认闸门 → `draw_judge_card` 判定
   → 判定牌弃置 → 非红桃则对伤害来源发出 1 点普通伤害（递归由确认
   闸门天然限流；自伤环路已在 filter 排除）。
2. **突袭（张辽，可选触发，每回合限一次）**：`PhaseBegin(Draw)` → 确认
   → `tuxi_target` 询问（SelectPlayers，1–2 名有手牌的其他角色，可弃权）
   → 弃权 = 不发动并保留摸牌；确认目标 = 弃置本阶段固定摸到的牌
   （手牌末尾至多 2 张——触发机器在阶段固定摸牌之后执行，末尾即刚摸
   的牌）并按各区首张确定性取牌（Dismantle 同款约定）获得目标各一张
   手牌。存档续行经 `active_ask` 的 TuXi arm 校验。
3. **裸衣（许褚，可选触发，每回合限一次）**：`PhaseBegin(Draw)` → 确认
   → 弃置本阶段固定摸到的最后一张（少摸一张）；伤害加成旗标**复用
   `once_used`**（确认时由 OptionalGate 置位，下回合判定阶段开始时随
   `reset_turn_limits` 清除），不新增 TurnState 字段。kernel 新增
   `luoyi_bonus(ctx, source)`：杀结算（无闪路径/出闪后放弃路径两处）
   与决斗（询问失败路径/无杀直接受伤路径两处）伤害 `1 + bonus`；
   南蛮/万箭不享受加成（官方语义）。
4. **版本与 golden**：技能表扩面 = 目录语义变化，`STANDARD_RULESET_VERSION`
   20 → 21，14 golden 全量重录。

## 验收

- `xiahoudun_ganglie_non_heart_reflects`：黑桃判定 → 来源 -1 体力；
  红桃判定 → 无反伤、受伤者仅承受原始伤害。
- `zhangliao_tuxi_replaces_draw`：取两名（弃 2 摸牌换偷 2 张、目标各
  -1、守恒）；询问挂起时存档→恢复续行终态哈希一致；弃权保留摸牌。
- `xuchu_luoyi_boosts_sha_damage`：裸衣后杀 2 点（放弃出闪路径）；
  决斗 2 点致死（无杀直接受伤路径）；守恒。
- 全量 **197 passed / 0 failed**（native_kernel 120 + boundary_snapshot 59
  + view_isolation 7 + auth_archive 6 + replay_scenarios 2 + src 2 + dbg4 1）。

## 后果

- 官方迁移武将增至 22 名；标准包剩余：刘备（仁德+激将主公技）、
  诸葛亮（观星+空城）、马超（马术+铁骑）、陆逊（谦逊+连营）。
- 裸衣加成旗标复用 `once_used` 避免状态扩面；若未来需要「本回合多次
  发动」类语义再引入显式旗标。
- 既有对局行为不变：金样场景无新技能持有者，重录仅反映版本号入摘要。
