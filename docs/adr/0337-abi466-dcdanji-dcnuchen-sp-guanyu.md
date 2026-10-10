# ADR 0337：ABI 466 单骑觉醒 + 怒嗔揭示弃置复合 + 新杀SP关羽

本批为 SP2 包新杀SP关羽（`dc_jsp_guanyu`，4047577994）的「单骑」（`dcdanji`，
3851102612）觉醒技完整移植（其派生「怒嗔」`dcnuchen`，2960451206），需要 ABI 466
两个新原语：过滤器 `owner_is_target_hej_above_hp` 与复合 `resolve_dcnuchen`。

## 过滤器（ABI 466）

- `owner_is_target_hej_above_hp`：owner 为事件角色且处于准备阶段，手牌 + 装备区 +
  判定区牌数之和大于体力值（旧 JS `countCards("hej") > getHp()`）。

## 复合 resolve_dcdanji（觉醒结算，ABI 466）

DSL 效果参数在触发批次生成时以「批前」状态求值（既有架构约束），而旧 JS 的
`num = maxHp - hp` 在 `loseMaxHp()` **之后**求值——故回复/摸牌数量不能写在
DSL 表达式里。`resolve_dcdanji(player, skill)` 以权威顺序执行：移除本技 →
减 1 体力上限 → 依新上限求 `num = maxHp - hp` → 回复 num → 摸 num 张 →
授予【马术】(def 2) 与【怒嗔】(def 2960451206)。

## 复合 resolve_dcnuchen（怒嗔，ABI 466）

阶段机（Custom 续体 `RESOLVE_DCNUCHEN_CUSTOM_CHOICE`）：
- 0：选择一名有手牌的其他角色（SelectTargets）；
- 1：选择其一张手牌（SelectCards，可取消）；
- 2：揭示该牌；owner 从自己的手牌/装备中按揭示花色多选弃置（可取消）；
  无同花色牌时直接进入获得分支；
- 3：弃置 → 对目标造成等量伤害（nocard）；取消 → 获得目标所有该花色手牌。

DSL 以 `phase_play owner_is_target` 触发（每回合一次，天然满足「限一次」），
VM 首发时 `skill` 为 `SkillId::NONE`，续体验证按 owner/回合/阶段守卫而非技能实例。

## 打包与验证

- `dcdanji.skill`（含 `dcnuchen`）加入 sp2 README 显式列表；`dc_jsp_guanyu`
  （男/魏/4 体力，技能 [dcdanji]）加入 sp2 `characters.json`；catalog 两技能标记
  `ported`。
- sp2 npack 重建（ABI 466，39 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/sp2/`。
- 运行时测试 `original_sp2_dcdanji.rs`（3 用例全绿）：觉醒结算（减上限→回复→摸牌→
  获技）、怒嗔弃置分支（1 张同花色弃置 → 1 点伤害）、怒嗔获得分支（取消 → 获得
  目标同花色手牌）。
- 旧 JavaScript 不进入 npack 或运行时。
