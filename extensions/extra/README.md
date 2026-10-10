# EXTRA 确定性移植

- `wuhun`：拥有者死亡时，若凶手仍存活，令凶手失去全部当前体力并进入濒死。
- `xinjuejing`：手牌上限 +2，进入或被救离濒死状态时各摸一张牌。
- `drlt_poxi1`：临时效果存在时，手牌上限 -1。
- `drlt_poxi`：出牌阶段限一次，观看一名其他角色的手牌并可弃置双方手牌中四张花色各异的牌；按自己的弃牌数执行减体力上限、结束出牌阶段并减手限、回复或摸四张牌。
- `drlt_jieying`：维持唯一“营”标记；每名技能拥有者使携营角色摸牌数、杀次数和手牌上限各 +1；可在结束阶段转交，其他携营角色回合结束时由首个结算者获得其全部手牌。
- `drlt_duorui`：出牌阶段对其他角色造成伤害后可废除一个可用装备栏（全部已废除则免除此项），选择其一项可获得的武将技能；该技能失效且夺锐不能再次发动，直到目标下个回合结束或死亡。
- `drlt_zhiti`：攻击范围内每名其他已受伤角色手牌上限 -1；与此类角色拼点或决斗获胜，或受到其伤害后，恢复一个已废除装备栏。
- `changandajian_equip5`：装备技能生效时，手牌上限 +2。
- `juejing`：摸牌数加已损失体力值，手牌上限 +2。
- `wushen`：红桃牌均视为普通【杀】，且无距离、次数限制并不可被响应。
- `new_wuhun`：受到伤害后令伤害来源获得等量“梦魇”；死亡时从梦魇数最多的其他存活角色中选择一名判定，结果不为【桃】或【桃园结义】则其无来源死亡。
- `new_guixin`：每受到一点伤害可选择手牌、装备牌或判定牌的优先级，按座次依次从其他存活角色的首个非空优先区域随机获得一张牌，最后翻面；规则随机数由引擎统一生成。
- `nzry_junlve`：每造成或受到一点伤害，获得一枚稳定 ID 的“军略”标记。
- `nzry_cuike`：出牌阶段开始时按“军略”奇偶选择造成伤害，或横置目标并弃置其区域内一张牌；超过七枚时可清空并依次伤害所有其他角色。
- `nzry_dinghuo`：限定技，清空“军略”并选择不超过原标记数的横置角色，按座次弃置其全部装备，再选择其中一名造成火焰伤害。
- `shelie`：摸牌阶段改为公开牌堆顶五张，获得每种花色各一张，其余牌进入弃牌堆。
- `gongxin`：出牌阶段限一次，观看一名其他角色的手牌，可选择其中一张红桃牌弃置或公开置于牌堆顶。
- `yeyan`：限定技；小业炎对一至三名角色各造成一点火焰伤害，大业炎弃置四种花色手牌并失去三点体力，对一至两名角色分配共三点火焰伤害。
- `qinyin`：弃牌阶段结束时，若本阶段弃置至少两张牌，可令所有角色各失去一点体力或各回复一点体力。
- `relonghun`：可将一至两张同花色牌按花色转化为火【杀】、【桃】、【闪】或【无懈可击】；两张红牌令伤害/回复值 +1，两张黑牌在响应后弃当前回合角色一张牌。
- `qixing`：开局将牌堆顶七张牌作为公开“星”，并可在开局及摸牌阶段结束后与手牌等量交换。
- `kuangfeng`：结束阶段可弃一枚“星”并选择一名角色；直到来源的下回合开始，该角色受到的火焰伤害 +1。
- `dawu`：结束阶段可弃任意枚“星”并选择等量角色；直到来源的下回合开始，防止这些角色受到的非雷电伤害。
- `nzry_longnu`：每个出牌阶段切换分支；首次失去一点体力并摸一张牌，本阶段红色手牌视为无距离限制的火【杀】；下次减一点体力上限并摸一张牌，本阶段锦囊牌视为不限次数的雷【杀】。
- `nzry_jieying`：开局横置且不能重置；所有横置角色手牌上限 +2；结束阶段强制横置一名其他未横置角色，多名拥有者的全局加成不叠加。
- `baonu`：游戏开始时获得两枚“暴怒”，之后每造成或受到一点伤害再获得一枚。
- `wumou`：使用普通锦囊后选择移去一枚“暴怒”或失去一点体力；无标记时只能失去体力。
- `ol_wuqian`：每个目标每回合限一次，移去两枚“暴怒”并获得标准包【无双】直到本回合结束。
- `ol_shenfen`：出牌阶段限一次，移去六枚“暴怒”，按座次逐人完成伤害、弃装备、弃四张手牌，最后翻面。
- `renjie` / `renjie2`：受到伤害后按伤害点数获得“忍”；弃牌阶段结束时按本阶段弃置的手牌/装备牌数获得“忍”。
- `sbaiyin` / `jilue`：准备阶段以至少四枚“忍”觉醒，减一点体力上限并获得鬼才、放逐、集智、制衡、完杀五个分支。
- `lianpo`：一回合内杀死过角色后，可在该回合完整结束后插入一个额外回合；结束后回到原回合锚点的正常下家。

```bash
cargo run -q -p content-cli -- pack \
  content/original/ported/extra/wuhun.skill \
  content/original/ported/extra/xinjuejing.skill \
  content/original/ported/extra/drlt_poxi1.skill \
  content/original/ported/extra/drlt_poxi.skill \
  content/original/ported/extra/drlt_duorui.skill \
  content/original/ported/extra/changandajian_equip5.skill \
  content/original/ported/extra/juejing.skill \
  content/original/ported/extra/wushen.skill \
  content/original/ported/extra/new_wuhun.skill \
  content/original/ported/extra/new_guixin.skill \
  content/original/ported/extra/nzry_junlve.skill \
  content/original/ported/extra/nzry_cuike.skill \
  content/original/ported/extra/nzry_dinghuo.skill \
  content/original/ported/extra/shelie.skill \
  content/original/ported/extra/gongxin.skill \
  content/original/ported/extra/yeyan.skill \
  content/original/ported/extra/qinyin.skill \
  content/original/ported/extra/relonghun.skill \
  content/original/ported/extra/qixing.skill \
  content/original/ported/extra/nzry_longnu.skill \
  content/original/ported/extra/baonu.skill \
  content/original/ported/extra/wumou.skill \
  content/original/ported/extra/ol_wuqian.skill \
  content/original/ported/extra/ol_shenfen.skill \
  content/original/ported/extra/shen_simayi.skill \
  --characters content/original/ported/extra/characters.json \
  --metadata content/original/ported/extra/metadata.json \
  --out content/original/ported/extra/extra.npack.json \
  --package-id extra
```
