# ONLYOL 确定性移植

`qixian`（七弦）：把手牌上限基数设为 7，不吞掉后续手牌上限加减修正。

`olqianju`（千驹）：已损体力动态减少距离；当所有其他存活角色均在攻击范围内时，
出牌阶段使用【杀】的次数上限 +1。

`old_oldanxin`（旧版丹心）：受到伤害后可摸一张牌，并把同名升级标记增加至多两级。

`oldanxin`（丹心）：受到伤害后可按当前等级摸牌，再将等级提升至多两级；第二次升级时清除
〖矫诏〗本轮的已使用状态。

`oljieyong`（竭勇）：手牌中恰好只有一张红牌时，可将该红牌当【杀】使用或打出。

武将 `ol_guanping`（OL界关平）：蜀，4 体力；通过 YIJIANG 依赖授予完整 `longyin`，并授予本包完整 `oljieyong`。

武将 `ol_jianyong`（OL界简雍）：蜀，3 体力；通过 YIJIANG 依赖授予 `jyzongshi` 与胜利分支 `qiaoshui3`。`olqiaoshui` 可在胜利后继续发动，平局或失败后本回合禁用锦囊并暂时移除自身，回合结束恢复。

`olfuli`（伏枥）：限定技，自己濒死时可回复并补牌至存活势力数；本局累计造成伤害点数低于该值时翻面。

`olqieting`（OL窃听）：其他角色回合结束时，若其本回合未使用牌指定其他角色，
可取消或选择移动其一张兼容装备、摸一张牌，也可依次执行两项；若其指定过其他角色但
未对其他角色造成伤害，则只能取消或执行其中一项。

武将 `ol_caifuren`（OL蔡夫人）：群，3 体力；本包提供完整 `olqieting`，并通过
YIJIANG 依赖授予完整 `xianzhou`。

`olqingxi`（倾袭）：使用【杀】对原目标造成伤害前可发动；无武器时由目标选择弃置至多
攻击范围张手牌/装备牌或令伤害 +1，有武器时依次强制执行弃牌、伤害 +1，并弃置武器。

武将 `ol_caoxiu`（OL界曹休）：魏，4 体力；本包提供完整 `olqianju` 与 `olqingxi`。

`oljiangchi`（将驰）：摸牌阶段结束后可取消，或摸一张牌并令本回合【杀】次数上限 -1、
【杀】不计入手牌上限；也可重铸一张手牌/装备牌，令本回合【杀】无距离限制且次数上限 +1。

武将 `ol_caozhang`（OL界曹彰）：魏，4 体力；本包提供完整 `oljiangchi`。

`oldangxian`（当先）：回合开始时先执行一个独立的额外出牌阶段；该阶段开始时可从牌堆、
弃牌堆依次检索并获得一张仅自身无距离限制的【杀】。接受后若本阶段未造成伤害，阶段结束
时对自己造成 1 点伤害。

武将 `ol_liaohua`（OL界廖化）：蜀，4 体力；本包提供完整 `oldangxian` 与 `olfuli`。

`oljingce`（精策）：任意角色的出牌阶段结束时，若技能拥有者在该精确阶段内使用过牌，
可按其使用过的基本牌、锦囊牌、装备牌类别数摸牌；自己的回合内，手牌上限增加本回合从
手牌使用过的不同花色数。

武将 `ol_guohuai`（OL郭淮）：魏，4 体力；本包提供完整 `oljingce`。

`oljuece`（绝策）：结束阶段可选择一名手牌数不大于自己的其他角色，对其造成 1 点普通伤害；
目标范围在询问和提交时都会重新校验。

```bash
cargo run -q -p content-cli -- pack \
  content/original/ported/onlyOL/*.skill \
  --characters content/original/ported/onlyOL/characters.json \
  --metadata content/original/ported/onlyOL/metadata.json \
  --out content/original/ported/onlyOL/onlyOL.npack.json \
  --package-id onlyOL
```
`chuyuan`（储元，OL 曹丕）：任意角色受伤后,其摸一张牌并将一张手牌置于你武将牌上（储,ABI 567 choose_other_hand_store_skill_pile 首个用例）;储数<体力上限时触发,「选牌者」以拥有者选择近似。
`dengji`（登极,OL 曹丕）：觉醒技。准备阶段若「储」≥3 → 减 1 上限 + 获得全部储牌(ABI 569 gain_all_skill_pile_cards 首个用例);「获天行/界奸雄」未建模。曹丕链(chuyuan 储元 + dengji 登极)至此闭环。
