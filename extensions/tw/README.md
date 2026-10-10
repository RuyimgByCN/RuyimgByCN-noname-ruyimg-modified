# TW 确定性移植

- `twjuyan`：对一名角色造成火焰伤害后摸一张牌，并令其减一点体力上限。
- `manji`：其他角色失去体力后，按双方结算后体力关系回复或摸牌。
- `gx_lingbaoxianhu`：大额伤害或死亡后增加体力上限并回复；当前仅发布技能定义，卡牌挂载随历史卡牌包迁移。
- `twyuanbu`：手牌上限和双向距离增加其他存活角色数，至多为四，查询时动态重算。
- `twshenxing`：装备区没有坐骑牌时，至其他角色的距离 -1 且手牌上限 +1。
- `twzhenglve3`：旧定义仅为 `charlotte` 临时状态位；保留技能定义但不生成规则索引。
- `old_twrouke`（柔克）：摸牌阶段外一次获得超过一张牌后，锁定摸一张牌；覆盖直接摸牌与区域移动获得牌。
- `jintao`（进讨）：使用【杀】无距离限制且次数上限 +1；出牌阶段第一张【杀】伤害 +1，第二张【杀】不可被响应。
- `twdianyi`（典仪）：自己的回合结束时，造成过伤害则弃置全部手牌，否则由权威选牌屏障将手牌精确调整至四张。
- `twsaotao`（扫讨）：使用【杀】和普通锦囊牌不可被响应。
- `liuli_daxiaoqiao`（流离）：旧 JS 仅声明继承 STANDARD「流离」；本包以自己的稳定技能 ID
  发布相同的弃牌并转移【杀】目标规则，不执行继承脚本。
- `twdaoji`（盗戟）：弃置一张非基本牌，获得攻击范围内目标的一张手牌或装备牌；基本牌额外摸一张，装备牌可用则强制使用并对原持有者造成伤害。
- `twdanji`（单骑）：准备阶段手牌数大于体力值且场上没有刘备主公时觉醒，减一点体力上限并获得【马术】【怒斩】；此后每回合首次使用由实体牌转化的【杀】后摸一张牌。
- `twbaimei`（白眉）：没有手牌时，锁定防止属性伤害或普通锦囊牌造成的伤害。
- `twrangyi`（让驿）：将所有手牌交给一名其他角色；其可使用其中一张并归还其余牌，否则受到一点伤害。
- `zhengfu`（征服）：使用【杀】指定目标后声明一种牌的类别；目标交给你一张同类别牌，否则此【杀】不可被其响应。
- `twfenwu`（奋武）：结束阶段可失去一点体力并使用无距离限制的虚拟【杀】；本回合使用过至少两种基本牌变体时伤害加一。
- `twfupan`（复叛）：造成或受到伤害后按伤害点数摸牌并交出一张牌；对每名角色首次交牌摸二，再次交牌可造成伤害并永久停止向其交牌。
- `twjiaohua`（教化）：自己或体力值最低的其他角色摸牌后，可令其从牌堆或弃牌堆获得一张本次未摸到且本回合未以此法获得过类别的牌。
- `twkujian`（苦谏）：将至多三张手牌交给其他角色并追踪实体牌；其使用或打出时双方摸牌，因其他原因从手牌失去时双方各弃一张牌。
- `twruilian`（睿敛）：每轮选择一名角色，记录其回合内弃置牌的数量和类别；其回合结束时可选择一种记录类别，令其与你依次获得弃牌堆中的一张同类别牌。
- `twfujian`（负剑）：游戏开始或准备阶段无武器时，从牌堆随机装备一张武器牌；回合外失去武器后失去一点体力。
- `twjianwei`（剑威）：准备阶段可双向发起拼点；装备武器时以攻击范围增加自己的拼点点数，胜者按规则获得败者各区域牌或武器。

```bash
cargo run -q -p content-cli -- pack \
  content/original/ported/tw/twjuyan.skill \
  content/original/ported/tw/manji.skill \
  content/original/ported/tw/gx_lingbaoxianhu.skill \
  content/original/ported/tw/twyuanbu.skill \
  content/original/ported/tw/twshenxing.skill \
  content/original/ported/tw/twzhenglve3.skill \
  content/original/ported/tw/old_twrouke.skill \
  content/original/ported/tw/jintao.skill \
  content/original/ported/tw/twdianyi.skill \
  content/original/ported/tw/twsaotao.skill \
  content/original/ported/tw/liuli_daxiaoqiao.skill \
  content/original/ported/tw/twdaoji.skill \
  content/original/ported/tw/twdanji.skill \
  content/original/ported/tw/twbaimei.skill \
  content/original/ported/tw/twrangyi.skill \
  content/original/ported/tw/zhengfu.skill \
  content/original/ported/tw/twfenwu.skill \
  content/original/ported/tw/twfupan.skill \
  content/original/ported/tw/twjiaohua.skill \
  content/original/ported/tw/twkujian.skill \
  content/original/ported/tw/twruilian.skill \
  content/original/ported/tw/twfujian.skill \
  content/original/ported/tw/twjianwei.skill \
  --characters content/original/ported/tw/characters.json \
  --metadata content/original/ported/tw/metadata.json \
  --out content/original/ported/tw/tw.npack.json \
  --package-id tw
```
