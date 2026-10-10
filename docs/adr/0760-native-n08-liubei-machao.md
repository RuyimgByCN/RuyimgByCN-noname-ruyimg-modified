# ADR 0760：N08 官方迁移第六十一批——刘备/马超（标准包收尾）

日期：2026-09-22（第六十一批）
状态：已采纳
上游：[0759](0759-native-n08-zhugeliang-luxun.md)

## 决策

1. **仁德（刘备，主动技，出牌阶段限一次）**：`activate` 形态（制衡同款）
   ——`rende_cards`（SelectCards，空应答 = 放弃）→ `rende_target`
   （SelectPlayers，弃权 = 不交付）→ 逐张 `give_hand_card`，给出 ≥2 张
   则 `recover` 1 点体力。
2. **激将（刘备，主公技主动化，出牌阶段限一次）**：`jijiang_helper`
   （候选 = 已声明蜀势力且手中有杀的其他角色——仅国战模式下存在候选，
   非国战 `activate` 返回 None）→ `jijiang_respond`（被请求者
   Confirm/Pass）→ 确认后其第一张杀置入刘备手牌。**原生简化**：官方
   「视为使用」以「获得后走正常出杀流程与次数校验」表达，未实现
   借牌直结算链，挂账覆盖清单。
3. **马术（马超，锁定门控）**：`computed_distance` 内查 `MA_SHU` 持有者
   （进攻方 a），距离修正 -1（既有 ±1 马下限 1 不变）。金样场景无持有
   者，既有对局行为不变。
4. **铁骑（马超，锁定式判定）**：`ShaResolveMachine` 新增
   `tieji_red: Option<bool>`——出闪询问构建前判定（红色 Heart/Diamond
   → 目标闪选项全滤除，走直接伤害路径）。六处同步：快照变体、存档
   enc/dec（0/1/2 三态）、状态哈希、restore、new()、结算逻辑。
   判定牌判定后即弃置。
5. **版本与 golden**：版本 22 → 23，14 golden 全量重录。

## 验收

- `liubei_rende_gives_cards_and_heals`：给 2 张回血 + 交付 + 每回合一次
  （SkillAlreadyUsed）+ 空应答放弃。
- `liubei_jijiang_borrows_sha_from_shu_helper`：蜀将确认交杀 / 拒绝不动 /
  无蜀候选 InvalidChoice。
- `machao_mashu_distance_minus_one`：4 人局隔座（距离 2）有马术可杀、
  无马术拒绝。
- `machao_tieji_judgement_blocks_shan_on_red`：红桃判定无出闪询问直接
  1 伤；黑桃判定正常出闪询问、放弃出闪 1 伤。
- 全量 **205 passed / 0 failed**（native_kernel 128 + boundary_snapshot 59
  + view_isolation 7 + auth_archive 6 + replay_scenarios 2 + src 2 + dbg4 1）。

## 后果

**标准包 25 武将全部迁移完成**（魏 7：曹操/司马懿/夏侯惇/张辽/许褚/
郭嘉/甄姬；蜀 7：刘备/关羽/张飞/诸葛亮/赵云/马超/黄月英；吴 8：孙权/
甘宁/吕蒙/黄盖/周瑜/大乔/陆逊/孙尚香；群 3：吕布/貂蝉/华佗）+ 张角。
N08 剩余：主公技全量语义（激将简化项）、顺手牵羊及谦逊对位、连营
弃牌路径触发——均挂账覆盖清单。
