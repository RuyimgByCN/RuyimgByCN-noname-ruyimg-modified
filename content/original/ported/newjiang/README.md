# NEWJIANG 确定性移植

## 武将

| 武将 | 技能 | 状态 |
|---|---|---|
| `sunli`（孙礼） | `kangli`（伉厉） | 完整单技能武将；造成或受到伤害后摸两张，下次造成伤害前弃置其中始终留在手里的牌 |

## 已移植技能（16）

- `yizu`（义拒）：每回合限一次；已受伤的拥有者成为【杀】或【决斗】目标后，若体力值不大于使用者，锁定回复一点体力。
- `mpbishi`（辟市）：不能成为伤害类锦囊牌的目标。
- `lkzhongzhuang`（忠壮）：逻辑【杀】伤害结算时，攻击范围大于 3 则伤害 +1；攻击范围小于 3 则最终大于 1 的伤害改为 1。
- `dcyiju`（义拒）：其他角色使用仅以你为目标的牌后，若你有手牌或装备牌，强制选择并弃置一张。
- `biancai`（辨材）：每名角色的回合开始时可判定；红色从牌堆、黑色从弃牌堆获得最靠近牌堆顶的一张装备牌。
- `spqishe`（骑射）：手牌或装备区的装备牌可当【酒】使用，手牌上限增加装备区牌数。
- `kangli`（伉厉）：见武将表。
- `weifeng2`：威风“惧”标记的旧客户端展示与 storage 清理身份，无独立规则行为。
- `xinxhzhiyan_used`：知言临时存储清理标记，无独立规则行为。
- `xinxhzhiyan_false`：知言失败后的临时效果；使用牌不能指定其他角色。
- `duanwan`（断腕，吴安国）：限定技，濒死时回复至 2 点体力；限定技以技能标记近似（每局一次），旧 JS 的转换技切换与觉醒分支未建模。
- `beiyu`（备预，司马孚）：出牌阶段限一次，手牌数小于体力上限时摸牌至体力上限；「将一种花色手牌置于牌堆底」分支未建模。
- `dunyong`（遁勇，藤甲）：锁定技。①造成伤害时受等量伤害（镜像伤害来源以自身近似）；②任意角色濒死时摸其体力上限张牌，本回合用牌无次数与距离限制（辅助技能 `dunyong_effect` 随包发布）；「体力不为全场最少/最多」豁免未建模。
- `dunyong_effect`：遁勇②授予的回合级临时效果：使用牌无次数限制且无视距离。
- `mpjiusong`（酒颂，刘玲）：可将一张锦囊牌当【酒】使用；「醉」标记计数与酕醄联动未建模。
- `jingyi`（精艺，马钧）：锁定技。装备置入装备区后摸与装备区牌数等量的牌，然后弃置两张手牌/装备牌；「每类副类别每回合一次」限制未建模，装备区牌数以 5 - 空装备栏近似。

## 打包命令

```bash
CARGO_TARGET_DIR=/private/tmp/noname-next-cargo-target \
cargo run -q -p content-cli -- pack \
  content/original/ported/newjiang/*.skill \
  --characters content/original/ported/newjiang/characters.json \
  --metadata content/original/ported/newjiang/metadata.json \
  --out content/original/ported/newjiang/newjiang.npack.json \
  --package-id newjiang
```

注：`xinxhzhiyan_used` / `xinxhzhiyan_false` 由 `weifeng2.skill` 一并声明，不存在独立文件。
`dckanghui`（亢惠，袁术）：锁定技,受到非自身来源的伤害+1（ABI 562 modify_current_damage）;「双爻标记均未翻转」条件未建模（需妄爻联动）。
