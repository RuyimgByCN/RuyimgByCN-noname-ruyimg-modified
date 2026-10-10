# ADR 0310：SHIJI SP 孙尚香「存嗣」与 `resolve_spcunsi`（ABI 406）

ABI 406 增加 `resolve_spcunsi(player, target)` 权威原语：拥有者翻面，
从牌堆/弃牌堆取一张【杀】移动到目标手牌。若两处均无【杀】则仅翻面。

「存嗣」（`spcunsi`）以该原语为主体，DSL 侧组合：
- 主动技（出牌阶段限一次，目标为其他角色）：`resolve_spcunsi(self, event_source)`
  + `add_mark(event_source, spcunsi2_def, 1)` + `add_skill(event_source, spcunsi2_def)`。
- 「存嗣·嗣」（`spcunsi2`）：`modifier sha_damage add owner_skill_mark`（伤害 + 嗣标记数），
  使用【杀】后 `remove_skill(self_skill)` 移除。

SP 孙尚香（`sp_sunshangxiang`）暂缺「枭姬」等其余技能，本技能先行发布，不入选将池。
旧 JavaScript 不进入 npack 或运行时。
