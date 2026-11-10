# G0 · U05 杀海拾遗（裁决 34 逐项清单 · 分册 2：第 1101–1621 项）

> 主文档：[U05-杀海拾遗.md](./U05-杀海拾遗.md) · 台账：[upstream-extension-reconstruction.md](../upstream-extension-reconstruction.md)

| 类别 | 实体 ID | 上游 `file:line` | 行为摘要 | 原生落点 | 状态 |
|---|---|---|---|---|---|
| 技能 | `kunlunjing2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:4634` | 幻镜；触发="phaseAfter"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `oldliaoyuan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:4676` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `oldliaoyuan2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:4723` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `shehun` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:4768` | 摄魂：出牌阶段限一次，你可以弃置任意张花色不同的牌，另一名其他角色弃置等量的牌，若其弃置的牌中有牌的花色与你弃置的牌相同，你对其造成1点伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:shehun`） |
| 技能 | `liaoyuan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:4856` | 燎原：每当你使用一张【杀】指定目标后，你可以弃置任意张与此【杀】花色相同的牌，若如此做，目标需额外打出等量的闪，每少打出一张闪，此【杀】的伤害+1。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:liaoyuan`） |
| 技能 | `liaoyuan2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:4917` | 触发="damageBegin"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `liaoyuan3` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:4929` | 触发="shaEnd"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `dunxing` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:4936` | 遁形：当你成为其他角色卡牌的目标时，你可以弃置一张牌并进行一次判定，若不为红桃，则取消之。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:dunxing`） |
| 技能 | `duanxing` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:4982` | 锻星：每当你装备一张未强化的装备牌，可以视为一名角色使用一张【杀】。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:duanxing`） |
| 技能 | `meihuo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5005` | 魅惑：每当你失去最后一张装备牌，你可以获得一名其他角色的一张牌，若此牌来自装备区，你立即装备之。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:meihuo`） |
| 技能 | `yishan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5059` | 异闪：每当你受到一次伤害，你可以重新获得最近失去的两张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:yishan`） |
| 技能 | `yishan2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5149` | 异闪；触发="loseEnd"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `guanhu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5159` | 贯鹄：每当你使用【杀】造成伤害，你可以弃置对方一张手牌和一张装备牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:guanhu`） |
| 技能 | `poxing` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5186` | 破星：锁定技，每当你即将造成伤害，若目标的体力值大于你，你令伤害+1。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:poxing`） |
| 技能 | `huanhun` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5196` | 唤魂：当一名角色进入濒死状态时，你可以弃置一张牌并令其进行一次判定，若结果为红色，其回复1点体力，否则其获得你弃置的牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:huanhun`） |
| 技能 | `yinyue` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5240` | 引月：每当有一名角色回复一次体力，你可以令其摸一张牌，若该角色不是你且你的手牌数不大于该角色，你也摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:yinyue`） |
| 技能 | `daofa` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5261` | 道法：每当有一名其他角色造成伤害，你可以令其弃置一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:daofa`） |
| 技能 | `daixing` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5278` | 代形：结束阶段，你可以弃置任意张牌并获得等量的护甲；这些护甲将在你的下个准备阶段消失。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:daixing`） |
| 技能 | `daixing2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5329` | 触发="phaseBegin"；护甲 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:daixing2`） |
| 技能 | `daixing3` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5339` | 触发=["damageEnd", "damageZero"]；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:daixing3`） |
| 技能 | `swd_wuxie` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5352` | 无邪：锁定技，你不能成为其他角色的延时锦囊的目标。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:wuxie`） |
| 技能 | `lqingcheng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5361` | 倾城：结束阶段，你可以进行判定，若为红色则可以继续判定，最多判定3次，判定结束后将判定成功的牌收入手牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:lqingcheng`） |
| 技能 | `lingxin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5410` | 灵心：结束阶段，你可以亮出牌堆顶的三张牌，然后获得其中的红桃牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:lingxin`） |
| 技能 | `lingwu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5439` | 灵舞：回合结束后，若你在本回合内使用的牌数不少于当前体力值，你可以进行一个额外的回合（不可重复发动）。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:lingwu`） |
| 技能 | `shengong` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5456` | 神工：每当你需要打出一张【杀】或【闪】时，你可以弃置一名其他角色装备区内的一张武器牌或防具牌，视为打出一张【杀】或【闪】，然后该角色摸一张牌，你弃一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:shengong`） |
| 技能 | `huajian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5569` | 化剑：出牌阶段结束时，你可以将一张牌当作【杀】对任意一名角色使用。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:huajian`） |
| 技能 | `polang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5601` | 破浪：每当你造成一次伤害，可以一张对方的装备牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:polang`） |
| 技能 | `jikong` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5618` | 亟空：准备阶段，你可以指定一名角色视为对其使用一张雷【杀】；每当你失去最后一张手牌，你可以指定一名角色视为对其使用一张雷【杀】（每回合限发动一次）。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:jikong`） |
| 技能 | `xielei` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5660` | 挟雷：每当你使用或打出一张【杀】，可以弃置一张牌并对目标以外的一名角色造成1点雷电伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xielei`） |
| 技能 | `pozhen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5705` | 破阵：每当你受到一次伤害，若你的手牌数大于伤害来源，你可以弃置X张手牌对其造成1点伤害；若你的手牌数小于伤害来源，你可以弃置其X张手牌。X为你与伤害来源的手牌数之差。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:pozhen`） |
| 技能 | `tanlin_defence` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5766` | 触发="damageEnd"；弃牌 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `tanlin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5803` | 探麟：出牌阶段限一次，你可以与一名其他角色进行拼点，若你赢，你获得双方拼点牌、对该角色使用卡牌无视距离且可以额外使用一张【杀】直到回合结束，若你没赢，你受到该角色的1点伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `tanlin2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5853` | 探麟；触发="phaseAfter"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `tanlin3` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5860` | 触发="phaseAfter"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `tanlin4` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5875` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `yunchou` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5884` | 运筹：出牌阶段限一次，你可以弃置一张手牌，并弃置一名其他角色的一张手牌，若两张牌颜色相同，你摸一张牌，否则对方摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:yunchou`） |
| 技能 | `jqimou` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5925` | 奇谋：每当你于回合外受到一次伤害，你可以摸一张牌，并可以使用一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:jqimou`） |
| 技能 | `lexue` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5950` | 乐学：回合内，你随机获得制衡、集智、缔盟、驱虎中的一个技能；回合外，你随机获得遗计、急救、鬼道、反馈中的一个技能。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:lexue`） |
| 技能 | `lexue1` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5953` | 触发="phaseBegin"；获得技能 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `lexue2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:5991` | 触发="phaseEnd"；获得技能 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `swdtianshu2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6029` | 天书；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `swdtianshu3` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6103` | 触发=["useCardAfter", "useSkillAfter", …；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `swdtianshu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6113` | 天书：出牌阶段，你可以弃置一张锦囊牌，然后获得一名其他角色的一项技能直到该角色死亡（替换以此法获得的前一个技能）。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:swdtianshu`） |
| 技能 | `swdtianshu_remove` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6242` | 触发="dieAfter"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `luomei` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6254` | 落梅：每当你使用或打出一张梅花花色的牌，你可以摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:luomei`） |
| 技能 | `xingdian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6281` | 星点：出牌阶段限一次，你可以弃置一张手牌，然后随机弃置两名敌人各一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xingdian`） |
| 技能 | `yulin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6327` | 玉鳞：每当你即将受到伤害，你可以弃置一张装备牌抵消此伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:yulin`） |
| 技能 | `funiao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6356` | 符鸟：出牌阶段限一次，你可以将一张手牌交给一名其他角色，然后摸一张牌并观看其手牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:funiao`） |
| 技能 | `xuehuang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6398` | 血凰：限定技，出牌阶段，若你没有黑色手牌，你可以展示并弃置所有手牌，每弃置一张牌视为使用一张火【杀】，随机指定两名敌人为目标。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xuehuang`） |
| 技能 | `zhuyu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6454` | 朱羽：每当一名横置的角色即将受到伤害时，你可以弃置一张红色牌令此伤害+1并变为火属性。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:zhuyu`） |
| 技能 | `ningshuang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6504` | 凝霜：每当你成为黑色牌的目标，你可以弃置一张黑色牌将其横置，并摸一张牌，若其已经横置则改为将其翻面。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:ningshuang`） |
| 技能 | `xielv` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6563` | 谐率：弃牌阶段结束后，若你的所有手牌（至少两张）颜色均相同，你可以展示所有手牌，然后回复1点体力并弃置场上的所有判定牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xielv`） |
| 技能 | `xiaomoyu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6619` | 魔愈：锁定技，每当你于一个回合内首次造成伤害，你回复1点体力，若你没有受伤，则改为摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xiaomoyu`） |
| 技能 | `tianhuo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6657` | 天火：出牌阶段，你可以令所有角色弃置其判定区域内的牌，并受到没有来源的等量火焰伤害，每阶段限一次。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:tianhuo`） |
| 技能 | `huanyin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6701` | 幻音：锁定技，每当你成为其他角色的卡牌的目标时，你进行一次判定，若为黑桃则取消之，若为红桃你摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:huanyin`） |
| 技能 | `xuanzhou` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6742` | 玄咒：出牌阶段限一次，你可以将一张普通锦囊牌当作任意一张延时锦囊，对任意一名角色使用（无视锦囊使用范围限制）。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xuanzhou`） |
| 技能 | `tianlun` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6815` | 天轮：任意一名角色的判定牌生效前，你可以弃置一张场上角色的判定牌代替之。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:tianlun`） |
| 技能 | `lanzhi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6876` | 兰芷：每当你使用一张梅花牌，你可以令所有体力值不大于你的角色回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:lanzhi`） |
| 技能 | `duanyi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6926` | 断意：出牌阶段限一次，你可以弃置两张【杀】，对一名角色造成1点伤害，然后其随机弃置X张牌，X为其已损失的体力值。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:duanyi`） |
| 技能 | `guxing` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6957` | 孤星：出牌阶段，你可以将最后至多X张手牌当【杀】使用，此【杀】无视距离且可以指定至多3个目标，每造成一次伤害，你摸一张牌，Ｘ为你已损失的体力值且至少为１。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:guxing`） |
| 技能 | `guxing1` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:6960` | 孤星（sha） | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `guxing3` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7008` | 触发="damageAfter"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:guxing3`） |
| 技能 | `miles_xueyi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7023` | 血裔：锁定技，你防止即将受到的伤害，然后失去1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:miles_xueyi`） |
| 技能 | `jifeng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7032` | 触发="useCardToBefore"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:jifeng`） |
| 技能 | `swd_xiuluo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7072` | 修罗：准备阶段，你可以弃一张手牌来弃置你判定区里的一张延时类锦囊（必须花色相同）。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:swd_xiuluo`） |
| 技能 | `mohua2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7119` | 魔化：锁定技，当你进入濒死状态时，你立即变身为撒旦，将体力回复至２，然后摸两张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:mohua2`） |
| 技能 | `duijue` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7160` | 对决：限定技，出牌阶段，你可以指定一名体力值大于1的其他角色，你结束出牌阶段，并在回合结束后将所有其他角色移出游戏，然后该角色与你轮流进行回合，直到有一方死亡或一共进行六个回合为… | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `duijue3` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7244` | 触发="phaseAfter"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `duijue2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7287` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `yueren` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7316` | 月刃：每当你使用一张【杀】，可以进行一次判定，若结果为黑色，你弃置目标一张牌，若结果为红色，你将此【杀】收回，每回合限发动一次。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:yueren`） |
| 技能 | `busi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7344` | 不死：锁定技，当你进入濒死状态时，你进行一次判定，若结果不为黑桃，你将体力回复至1并将武将牌翻至背面。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:busi`） |
| 技能 | `xuying` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7367` | 虚影：锁定技，每当你即将受到伤害，你防止此伤害，若你此时有手牌，你失去1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xuying`） |
| 技能 | `yinguo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7387` | 因果：除你之外的任意一名角色即将受到受到伤害时，若有伤害来源，你可以弃置一张牌将伤害来源和目标对调。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:yinguo`） |
| 技能 | `yinguo3` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7434` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `guiyan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7456` | 鬼眼：出牌阶段，你可以观看一名角色的手牌，并获得其中一张梅花牌，每阶段限一次。当你首次进入濒死状态时，你须回复1点体力并失去技能鬼眼。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:guiyan`） |
| 技能 | `guiyan2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7490` | 鬼眼；触发="dying"；回复 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `xianyin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7500` | 散结：出牌阶段，你可以令所有判定区内有牌的角色弃置判定区内的牌，然后交给你一张手牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xianyin`） |
| 技能 | `mailun` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7556` | 脉轮：准备阶段，你可以选择一个脉轮效果直到下一回合开始。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:mailun`） |
| 技能 | `mailun31` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7682` | 脉轮；触发="damageBegin"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `mailun32` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7690` | 脉轮；触发="damageBegin"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `mailun41` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7698` | 脉轮；触发="damageBegin"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `mailun42` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7706` | 脉轮；触发="damageBegin"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `mailun5` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7714` | 触发="phaseDrawBegin"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `mailun6` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7727` | 触发="phaseDrawBegin"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `mailun7` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7740` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `mailun8` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7750` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `fengming` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7760` | 凤鸣：出牌阶段限一次，你可以弃置一张装备牌，令一名角色恢复1点体力并摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:fengming`） |
| 技能 | `wanjun` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7786` | 万钧：你可以将一张装备牌当作南蛮入侵使用。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:wanjun`） |
| 技能 | `huanling` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7812` | 幻灵：结束阶段，你可以选择一名角色与你同时翻面；翻面状态下，你防止一切伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:huanling`） |
| 技能 | `huanling2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7859` | 幻灵；触发="damageBefore"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `ljifeng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7869` | 疾风：锁定技，你的攻击范围+X，【杀】可以额外指定X个目标，X为你已损失的体力值。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:ljifeng`） |
| 技能 | `lxianglong` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7881` | 翔龙：每当你闪避一张【杀】，你可以视为使用一张【杀】。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:lxianglong`） |
| 技能 | `zhenjiu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7909` | 针灸：出牌阶段限一次，你可以将一张红色手牌置于一名角色的武将牌上，该角色于下一个准备阶段回复1点体力，然后获得此牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:zhenjiu`） |
| 技能 | `zhenjiu2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7942` | 针灸；触发="phaseBegin"；回复、获得牌 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `shoulie` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7956` | 狩猎：当你使用一张【杀】指定目标后，可以弃置一张手牌令此【杀】不可闪避。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:shoulie`） |
| 技能 | `hudun` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7988` | 盾甲：锁定技，当你对其他角色造成伤害后，若你没有护甲，你获得1点护甲值。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:hudun`） |
| 技能 | `toudan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:7999` | 投弹：出牌阶段限一次，你可以弃置一张黑桃牌对一名其他角色造成1点火焰伤害，然后你与距离该角色1以内的所有角色各弃置一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:toudan`） |
| 技能 | `shending` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:8041` | 神丁：锁定技，若你没有宝物牌，视为装备了蓝格怪衣。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:shending`） |
| 技能 | `poxiao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js:8107` | 破霄：你可以将一张装备牌当【杀】使用；当你没有武器牌时，你的攻击范围+1，【杀】可以额外指定一个目标。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:poxiao`） |
| 技能 | `lingquan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:5` | 灵泉：觉醒技，结束阶段，若游戏轮数不小于3且你本回合使用的牌数大于你的体力值，你摸三张牌，然后获得技能〖水蕴〗。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_lingquan`） |
| 技能 | `shenwu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:26` | 神舞：觉醒技，在一名角色的结束阶段，若你本局至少发动过3次〖水蕴〗，你增加1点体力和体力上限并获得2点护甲，然后获得技能〖回梦〗。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_shenwu`） |
| 技能 | `qiongguang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:43` | 穹光：弃牌阶段结束时，若你弃置了至少两张牌，你可以对所有敌方角色施加一个随机的负面效果。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `txianqu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:93` | 仙音：出牌阶段限一次，当你即将造成伤害时，你可以防止之，然后摸两张牌并回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_txianqu`） |
| 技能 | `xunying` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:153` | 迅影：每当你使用【杀】对一名目标结算完毕后，你可以继续对目标使用【杀】。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_xunying`） |
| 技能 | `liefeng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:172` | 冽风：锁定技，当你在回合内使用第二张牌时，你本回合获得〖炎斩〗；当你在回合内使用第三张牌时，你本回合获得〖天剑〗；当你在回合内使用第四张牌时，你本回合获得〖御风〗。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_liefeng`） |
| 技能 | `yuexing` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:238` | 越行：出牌阶段限一次，你可以指定一名角色，本阶段内将你与该角色到其他角色的距离基数互换。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_yuexing`） |
| 技能 | `yuexing2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:281` | 越行；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `minsha` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:297` | 冥煞：出牌阶段限一次，你可以弃置两张牌，对一名体力值大于1的其他角色造成1点雷属性伤害，然后距离目标1以内的所有其他角色随机弃置一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_minsha`） |
| 技能 | `lingdi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:358` | 灵笛：出牌阶段，你可以弃置一张本回合与此法弃置的牌花色均不同的手牌，然后选择一名与你距离为X的角色与其各摸一张牌，X为本回合发动〖灵笛〗的次数（含此次）。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `xiaoyue` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:411` | 啸月：锁定技，每轮开始时，若你手牌中有【杀】，你将手牌中的一张随机【杀】对一名随机敌方角色使用，然后获得1点护甲。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `xiaoyue2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:439` | 触发="damageEnd"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `huanlei` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:448` | 触发="damageEnd"；摸牌、伤害 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `anwugu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:464` | 巫蛊：每当你对其他角色造成一次伤害，你可以令目标获得三枚蛊标记；拥有蛊标记的角色手牌上限-1，每回合最多使用X张牌（X为蛊标记数），每个结束阶段失去一枚蛊标记。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_anwugu`） |
| 技能 | `anwugu2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:480` | 蛊；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_anwugu2`） |
| 技能 | `xtanxi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:513` | 探息：出牌阶段限一次，你可以弃置一张手牌，然后随机选择一名手牌中有与之同名的牌的敌方角色，观看其手牌并获得任意一张。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_xtanxi`） |
| 技能 | `linghuo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:574` | 灵火：每两轮限一次，在一名其他角色的结束阶段，若其本回合内造成过伤害，你可以对其造成1点火属性伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_linghuo`） |
| 技能 | `guijin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:593` | 归烬：每三轮限一次，出牌阶段，你可以观看牌堆顶的四张牌，然后可以将其中任意张牌分配给任意角色，被分到牌的角色跳过下一摸牌阶段，然后将剩余牌以原顺序放回牌堆顶。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_guijin`） |
| 技能 | `guijin2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:704` | 归烬；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `chengxin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:719` | 澄心：每四轮限一次，当一名角色进入濒死状态时，你可以令其将体力值回复至1，然后该角色防止一切伤害直到下一回合结束。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_chengxin`） |
| 技能 | `chengxin2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:750` | 澄心；触发="damageBefore"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `tianwu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:773` | 天舞：每当你使用卡牌指定一名敌方角色为唯一目标，你可以对其施加一个随机的负面效果。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `shiying` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:785` | 逝影：限定技，在一名其他角色死亡前，你可以防止其死亡，然后你与该角色失去全部技能、将体力和体力上限变为3、移去所有牌并摸三张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_shiying`） |
| 技能 | `liguang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:833` | 离光：结束阶段，你可以弃置一张手牌并将场上的一张牌移动到另一个合理的位置。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_liguang`） |
| 技能 | `xiepan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:873` | 械磐：每当你失去最后一张基本牌，你可以弃置一张手牌，然后获得一张随机装备牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_xiepan`） |
| 技能 | `yujia` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:901` | 御甲：每当你使用一张未强化的装备牌，你可以随机观看X张机关牌，并选择一张获得之，X为你本局使用过的机关牌数且至少为1。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `yanshi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:973` | 偃师：觉醒技，结束阶段，若你累计有4个回合使用过机关牌，你增加1点体力和体力上限，然后用随机装备填满你的装备区。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `tanhua` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1028` | 昙华：锁定技，你回复体力的效果改为摸两张牌；当你进入濒死状态时，你回复1点体力并失去此技能。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `xjyingfeng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1051` | 影锋：锁定技，每当你使用一张【杀】结算完毕后，你随机对一名不是此【杀】目标的敌方角色使用一张【杀】。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_xjyingfeng`） |
| 技能 | `ywuhun` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1074` | 雾魂：锁定技，回合开始前，你获得一个额外的回合，并在此回合结束后复原场上及牌堆中的所有牌；当你在此回合中造成伤害后，终止所有结算并结束此回合。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `fenglue` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1249` | 风掠：你可以放弃出牌阶段，改为指定一名其他角色并选择任意张手牌，依次对该角色使用，若如此做，此阶段内该角色每受到1点伤害，你在结算后摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_fenglue`） |
| 技能 | `xjzongyu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1385` | 纵雨：出牌阶段限一次，你可以弃置一张黑色手牌，视为使用一张飞镖，随机指定两名敌方角色为目标。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `fanling` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1409` | 返灵：锁定技，每当一名角色失去体力，你回复1点体力，每回合只能发动一次。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_fanling`） |
| 技能 | `dujiang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1423` | 毒降：出牌阶段限一次，你可以弃置一张黑色牌令一名随机敌人获得一张毒。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `sheying` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1447` | 蛇影：每回合限一次，当你使用卡牌造成一次伤害后，你可以进行一次判定，若结果为黑色，你收回此牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_sheying`） |
| 技能 | `huahu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1468` | 化狐：限定技，你可以失去所有技能和1点体力上限，然后令任意名其他角色回复1点体力、获得1点护甲并摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_huahu`） |
| 技能 | `binxin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1519` | 冰心：在一名角色的结束阶段，若其体力值为1，你可以令其获得1点护甲。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_binxin`） |
| 技能 | `qixia` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1535` | 绮霞：锁定技，当你累计使用或打出了4种不同花色的牌后，你于本回合结束后获得一个额外回合。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `jianzhen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1582` | 剑阵：锁定技，当你使用【杀】对目标结算完毕后，其他角色可以对该目标使用一张【杀】，当有人选择出【杀】后终止此结算。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `husha` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1620` | 虎煞：每当你于出牌阶段造成1点伤害，你获得一枚虎煞标记（标记数不超过3）；结束阶段，你可以选择一项：1. 移去一枚虎煞标记，视为对任意角色使用一张【杀】；2. 移去两枚虎煞标记，… | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_husha`） |
| 技能 | `fenshi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1755` | 焚世：觉醒技，当你解除濒死状态时，你获得2点护甲，摸两张牌，然后获得技能〖龙火〗。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_fenshi`） |
| 技能 | `longhuo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1769` | 龙火：结束阶段，你可以对所有角色各造成1点火焰伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_longhuo`） |
| 技能 | `yanzhan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1799` | 炎斩：出牌阶段限一次，你可以将一张红色牌当作火【杀】使用，此【杀】只能用与之花色相同的闪响应；若此【杀】造成了伤害，你本回合可以额外使用一张【杀】。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `yanzhan2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1830` | 触发="damageEnd"；临时技 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `yanzhan3` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1841` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `yanzhan4` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1850` | 触发="loseEnd"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `xjyufeng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1859` | 御风：锁定技，当你失去手牌后，若手牌数少于2，你将手牌数补至2（每回合最多发动两次）。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_xjyufeng`） |
| 技能 | `feixia` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1884` | 飞霞：出牌阶段限一次，你可以弃置一张红色牌视为对一名随机敌人使用一张不计入出杀次数的【杀】。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `lueying` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1910` | 掠影：每当你使用一张【杀】，你可以随机获得目标的一张牌，然后目标可以指定一名其他角色，你弃置该角色一张牌（每回合限发动一次，没有弃牌目标时无法发动）。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_lueying`） |
| 技能 | `tianjian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1959` | 天剑：出牌阶段限一次，你可以将一张【杀】当作【万箭齐发】使用，受到伤害的角色随机弃置一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_tianjian`） |
| 技能 | `feng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:1985` | 风：锁定技，当你累计摸2次牌后，你下一次摸牌时摸牌数+1。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_feng`） |
| 技能 | `ya` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2009` | 雅：锁定技，当你累计受到2次伤害后，你下一次受到的伤害-1。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_ya`） |
| 技能 | `song` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2038` | 颂：锁定技，当你累计造成2次伤害后，你下一次造成的伤害+1。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_song`） |
| 技能 | `longxiang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2061` | 龙翔：当你使用【杀】指定目标后，你可以弃置目标若干张手牌直到其手牌数与你相同。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_longxiang`） |
| 技能 | `huxi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2075` | 虎袭：你可以失去1点体力并获得1点护甲，视为使用一张【杀】。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `xuanmo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2105` | 玄墨：出牌阶段限一次，你可以将一张手牌置于牌堆顶并随机获得两张与之类别相同的牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_xuanmo`） |
| 技能 | `danqing` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2137` | 丹青：结束阶段，若你累计使用了四张花色不同的牌，你可以选择至多4名角色随机获得一个正面效果。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `danqing_count` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2185` | 触发="useCard"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `sheling` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2196` | 摄灵：其他角色于你的回合内因使用、打出或弃置而失去牌时，你可以获得之（每回合最多发动三次）。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_sheling`） |
| 技能 | `zhaoyao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2231` | 招摇：其他角色的摸牌阶段开始时，你可以与其拼点，若你赢，你摸两张牌，然后将两张牌置于牌堆顶。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_zhaoyao`） |
| 技能 | `zhangmu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2298` | 障目：每回合限一次，当你需要使用或打出一张闪时，你可以展示一张闪，视为使用或打出了此闪。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_zhangmu`） |
| 技能 | `zhangmu_ai` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2345` | 触发="loseAfter"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `leiyu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2355` | 雷狱：结束阶段，你可以弃置一张黑色手牌，视为对本回合内所有成为过你的卡牌目标的角色使用一张【惊雷闪】。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `leiyu2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2413` | 触发="phaseUseBegin"；回复 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `leiyu3` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2420` | 触发="dieAfter"；回复 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `leiyu4` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2432` | 触发="useCardToBegin"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `feizhua` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2442` | 飞爪：当你使用一张【杀】时，你可以将与目标相邻的角色追加为额外目标。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `lingxue` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2502` | 灵血：锁定技，每当你回复1点体力，你获得1点护甲。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_lingxue`） |
| 技能 | `diesha` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2509` | 叠杀：锁定技，每当你使用【杀】造成伤害，受伤害角色失去1点体力，你回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `guijiang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2526` | 鬼降：出牌阶段限一次，你可以弃置一张黑色牌，令一名其他角色无法成为回复牌的目标直到你下一回合开始。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `guijiang2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2560` | 鬼降；触发=["dieBegin", "phaseBegin"]；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `guijiang3` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2583` | 触发="phaseBegin"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `fenxing` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2592` | 分形：锁定技，准备阶段，你有50%概率变身为另一形态。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `diewu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2619` | 蝶舞：出牌阶段，你可以将一张【杀】交给一名角色，若你于此阶段内首次如此做，你摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_diewu`） |
| 技能 | `lingyu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2651` | 灵愈：结束阶段，你可以令一名其他角色回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_lingyu`） |
| 技能 | `duxinshu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2677` | 读心：出牌阶段限一次，你可以观看一名其他角色的手牌，然后可以用一张手牌替换其中的一张。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `feixu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2722` | 飞絮：每当一名角色使用或打出一张闪，你可以令其摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_feixu`） |
| 技能 | `xuanyan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2740` | 玄炎：锁定技，你的火属性伤害+1；你造成火属性伤害后失去1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_xuanyan`） |
| 技能 | `xuanyan2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2744` | 玄炎；触发="damageBegin"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_xuanyan2`） |
| 技能 | `xuanyan3` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2754` | 触发="damageEnd"；失去体力 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_xuanyan3`） |
| 技能 | `ningbin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2765` | 凝冰：锁定技，每当你受到1次雷属性伤害，你回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_ningbin`） |
| 技能 | `xfenxin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2787` | 焚心：锁定技，每当你的体力值发生改变，你摸等量的牌；每当你杀死一名角色，你增加1点体力上限并回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `xfenxin2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2810` | 焚心（sha） | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `luanjian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2818` | 乱剑：出牌阶段，你可以将两张【杀】当【杀】使用，此【杀】无视距离，可以指定任意名目标且有50%的机率伤害+1。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_luanjian`） |
| 技能 | `luanjian2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2868` | 触发="damageBegin"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_luanjian2`） |
| 技能 | `ctianfu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2881` | 天符：出牌阶段，你可以将一张闪置于一名其他角色的武将牌上，该角色在下一次造成伤害时受到来自你的1点雷属性伤害并随机弃置一张牌，然后移去此牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_ctianfu`） |
| 技能 | `ctianfu2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2919` | 天符；触发="damageAfter"；伤害 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_ctianfu2`） |
| 技能 | `ctianfu3` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2951` | 天符；触发="dieBegin"；弃置 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_ctianfu3`） |
| 技能 | `shuiyun` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:2962` | 水蕴：结束阶段，你可以将一张与武将牌上的牌类别均不相同的手牌置于武将牌上称为“蕴”；任意一名角色处于濒死状态时，你可以弃置一张“蕴”令其回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_shuiyun`） |
| 技能 | `shuiyun5` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3048` | 水蕴；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_shuiyun5`） |
| 技能 | `wangyou` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3090` | 忘忧：其他角色的结束阶段，你可以弃置一张牌，令此回合内受过伤害的所有角色各摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_wangyou`） |
| 技能 | `wangyou2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3152` | 触发="damageEnd"；临时技 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_wangyou2`） |
| 技能 | `changnian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3163` | 长念：你死亡时，可以将所有牌交给一名其他角色，令其获得技能〖追思〗。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_changnian`） |
| 技能 | `changnian2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3200` | 追思：锁定技，结束阶段，你摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_changnian2`） |
| 技能 | `sajin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3208` | 洒金：出牌阶段限一次，你可以弃置一张手牌并指定任意名角色进行判定，若判定颜色与你弃置的牌相同，该角色回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_sajin`） |
| 技能 | `jtjubao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3244` | 聚宝：当其他角色于你的回合外首次弃置非基本牌时，你可以获得其中的随机一张。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_jtjubao`） |
| 技能 | `duci` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3288` | 毒刺：每当你失去一次装备牌，可以对距离1以内的一名其他角色造成1点伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_duci`） |
| 技能 | `xshuangren` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3329` | 双刃：当你的武器牌被替换时，你可以将其置于你的武将牌上，并获得此装备的武器效果（不含距离）。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_xshuangren`） |
| 技能 | `xshuangren2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3385` | sha | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_xshuangren2`） |
| 技能 | `guiyuan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3398` | 归元：出牌阶段限一次，你可以弃置一张【杀】，然后回复1点体力并摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_guiyuan`） |
| 技能 | `qijian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3416` | 气剑：弃牌阶段结束时，你可以指定至多X名目标视为使用一张【杀】，X为你于此阶段弃置的卡牌数。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `shenmu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3436` | 神木：任意一名角色濒死时，你可以展示你的手牌并弃置其中的所有红色牌（至少一张），若如此做，该角色回复1点体力，然后摸X张牌，X为你弃置的手牌数。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_shenmu`） |
| 技能 | `qianfang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3472` | 千方：准备阶段，若你有玄凝标记，你可以弃置3-X张牌和所有玄凝标记，视为使用了一张【万箭齐发】，每当一名敌方角色因此牌受到伤害，你摸一张牌。X为你的玄凝标记数。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_qianfang`） |
| 技能 | `qianfang2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3549` | 触发="phaseDrawBegin"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `poyun` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3557` | 破云：每当你造成一次伤害，你可以弃置一枚玄凝标记，然后弃置对方两张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_poyun`） |
| 技能 | `poyun2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3582` | 触发="damageEnd"；摸牌 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `zhuyue` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3596` | 逐月：出牌阶段限一次，你可以弃置一张非基本牌并指定至多两个目标各随机弃置一张牌，若如此做，你本回合使用的【杀】须指定选中角色为目标。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_zhuyue`） |
| 技能 | `zhuyue2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3674` | 触发="phaseUseEnd"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `longxi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3681` | 龙息：锁定技，在回合外每当你需要使用或打出一张卡牌时，若牌堆顶的前两张中有可使用或打出的牌，你立即获得之。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `guanri` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3733` | 贯日：限制技，你可以弃置两张红色手牌并失去1点体力，然后对一名体力值不少于你的其他角色造成2点火焰伤害并弃置其所有装备牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_guanri`） |
| 技能 | `tianxian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3781` | 天弦：锁定技，你的【杀】无视距离且可指定任意多个目标，目标须进行一次判定，若结果为黑色则取消之。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_tianxian`） |
| 技能 | `runxin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3814` | 润心：每当你使用或打出一张红桃牌，你可以令一名角色回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_runxin`） |
| 技能 | `xjzhimeng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3856` | 织梦：结束阶段，你可以选择一名其他角色将牌堆顶的一张牌置于该角色的武将牌上，直到你的下个准备阶段将其收入手牌。当一名角色武将牌上有织梦牌时，每当其成为与此牌类型相同的卡牌的目标，… | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_xjzhimeng`） |
| 技能 | `xjzhimeng2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3897` | 织梦；触发="useCardToBegin"；摸牌 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_xjzhimeng2`） |
| 技能 | `xjzhimeng3` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3923` | 织梦；触发=["phaseBegin", "dieBegin"]；弃置、获得牌 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `tannang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3948` | 探囊：出牌阶段限一次，你可以将一张梅花手牌当【顺手牵羊】使用；你的【顺手牵羊】无距离限制。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_tannang`） |
| 技能 | `tuoqiao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:3987` | 烟瘴：你可以将一张黑色牌当作【石灰粉】使用。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `huimeng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:4002` | 回梦：每当你回复1点体力，可以摸两张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_huimeng`） |
| 技能 | `tianshe` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:4012` | 天蛇：锁定技，你防止即将受到的属性伤害，每当你造成一次属性伤害，你回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_tianshe`） |
| 技能 | `tianshe2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js:4043` | 天蛇；触发="damageAfter"；回复 | 原生规则 / `rule-vm` 包 | 忠实实现（正式包已映射 `host:xj_tianshe2`） |
| 技能 | `xicai` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:5` | 惜才：你可以立即获得对你造成伤害的牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `diyjianxiong` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:29` | 奸雄：锁定技，在身份局中，于你回合内死亡的角色均视为反贼，国战中，于你回合内死亡的角色若与你势力相同则随机改为另一个势力。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `diykuanggu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:108` | 狂骨：锁定技，当你造成1点伤害后，若你在其攻击范围内，你回复1点体力；否则你摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `xiongzi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:143` | 雄姿：锁定技，你于摸牌阶段额外摸X+1张牌，X为你装备区牌数的一半，向下取整。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `yaliang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:150` | 雅量：每当你对其他角色造成1点伤害后，或受到其他角色造成的1点伤害后，你可与该角色各摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `diy_jiaoxia` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:184` | 皎霞：每当你成为红色牌的目标，你可以摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `yiesheng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:203` | 弃置任意张黑色手牌并摸等量的牌 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `guihan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:224` | 归汉：限定技，当你进入濒死状态时，可以指定一名男性角色与其各回复1点体力并摸两张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `luweiyan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:278` | 围堰：出牌阶段限一次，你可以将一张非基本牌当作【水攻】使用；结算后你可以视为对其中一个目标使用一张不计入出杀次数的【杀】。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `luweiyan2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:297` | 触发="useCardAfter"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `jieyan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:329` | 劫焰：出牌阶段限一次，你可以弃置一张红色手牌令场上所有角色受到1点火焰伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `honglian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:348` | 红莲：每当你受到来自其他角色的伤害，可以弃置伤害来源的所有红色牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `diyguhuo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:371` | 蛊惑：锁定技，准备阶段，你摸两张牌，然后弃置区域内的两张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `diychanyuan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:402` | 缠怨：锁定技，杀死你的角色失去1点体力上限。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `zonghuo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:429` | 纵火：你可弃置一张牌将你即将造成的伤害变为火焰伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `shaoying` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:458` | 烧营：每当你造成一次火焰伤害，可指定距离受伤害角色1以内的另一名角色，并亮出牌堆顶的一张牌，若此牌为红色，该角色受到1点火焰伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `xinfu_qiaosi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js:489` | 巧思：出牌阶段限一次，你可以投掷一枚六面骰子，亮出牌堆顶的X张牌并获得之。然后，你选择一项：1.交给一名其他角色X张牌。2.弃置X张牌。（X为骰子的点数） | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `stone_chenshui` | `archive/upstream/noname/apps/core/extension/杀海拾遗/main/content.js:397` | 沉睡：在你的结束阶段，令己方主将获得一张梦境牌。 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `_shengke` | `archive/upstream/noname/apps/core/extension/杀海拾遗/play/wuxing/skill.js:5` | 触发="useCardToBegin"；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 技能 | `wuxingpan_skill` | `archive/upstream/noname/apps/core/extension/杀海拾遗/play/wuxing/skill.js:79` | 五行：出牌阶段限一次，你可以永久改变一张手牌的五行属性 | 原生规则 / `rule-vm` 包 | 忠实实现（已按上游语义实现并接入正式包产品路径） |
| 卡牌 | `luyugeng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:4` | 鲈鱼羹：出牌阶段限一次，你可以弃置一张基本牌并发现一张牌，持续三回合。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `jinlianzhu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:26` | 金莲珠：对一名即将受到伤害的角色使用，防止此伤害，并令伤害来源摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `chunbing` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:57` | 春饼：你的手牌上限+1，持续五回合。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gudonggeng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:88` | 骨董羹：当你下一次受到杀造成的伤害时，令伤害-1，持续三回合。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `liyutang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:115` | 鲤鱼汤：结束阶段，若你的体力值为全场最少或之一，你获得1点护甲，持续两回合。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `mizhilianou` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:145` | 蜜汁藕：你可以将一张红桃牌当作桃使用，持续四回合。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `xiajiao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:180` | 虾饺：你在摸牌阶段额外摸一张牌，然后弃置一张牌，持续三回合。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `tanhuadong` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:203` | 昙花冻：出牌阶段结束时，你摸一张牌，持续三回合。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `mapodoufu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:225` | 麻婆豆腐：结束阶段，你弃置一名随机敌人的一张随机牌，持续两回合。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `qingtuan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:251` | 青团：你在回合内使用首张杀时摸一张牌，持续两回合。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yougeng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:291` | 酉羹：准备阶段，若你的体力值为全场最少或之一，你回复1点体力，持续两回合。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `molicha` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:321` | 茉莉茶：你不能成为其他角色的黑色牌的目标，持续四回合。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yuanbaorou` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:343` | 元宝肉：你在出牌阶段可以额外使用一张杀，持续四回合。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `heilonglinpian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:383` | 黑龙鳞片：出牌阶段对自己使用，获得1点护甲，直到下一回合开始，你的防御距离+1。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `mutoumianju` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:406` | 木头面具：出牌阶段限一次，你可以将一张手牌当作杀使用。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gjyuheng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:419` | 玉衡：出牌阶段限一次，若敌方角色有黑桃手牌，你可以弃置一张黑桃手牌，然后获得一名随机敌方角色的一张随机黑桃手牌（此牌在本局游戏中第三次和第七次发动效果后，分别自动获得一次强化）。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gjyuheng_plus` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:435` | 玉衡：由普通玉衡强化得到，将玉衡技能描述中的“弃置一张黑桃手牌”改为“弃置一张黑色手牌”。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gjyuheng_pro` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:452` | 玉衡：由普通玉衡二次强化得到，将玉横技能描述中的“弃置一张黑桃手牌”改为“弃置一张黑色手牌”，并去掉使用次数限制。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `shatang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:466` | 沙棠：出牌阶段对一名角色使用，对目标造成1点火焰伤害，然后目标获得1点护甲。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `shujinsan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:504` | 舒筋散：出牌阶段对任意角色使用，目标可弃置任意张牌，并摸等量的牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `dinvxuanshuang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:551` | 帝女玄霜：对一名濒死状态的角色使用，目标回复1点体力，然后可以弃置任意张牌并摸等量的牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `ziyangdan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:585` | 紫阳丹：出牌阶段对一名角色使用，目标获得3点护甲，此后每个准备阶段失去1点护甲，直到首次失去所有护甲或累计以此法失去3点护甲。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yunvyuanshen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:621` | 玉女元参：出牌阶段对一名角色使用，目标在下一次进入濒死状态时回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `bingpotong` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:656` | 天女散花：出牌阶段对至多3名角色使用，你与每个目标依次同时展示一张手牌，若颜色相同，你弃置展示的手牌，目标失去1点体力并终止结算。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `feibiao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:781` | 飞镖：出牌阶段，对一名距离1以外的角色使用，令其弃置一张黑色手牌或失去1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `longxugou` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:820` | 龙须钩：出牌阶段对一名装备区内有牌的其他角色使用，获得其装备区内的一张牌并装备之。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `qiankunbiao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:870` | 乾坤镖：随机弃置一名其他角色和其相邻角色的一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `shenhuofeiya` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:906` | 神火飞鸦：出牌阶段对一名其他角色和其相邻角色使用，目标需打出一张【闪】，否则受到1点火属性伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `mianlijinzhen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:992` | 棉里针：出牌阶段对一名体力值不小于你的角色使用，目标摸一张牌然后失去1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `liutouge` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:1017` | 六骰格：出牌阶段对一名角色使用，若目标是敌人，对目标施加一个随机的负面效果；否则对目标施加一个随机的正面效果。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `liufengsan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:1043` | 流风散：出牌阶段对一名角色使用，目标获得两张闪。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `shihuifen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js:1092` | 石灰粉：在一名其他角色的出牌阶段开始时对其使用，目标需打出一张【闪】，否则此阶段使用卡牌无法指定其他角色为目标。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_dieyi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:4` | 蝶翼；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_dieyi_equip1` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:7` | 蝶翼·器：在你从装备区中失去此牌后，你获得一枚“蝶翼”标记；在任意角色的结束阶段，你移去所有“蝶翼”标记，并随机弃置等量的牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_dieyi_equip2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:24` | 蝶翼·衣：在你从装备区中失去此牌后，你获得一枚“蝶翼”标记；在任意角色的结束阶段，你移去所有“蝶翼”标记，并随机弃置等量的牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_dieyi_equip3` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:40` | 蝶翼·攻：在你从装备区中失去此牌后，你获得一枚“蝶翼”标记；在任意角色的结束阶段，你移去所有“蝶翼”标记，并随机弃置等量的牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_dieyi_equip4` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:56` | 蝶翼·防：在你从装备区中失去此牌后，你获得一枚“蝶翼”标记；在任意角色的结束阶段，你移去所有“蝶翼”标记，并随机弃置等量的牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_dieyi_equip5` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:72` | 蝶翼·宝：在你从装备区中失去此牌后，你获得一枚“蝶翼”标记；在任意角色的结束阶段，你移去所有“蝶翼”标记，并随机弃置等量的牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_dieyi_judge` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:88` | 蝶翼·判：你在判定阶段移去此牌，并获得一枚“蝶翼”标记；在任意角色的结束阶段，你移去所有“蝶翼”标记，并随机弃置等量的牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_hudiewu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:100` | 蝴蝶舞：将其他角色在场上的所有牌替换为蝶翼（每当你失去一张蝶翼，你获得一枚“蝶翼”标记；在任意角色的结束阶段，你移去所有“蝶翼”标记，并随机弃置等量的牌），然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_yigeniyin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:158` | 伊格尼印：对敌方角色中体力值最高的一名随机角色造成1点火焰伤害，然后对场上体力值最高的所有角色各造成1点火焰伤害，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_leizhoushu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:245` | 雷咒术：获得技能雷咒术（锁定技，准备阶段，你令全场牌数最多的所有其他角色各随机弃置一张牌，若目标不包含敌方角色，将一名随机敌方角色追加为额外目标，结算X次，X为本局获得此技能的次… | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_aerdeyin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:293` | 阿尔德印：对一名随机敌方角色造成1点伤害，若目标武将牌正面朝上，则将其翻面；新的一轮开始时，若目标武将牌正面朝上，则在当前回合结束后进行一个额外回合，否则将武将牌翻回正面。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_ansha` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:356` | 暗杀：令一名体力为1的随机敌方角立即死亡，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_xinsheng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:398` | 新生：选择一名角色，随机观看12张武将牌，选择一张替代其武将牌，并令其增加1点体力，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_niuquzhijing` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:514` | 纽曲之镜：令全场体力最多的角色减少1点体力和体力上限，体力最少的角色增加1点体力和体力上限（不触发技能），然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_zhongmozhizhan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:609` | 终末之战：将所有角色区域内的所有牌置入弃牌堆（不触发技能），然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_ganhan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:661` | 干旱：所有角色减少1点体力上限（不触发技能），然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_huangjiashenpan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:737` | 皇家审判：获得任意一张金卡法术（皇家审判除外），然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_tunshi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:779` | 吞噬：随机移除一名敌方角色的一个随机技能，你获得此技能并减少1点体力和体力上限，被移除技能的角色增加1点体力和体力上限，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_chongci` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:844` | 冲刺：弃置所有牌并随机获得一张非金法术牌，每弃置一张手牌，便随机获得一张类别相同的牌；每弃置一张装备区内的牌，随机装备一件类别相同的装备；获得潜行直到下一回合开始，然后结束出牌阶… | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_youer` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:930` | 诱饵：将一名其他角色的所有手牌移出游戏，然后摸一张牌，当前回合结束后该角色将以此法失去的牌收回手牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_tongdi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:968` | 通敌：观看一名其他角色的手牌并获得其中一张，然后令目标获得一张杀。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_fuyuan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1001` | 复原：对一名濒死状态角色使用，目标回复1点体力并摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_zhuoshao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1027` | 灼烧：对任意名体力值为全场最高的角色使用，造成1点火属性伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_butianshu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1057` | 卜天术：出牌阶段对任意角色使用，将任意一张延时锦囊牌置入其判定区。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_shizizhaohuan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1137` | 十字召唤：从牌堆中获得一张杀以及决斗、火攻、火烧连营、南蛮入侵四张牌中的随机一张。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_zuihouyuanwang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1189` | 最后愿望：摸X张牌并弃置X张牌，X为存活角色数。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_zirankuizeng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1228` | 自然馈赠：选择任意一张铜卡法术使用。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_nuhaifengbao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1345` | 怒海风暴：天气牌，出牌阶段对一名角色使用，目标在结束阶段随机弃置一张牌，持续2回合。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_baishuang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1367` | 白霜：天气牌，出牌阶段对至多三名角色使用，目标下个摸牌阶段摸牌数-1。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_baobaoshu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1388` | 雹暴术：天气牌，出牌阶段对至多两名角色使用，目标每使用一张基本牌或锦囊牌，需弃置一张牌，直到下一回合结束。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_guaiwuchaoxue` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1411` | 怪物巢穴：出牌阶段限用一次，随机获得一个卖血技能直到下一回合开始；令一名随机敌方角色对你造成1点伤害，然后你回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_qinpendayu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1456` | 倾盆大雨：天气牌，出牌阶段对一名角色及其相邻角色使用，目标手牌上限-1直到下一个弃牌阶段结束。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_birinongwu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1486` | 蔽日浓雾：天气牌，出牌阶段对一名角色及其相邻角色使用，目标不能使用杀直到下一个出牌阶段结束。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_ciguhanshuang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1516` | 刺骨寒霜：天气牌，出牌阶段对一名角色及其相邻角色使用，目标下个摸牌阶段摸牌数-1。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_baoxueyaoshui` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1546` | 暴雪药水：令一名角色弃置两张手牌并摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_zhihuanjun` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1585` | 致幻菌：出牌阶段对一名已受伤角色使用，令其减少1点体力上限；若该角色仍处于受伤状态且手牌数小于体力上限，则重复此结算。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_zumoshoukao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1616` | 阻魔手铐：令一名角色非锁定技失效直到下一回合结束。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_aozuzhilei` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1648` | 奥祖之雷：对一名体力值不小于你的角色造成1点雷属性伤害，然后该角色摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_poxiao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1681` | 破晓：选择一项：解除任意名角色的天气效果并移除其判定区内的牌，或随机获得一张铜卡法术（破晓除外）并展示之。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_kunenfayin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1758` | 昆恩法印：出牌阶段对一名角色使用，目标防止所有非属性伤害，持续X个角色的回合（X为存活角色数且最多为5）。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_yanziyaoshui` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1785` | 燕子药水：令一名角色摸一张牌，若其手牌数为全场最少或之一，改为摸两张。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_wenyi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1814` | 瘟疫：令所有体力值为全场最少的角色随机弃置一张手牌；若没有手牌，改为失去1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_shanbengshu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/gwent/card.js:1850` | 山崩术：出牌阶段对自己使用，随机弃置两件敌方角色场上的装备。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `linghunzhihuo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js:4` | 灵魂之火：对一名角色造成1点火焰伤害，然后随机弃置一张手牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `jihuocard` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js:47` | 激活：摸一张牌，本回合手牌上限+2。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `zhaomingdan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js:70` | 照明弹：观看一名其他角色的手牌，并弃置其区域内的一张牌，然后其与你各摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `shijieshu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js:135` | 视界术：目标从牌堆或弃牌堆中随机装备两张类别不同的装备牌，然后弃置一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `shandianjian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js:210` | 闪电箭：目标角色展示一张手牌，然后若你能弃掉一张与所展示牌相同花色的手牌，则对该角色造成1点雷电伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `shihuawuqi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js:342` | 石化武器；获得牌、临时技 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `dunpaigedang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js:371` | 盾牌格挡：获得1点护甲值，摸一张牌，然后弃置一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `chuansongmen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js:399` | 传送门：摸一张牌并展示，若发生在出牌阶段，你可以立即使用摸到的牌，若如此做，你将传送门收回手牌（每阶段最多收回2张传送门）。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `tanshezhiren` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js:458` | 弹射之刃：出牌阶段对自己使用，依次按敌方-友方-敌方-的顺序随机弃置阵营内一名随机角色的一张牌（目标不包含你），共结算X次，X为存活角色数，若X为偶数，改为X-1。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `xingjiegoutong` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js:508` | 星界沟通：增加1点体力上限并回复1点体力，弃置你的所有手牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `shenenshu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js:549` | 神恩术：出牌阶段对自己使用，将所有手牌（含此张）替换为基本牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `zhiliaobo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js:591` | 治疗波：出牌阶段对一名受伤角色使用，目标进行一次判定，若结果为红色，则回复1点体力，否则获得1点护甲。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yuansuhuimie` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js:638` | 元素毁灭：对所有角色使用，令目标弃置0~2张牌，并受到2-X点雷电伤害，X为其弃置的手牌数。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `mtg_lindixiliu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js:4` | 林地溪流：从牌堆中摸一张红桃牌，然后弃置任意张红桃牌，每弃置一张牌，将一张延时锦囊牌置入一名随机敌方角色的判定区。地图效果：准备阶段，你可以弃置一张红桃牌，然后弃置判定区内的一张… | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `mtg_longlushanfeng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js:63` | 龙颅山峰：选项一：获得一张杀；选项二：获得一张闪。地图效果：你出杀的次数上限+1。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `mtg_cangbaohaiwan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js:99` | 藏宝海湾：选择一个扩展包，随机获得来自该扩展包的一张衍生牌。地图效果：你在摸牌时有可能摸到衍生牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `mtg_linzhongjianta` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js:165` | 林中尖塔：发现一张武器牌并装备之。地图效果：若你装备区内有武器牌，你可以将一张基本牌当作杀使用。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `mtg_youlin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js:201` | 幽林：随机使用一张普通锦囊牌，随机指定一个具有正收益的目标。地图效果：出牌阶段限一次，你可以弃置一张锦囊牌并发现一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `mtg_haidao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js:235` | 海岛：选项一：获得1点护甲（无视地图效果）；选项二：弃置一张牌并回复1点体力。地图效果：锁定技，当你获得护甲时，若你已受伤，改为回复等量体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `mtg_yixialan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js:275` | 依夏兰：选项一：随机获得一张基本牌；选项二：随机获得一张锦囊牌。地图效果：出牌阶段限一次，你可以将一张基本牌重铸为锦囊牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `mtg_shuimomuxue` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js:311` | 水没墓穴：你可以弃置一张牌并摸两张牌。地图效果：锁定技，你的手牌上限-1；若弃牌阶段弃置了至少一张牌，则在此阶段结束时摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `mtg_feixu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js:351` | 废墟：从弃牌堆中发现一张牌。地图效果：准备阶段，随机将弃牌堆的一张牌置于牌堆顶。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `mtg_shamolvzhou` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js:371` | 沙漠绿洲：发现一张基本牌。地图效果：你的基本牌不计入手牌上限。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `mtg_duzhao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js:391` | 毒沼：选择一名角色令其获得一张毒。地图效果：结束阶段，你获得一张毒。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `mtg_bingheyaosai` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js:441` | 冰河要塞：摸两张牌，然后弃置两张牌。地图效果：锁定技，每当你使用一张杀，若你有牌，则需弃置一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hufu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:4` | 玉符：你可以将一张玉符当作杀、闪或酒使用或打出。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `liuxinghuoyu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:16` | 流星火羽：出牌阶段，对一名角色使用，令目标弃置两张牌，或受到1点火焰伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `dujian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:99` | 脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yangpijuan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:189` | 羊皮卷：出牌阶段对自己使用，选择一种卡牌类别，发现一张该类别的卡牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `shencaojie` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:242` | 神草结：你的锦囊牌即将造成伤害时对目标使用，令此伤害+1；你即将受到锦囊牌伤害时对自己使用，令此伤害-1。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yuruyi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:282` | 玉如意：你有更高的机率摸到好牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `fengyinzhidan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:293` | 封印之蛋：随机使用两张普通锦囊牌（随机指定目标）。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yuchanqian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:364` | 乾玉蝉：在你行动时可当作杀使用；可用于煅造装备；在你使用一张牌后，此牌会随机切换形态。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yuchankun` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:375` | 坤玉蝉：在你行动时可当作草药使用；可用于煅造装备；在你使用一张牌后，此牌会随机切换形态。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yuchanzhen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:386` | 震玉蝉：在你行动时可当作酒使用；可用于煅造装备；在你使用一张牌后，此牌会随机切换形态。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yuchanxun` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:400` | 巽玉蝉：在你行动时可当作桃使用；可用于煅造装备；在你使用一张牌后，此牌会随机切换形态。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yuchankan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:412` | 坎玉蝉：在你行动时可当作神秘果使用；可用于煅造装备；在你使用一张牌后，此牌会随机切换形态。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yuchanli` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:454` | 离玉蝉：在你行动时可当作天仙酒使用；可用于煅造装备；在你使用一张牌后，此牌会随机切换形态。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yuchangen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:468` | 艮玉蝉：在你行动时可当作封印之蛋使用；可用于煅造装备；在你使用一张牌后，此牌会随机切换形态。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yuchandui` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:479` | 兑玉蝉：在你行动时可当作雪肌冰鲍使用；可用于煅造装备；在你使用一张牌后，此牌会随机切换形态。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `mujiaren` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:490` | 木甲人：出牌阶段限用一次，将你手牌中的非基本牌（含此张）替换为随机的机关牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `zhiluxiaohu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:533` | 指路小狐：出牌阶段对自己使用，视为对一名随机敌方角色使用一张杀，若此杀造成伤害，你摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `xuejibingbao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:567` | 雪肌冰鲍：出牌阶段对一名角色使用，该角色摸牌阶段摸牌数+1，持续2个回合。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gouhunluo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:592` | 勾魂锣：出牌阶段对一名角色使用，在3轮后你的准备阶段令该角色失去1点体力并弃置所有手牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `zhuquezhizhang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:615` | 朱雀之璋：可用于煅造装备；此牌在你手牌中时，每当你受到其他角色造成的伤害，你对伤害来源造成1点火属性伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `xuanwuzhihuang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:624` | 玄武之璜：可用于煅造装备；此牌在你手牌中时，每当你造成伤害，你回复等量的体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `huanglinzhicong` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:633` | 黄麟之琮：可用于煅造装备；此牌在你手牌中时，准备阶段，若你没有护甲，你获得1点护甲。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `cangchizhibi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:642` | 苍螭之璧：可用于煅造装备；此牌在你手牌中时，准备阶段，你可以选择至多3名角色横置或重置之。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `qinglongzhigui` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:651` | 青龙之圭：可用于煅造装备；此牌在你手牌中时，准备阶段，你摸两张牌然后弃置一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `baishouzhihu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:660` | 白兽之琥：可用于煅造装备；此牌在你手牌中时，每当你弃置卡牌，你可以弃置一名其他角色的一张随机牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `jiguantong` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:669` | 机关火筒：出牌阶段对所有其他角色使用，目标弃置一张手牌，或受到1点火焰伤害；若没有人选择受到伤害，使用者摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `donghuangzhong` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:750` | 东皇钟：结束阶段，你可以弃置一张红色手牌，并选择一名角色将一张随机单体延时锦囊置入其判定区。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `xuanyuanjian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:762` | 轩辕剑：装备时获得1点护甲；每当你即将造成一次伤害，你令此伤害加一并变为雷属性，并在伤害结算后失去1点体力。任何时候，若你体力值不超过2，则立即失去轩辕剑。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `pangufu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:785` | 盘古斧：锁定技，每当你造成一次伤害，受伤角色须弃置一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `lianyaohu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:798` | 炼妖壶：出牌阶段各限一次，你可以选择一项：1.弃置一张手牌，并将一名其他角色的一张手牌置入炼妖壶；2.弃置两张炼妖壶中的牌，从牌堆中获得一张与弃置的牌类别均不相同的牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `haotianta` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:823` | 昊天塔：锁定技，任意一名角色进行判定前，你观看牌堆顶的两张牌，并选择一张作为判定结果，此结果不可被更改，也不能触发技能。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `fuxiqin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:835` | 伏羲琴：出牌阶段限一次，你可以与一名其他角色进行拼点，若你赢，你可以指定另一名角色视为对方对该角色使用一张杀，否则对方可弃置你一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `shennongding` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:850` | 神农鼎：出牌阶段，你可以弃置两张手牌，然后回复1点体力。每阶段限一次。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `kongdongyin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:865` | 崆峒印：令你抵挡一次死亡，将体力回复至1，并摸一张牌，发动后进入弃牌堆。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `kunlunjingc` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:888` | 昆仑镜：出牌阶段限一次，你可以观看牌堆顶的三张牌，然后用一张手牌替换其中的一张。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `nvwashi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:903` | 女娲石：当一名角色濒死时，若你的体力值大于1，你可以失去1点体力并令其回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `guisheqi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:915` | 龟蛇旗：出牌阶段对一名角色使用，目标获得1点护甲。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `jiguanfeng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:936` | 机关蜂：出牌阶段对一名其他角色使用，目标需打出一张【闪】，否则非锁定技失效直到下一回合开始，并受到1点伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `jiguanyuan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:975` | 机关鸢：出牌阶段对一名其他角色使用，你将此牌和一张其它牌置于一名其他角色的武将牌上，然后摸一张牌；该角色于下一结束阶段获得武将牌上的牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `shenmiguo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1027` | 神秘果：出牌阶段内，当你使用一张基本牌或普通锦囊牌后使用，令此牌再结算一次。每阶段限用一次。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `qinglianxindeng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1051` | 青莲心灯：你防止锦囊牌造成的伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `lingjiandai` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1062` | 零件袋：出牌阶段对自己使用，获得3张随机零件。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `jiguanshu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1099` | 机关鼠：出牌阶段对自己使用，用随机祭器强化装备区内的一张随机装备，然后用随机零件强化其余的装备。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `jiguanyaoshu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1212` | 机关要术：出牌阶段对距离1以内的一名角色使用，目标随机装备一件装备牌并获得技能巧匠（每当你于回合外失去装备区内的牌，你获得一个随机零件）。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hslingjian_xuanfengzhiren` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1258` | 旋风之刃：可用于煅造装备；随机弃置一名角色的一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hslingjian_zhongxinghujia` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1280` | 重型护甲：可用于煅造装备；令一名角色装备一件随机防具，然后随机弃置其一张手牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hslingjian_xingtigaizao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1358` | 型体改造：可用于煅造装备；摸一张牌，本回合手牌上限-1。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hslingjian_shijianhuisu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1392` | 时间回溯：可用于煅造装备；令一名其他角色将其装备牌收回手牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hslingjian_jinjilengdong` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1445` | 紧急冷冻：可用于煅造装备；令一名武将牌正面朝上的其他角色获得2点护甲并翻面，该角色不能使用卡牌，也不能成为卡牌的目标直到武将牌翻回正面。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hslingjian_shengxiuhaojiao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1495` | 生锈号角：可用于煅造装备；令一名角色获得技能嘲讽，直到其下一回合开始。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hslingjian_yinmilichang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1525` | 隐秘力场：可用于煅造装备；令一名其他角色获得技能潜行，直到其下一回合开始。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `xingjunyan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1558` | 星君眼：你的杀造成的伤害+1；杀对你造成的伤害+1。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `qinglonglingzhu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1569` | 青龙灵珠：每当你造成一次属性伤害，你可以获得对方的一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `baihupifeng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1580` | 白狐披风：结束阶段，若你的体力值是全场最小的之一，你可以回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `fengxueren` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1597` | 封雪刃：你使用杀击中目标后，若目标武将牌正面朝上，你可以防止伤害，然后令目标摸一张牌并翻面。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `chilongya` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1609` | 赤龙牙：锁定技，你的火属性伤害+1。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `daihuofenglun` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1621` | 带火风轮：你的进攻距离+2，你的防御距离-1。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `xiayuncailing` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1633` | 霞云彩绫：你的进攻距离-1，你的防御距离+2。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `shentoumianju` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1639` | 神偷面具：出牌阶段限一次，你可以指定一名手牌比你多的角色，弃置一张手牌并进行一次判定，若结果不为梅花，你获得其一张手牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `xianluhui` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1651` | 仙炉灰：令所有已受伤角色获得1点护甲。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `xiangyuye` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1685` | 失去体力、弃牌 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `caoyao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1722` | 草药：出牌阶段，对距离为1以内的角色使用，回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `tianxianjiu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1747` | 天仙酒：出牌阶段对自己使用，你使用的下一张杀造成伤害后可以摸两张牌；濒死阶段，对自己使用，回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `huanpodan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1855` | 获得技能 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `langeguaiyi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1890` | 蓝格怪衣：出牌阶段限一次，你可以进行一次判定，然后按花色执行以下效果。红桃：你回复1点体力；方片：你摸一张牌；梅花：你令一名随机敌方角色随机弃置一张牌；黑桃：无事发生。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `guiyoujie` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1902` | 鬼幽结：出牌阶段，对一名其他角色使用。若判定结果为黑色，其失去1点体力并随机弃置一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yufulu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1936` | 御夫录：出牌阶段，可弃置一张武器牌令一名角色受到1点伤害，然后该角色获得此武器牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `xixueguizhihuan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1947` | 吸血鬼指环：锁定技，每当你使用杀造成1点伤害，你回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `zhufangshenshi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1958` | 祠符：出牌阶段，对一名其他角色使用，本回合内对其使用卡牌无视距离，结算后摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `jingleishan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:1995` | 惊雷闪：出牌阶段，对所有其他角色使用。每名目标角色需打出一张【杀】，否则受到1点雷电伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `chiyuxi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:2076` | 炽羽袭：出牌阶段，对所有其他角色使用。每名目标角色需打出一张【闪】，否则受到1点火焰伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `guangshatianyi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:2159` | 光纱天衣：锁定技，每当你即将受到伤害，有三分之一的概率令伤害减一。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `guilingzhitao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:2170` | 归灵指套：每当你受到1点伤害，你获得一个逆攻标记，标记数不能超过4。出牌阶段，你可以弃置所有逆攻标记并令对一名其他角色造成标记数一半的伤害（若非整数则向下取整并摸一张牌）。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `qipoguyu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:2198` | 奇魄古玉；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `sadengjinhuan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:2212` | 萨登荆环：当你的【杀】被【闪】避后，可以进行一次判定，若结果为红色目标需再打出一张【闪】。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `sifeizhenmian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:2223` | 四非真面：出牌阶段限一次，你可以令一名有手牌的其他角色进行一次判定，若结果为不为红桃且目标有可用的手牌，目标随机使用一张手牌（随机指定目标）。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `shuchui` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:2235` | 鼠槌：出牌阶段限一次，你可以指定一名攻击范围内的角色，依次将手牌中的至多3张杀对该角色使用，若杀造成了伤害，你摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `guiyanfadao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:2247` | 鬼眼法刀：每当你使用杀命中目标，你可以防止伤害，改为令目标失去1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `qiankundai` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/swd/card.js:2259` | 乾坤袋：你的手牌上限+1。当你失去该装备时，你摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `caochuanjiejian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/yunchou/card.js:4` | 草船借箭：出牌阶段对一名有手牌的其他角色使用，目标选择一项：将手牌中的所有杀（至少一张）交给你，并视为对你使用一张杀；或展示手牌并令你弃置任意张。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `geanguanhuo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/yunchou/card.js:62` | 隔岸观火：①出牌阶段，对一名其他角色使用，令目标与一名你指定的另一名角色拼点，赢的角色获得对方的一张牌；若点数相同，目标可弃置你一张牌。②存活角色不超过2时此牌可重铸。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `shezhanqunru` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/yunchou/card.js:132` | 舌战群儒：出牌阶段，对你使用。你请求所有有手牌的其他角色响应，然后同时与响应的角色拼点。若有角色响应且结果中你赢的次数更多，或若没有角色响应，你摸三张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `suolianjia` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/yunchou/card.js:245` | 锁链甲：锁定技，你防止即将受到的属性伤害，当装备时进入连环状态，当卸下时解除连环状态。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `chenhuodajie` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/yunchou/card.js:266` | 趁火打劫：任意一名其他角色受到伤害时对其使用，获得其一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `fudichouxin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/yunchou/card.js:288` | 釜底抽薪：与一名角色进行拼点，若成功则获得双方拼点牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `shuigong` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/yunchou/card.js:333` | 水攻：令所有有装备的角色各弃置一张装备牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `toulianghuanzhu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/yunchou/card.js:363` | 偷梁换柱：出牌阶段，对一名其他角色使用，你观看其手牌，然后可以用一张手牌替换其中的一张；若两张牌颜色相同，你摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `huoshan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/yunchou/card.js:442` | 火山：出牌阶段，对自己使用。若判定结果为红桃2~9，则目标角色受到2点火焰伤害，距离目标1以内的其他角色受到1点火焰伤害。若判定结果不为红桃2~9，将之移动到下家的判定区里。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hongshui` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/yunchou/card.js:505` | 洪水：出牌阶段，对自己使用。若判定结果为梅花2~9，该角色随机弃置三张牌，距离该角色为X的角色随机弃置3-X张牌，若没有牌则失去1点体力。若判定结果不为梅花2~9，将之移动到下家… | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `pozhenjue` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/zhenfa/card.js:4` | 破阵决；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `changshezhen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/zhenfa/card.js:24` | 长蛇阵：若你处于队列中，与你同一队列的所有角色摸一张牌，否则将与你逆时针距离最近的同势力角色移至你下家。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `tianfuzhen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/zhenfa/card.js:70` | 天覆阵：所有大势力角色弃置一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `dizaizhen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/zhenfa/card.js:96` | 地载阵：所有小势力角色摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `fengyangzhen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/zhenfa/card.js:123` | 风扬阵；临时技 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yunchuizhen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/zhenfa/card.js:144` | 云垂阵；临时技 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `qixingzhen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/zhenfa/card.js:165` | 七星阵：弃置所有围攻你的角色各一张牌，然后视为对所有你围攻的角色使用一张不计入出杀次数的杀。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `shepanzhen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/zhenfa/card.js:204` | 蛇蟠阵；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `longfeizhen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/zhenfa/card.js:237` | 龙飞阵；弃置 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `huyizhen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/zhenfa/card.js:269` | 虎翼阵（sha） | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `niaoxiangzhen` | `archive/upstream/noname/apps/core/extension/杀海拾遗/card/zhenfa/card.js:320` | 鸟翔阵：令所有非你阵营的队列的角色依次打出一张【闪】，或者受到1点伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yanjiadan_heart` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gujian/card.js:4` | 偃甲蛋：可以当作紫阳丹、玉女元参或沙棠使用。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yanjiadan_diamond` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gujian/card.js:68` | 偃甲蛋：可以当作流风散、舒筋散或神火飞鸦使用。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yanjiadan_club` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gujian/card.js:117` | 偃甲蛋：可以当作天女散花、六骰格或锦里针使用。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `yanjiadan_spade` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gujian/card.js:149` | 偃甲蛋：可以当作飞镖、乾坤镖或龙须钩使用。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwjinli_jiu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:4` | 先祖麦酒；脚本逻辑 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_xianzumaijiu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:9` | 先祖麦酒：出牌阶段对自己使用，你使用下一张【杀】造成伤害后，令所有友方角色摸一张牌；濒死阶段，对自己使用，回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_yioufeisi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:119` | 伊欧菲斯：选择两名角色，令目标依次视为对对方使用一张【杀】，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_luoqi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:158` | 罗契：选择一名角色视为对其使用一张不计入出杀次数的【杀】，然后所有其他角色可以对目标使用一张【杀】，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_jieluote` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:201` | 杰洛特：对一名角色造成1点伤害，若目标体力值大于2且为全场最多，改为造成2点伤害，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_yenaifa` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:261` | 叶奈法：对至多3名随机敌方角色施加一个随机负面效果，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_telisi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:294` | 特丽斯：对至多3名随机友方角色施加一个随机正面效果，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_hengsaite` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:329` | 亨赛特：视为使用一张万箭齐发，每当有一名角色因此受到伤害，你获得一张【杀】，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_fuertaisite` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:371` | 弗尔泰斯特：令至多两名角色各获得1点护甲，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_laduoweide` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:406` | 拉多维德：令一名角色的非锁定技失效直到其下一回合结束，并对其造成1点伤害，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_enxier` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:438` | 恩希尔：与一名手牌并不超过1的其他角色交换手牌，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_fulisi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:474` | 符里斯：选择至多三名角色，观看目标的手牌并可以弃置其中1~2张，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_kaerweite` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:526` | 卡尔维特：获得至多两名角色各一张手牌，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_bulanwang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:561` | 布兰王：弃置至多两张牌并摸数量等于弃牌数2倍的牌，跳过弃牌阶段，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_kuite` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:603` | 奎特：视为对一名手牌数不小于你的角色连续使用两张【决斗】，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_haluo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:637` | 哈洛：对所有体力值全场最少的角色造成1点伤害，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_dagong` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:668` | 达贡：视为同时使用刺骨寒霜、蔽日浓雾和倾盆大雨，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_gaier` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:704` | 盖尔：令一名角色增加或减少1点体力和体力上限，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_airuiting` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:751` | 艾瑞汀：令所有其他角色选择一项：使用一张【杀】，或失去1点体力，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_aisinie` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:796` | 埃丝涅：回复1点体力并获得任意一张银卡法术，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_falanxisika` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:833` | 法兰茜斯卡：随机观看三张金卡法术并使用其中一张，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gwmaoxian_huoge` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:875` | 霍格：观看牌堆顶的六张牌，使用至多两张，然后弃掉其余的牌，然后结束出牌阶段。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_wuyao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:925` | 雾妖：在你行动时可当作【杀】使用；回合结束后，从手牌中消失。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_lang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:938` | 狼：在你行动时可当作酒使用；回合结束后，从手牌中消失。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_dudayuanshuai1` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:951` | 杜达元帅：当你成为其他角色使用牌的目标时，你可以使用此牌取消之，然后获得对你使用的牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `gw_dudayuanshuai2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/gwent/card.js:975` | 杜达元帅：你不能使用、打出或弃置此牌；结束阶段，若此牌在你手牌中，你弃置之并随机弃置一张手牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsfashu_anyingjingxiang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:4` | 暗影镜像：当你使用或打出一张牌后，暗影镜像变为该牌的复制。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsfashu_buwendingyibian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:11` | 不稳定异变：出牌阶段对一名角色使用，目标将一张手牌重铸为同类别的牌，本回合可重复使用此牌，最多使用X次，X为当前体力值。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hstianqi_dalian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:94` | 达里安：每当你造成一次伤害，你回复等量的体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hstianqi_shali` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:107` | 莎莉：每当你回复体力，你获得等量的护甲。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hstianqi_nazigelin` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:120` | 纳兹戈林：当你装备此牌时，你获得1点护甲。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hstianqi_suolasi` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:136` | 索拉斯：当你失去此牌时，你回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsjixie_zhadan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:155` | 炸弹机器人：出牌阶段对自己使用，对一名随机敌人造成1点火属性伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsqizhou_feng` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:184` | 风之祈咒：出牌阶段对自己使用，令所有目标的敌人打出一张【杀】或受到1点雷属性伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsqizhou_shui` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:236` | 水之祈咒：出牌阶段对自己使用，回复2点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsqizhou_huo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:263` | 火之祈咒：出牌阶段对任意角色使用，令目标受到1点火属性伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsqizhou_tu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:287` | 土之祈咒：出牌阶段对任意其他角色使用，令目标获得1点护甲。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsqingyu_feibiao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:309` | 青玉飞镖：弃置一名角色的一张随机手牌和一张随机装备牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsqingyu_zhanfang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:348` | 青玉绽放：令一名角色增加1点体力上限并摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsqingyu_hufu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:379` | 青玉护符：令一名角色获得1点护甲。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsqingyu_shandian` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:400` | 青玉闪电：对一名角色造成1点雷电伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsqingyu_zhao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:424` | 青玉之爪：当你装备此装备时，摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsdusu_xueji` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:440` | 血蓟：随机弃置一名角色的两张装备牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsdusu_kuyecao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:468` | 枯叶草：令一名角色获得技能潜行，直到其下一回合开始。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsdusu_huangxuecao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:503` | 皇血草：摸两张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsdusu_huoyanhua` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:525` | 火焰花：对攻击范围内的一名角色造成1点火焰伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsdusu_shinancao` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:550` | 石楠草：令一名角色下一次造成的伤害+1。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsbaowu_cangbaotu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:578` | 藏宝图：结束阶段，将一张黄金猿猴置入你的手牌；摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsbaowu_huangjinyuanhou` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:602` | 黄金猿猴：将你的手牌（含此张）替换为随机炉石衍生牌，并获得潜行直到下一回合开始。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsshenqi_nengliangzhiguang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:663` | 能量之光：限武将牌正面朝上时使用，令一名角色增加1点体力上限，回复1点体力，并摸四张牌；使用后将武将牌翻至背面。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsshenqi_kongbusangzhong` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:699` | 恐怖丧钟：限武将牌正面朝上时使用，对所有其他角色各造成1点伤害；使用后将武将牌翻至背面。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsshenqi_morijingxiang` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:733` | 末日镜像：限武将牌正面朝上时使用，从所有其他角色的区域内各获得一张牌；使用后将武将牌翻至背面。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsmengjing_feicuiyoulong` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:775` | 翡翠幼龙：出牌阶段对任意一名角色使用，对目标造成1点伤害。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsmengjing_suxing` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:797` | 苏醒：令所有其他角色失去1点体力并随机弃置两张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsmengjing_mengye` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:823` | 梦魇：令一名角色摸一张牌，并在其下一个结束阶段弃置其所有牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsmengjing_mengjing` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:848` | 梦境：令一名角色将装备区内的所有牌收入手牌，并将一张乐不思蜀置于其判定区。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hsmengjing_huanxiaojiemei` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:892` | 欢笑姐妹：出牌阶段对一名已受伤角色使用，令目标恢复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hszuzhou_nvwudeganguo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:915` | 女巫的钳锅：出牌阶段对一名角色使用，目标弃置一张牌，然后随机获得一张炉石衍生牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hszuzhou_nvwudepingguo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:957` | 女巫的苹果：出牌阶段对一名角色使用，目标获得两张【杀】。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hszuzhou_nvwudexuetu` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:975` | 女巫的学徒：出牌阶段对没有咒降技能的角色使用，令目标非锁定技失效，并获得技能咒降直到下一回合结束。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hszuzhou_wushushike` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:1009` | 巫术时刻：出牌阶段对所有角色使用，将手牌中的【闪】替换为【杀】。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `hszuzhou_guhuo` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:1060` | 蛊惑：出牌阶段对一名其他角色使用，令其交给你一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `tuteng1` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:1087` | 治疗图腾：结束阶段，你回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `tuteng2` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:1094` | 灼热图腾：每当你造成一次伤害，你摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `tuteng3` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:1101` | 石爪图腾：你受到下一次伤害时，令伤害-1，然后失去此图腾。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `tuteng4` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:1108` | 空气图腾：在你的回合内，你的锦囊牌造成的首次伤害+1。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `tuteng5` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:1115` | 法潮图腾：结束阶段，你摸一张牌。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `tuteng6` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:1122` | 火舌图腾：在你的回合内，你的【杀】造成的首次伤害+1。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `tuteng7` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:1129` | 活力图腾：结束阶段，你令一名其他角色回复1点体力。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `tuteng8` | `archive/upstream/noname/apps/core/extension/杀海拾遗/character/hearth/card.js:1136` | 图腾魔像：你的进攻距离+1。 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 卡牌 | `wuxingpan` | `archive/upstream/noname/apps/core/extension/杀海拾遗/play/wuxing/card.js:2` | 五行盘：出牌阶段限一次，你可以永久改变一张手牌的五行属性 | 原生规则 / `rule-vm` 包 | 忠实实现 |
| 配置项 | `tip` | `archive/upstream/noname/apps/core/extension/杀海拾遗/main/config.js:4` | 五行生克（默认=false）：每名角色和部分卡牌在游戏开始时随机获得一个属性 | 原生规则（`crates/content-native-loader/src/shahai_config.rs`） | 忠实实现 |
| 配置项 | `wuxing` | `archive/upstream/noname/apps/core/extension/杀海拾遗/main/config.js:14` | 五行生克（默认=false）：每名角色和部分卡牌在游戏开始时随机获得一个属性 | 原生规则（`crates/content-native-loader/src/shahai_config.rs`） | 忠实实现 |
| 配置项 | `rand` | `archive/upstream/noname/apps/core/extension/杀海拾遗/main/config.js:19` | 带属性卡牌概率（默认="0.3"）：包含12种地图牌和3张武将牌，首次引入“随从”概念 | 原生规则（`crates/content-native-loader/src/shahai_config.rs`） | 忠实实现 |
| 配置项 | `mtg` | `archive/upstream/noname/apps/core/extension/杀海拾遗/main/config.js:29` | 万智牌（默认=true）：包含12种地图牌和3张武将牌，首次引入“随从”概念 | 原生规则（`crates/content-native-loader/src/shahai_config.rs`） | 忠实实现 |
| 配置项 | `zhenfa` | `archive/upstream/noname/apps/core/extension/杀海拾遗/main/config.js:34` | 阵法牌（默认=true）：国战模式专属卡牌扩展包 | 原生规则（`crates/content-native-loader/src/shahai_config.rs`） | 忠实实现 |
| 配置项 | `gwent` | `archive/upstream/noname/apps/core/extension/杀海拾遗/main/config.js:39` | 昆特牌（<span style=（默认=true）：包含32种法术牌和52张武将牌，并引入“天气牌” | 原生规则（`crates/content-native-loader/src/shahai_config.rs`） | 忠实实现 |
| 配置项 | `yunchou` | `archive/upstream/noname/apps/core/extension/杀海拾遗/main/config.js:44` | 运筹帷幄（<span style=（默认=true）：以三国背景设计的若干武将牌和卡牌 | 原生规则（`crates/content-native-loader/src/shahai_config.rs`） | 忠实实现 |
