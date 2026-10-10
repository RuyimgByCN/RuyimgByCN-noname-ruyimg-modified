# 英雄杀（yxs）归档扩展迁移

- 来源：`archive/upstream/noname/apps/core/extension/英雄杀`（GNU GPL v3）。
- 本包登记全部 43 名武将（yxs 包名），技能 105 项（含辅助）；依赖 standard 与
  shenhua 包（`mashu/wushuang/jianxiong/longdan/keji/xiaoji/jieyin` 复用 standard
  定义，`qiangxi/kanpo` 复用 shenhua 定义）。

## 已发布（真实运行时）

- `yxs_fanji`（反击，秦琼）：受伤害后可对来源使用虚拟【杀】。
- `yxsweiyi`（威仪，武则天）：受伤害后令来源弃置两张手牌/装备牌。
- `tuqiang`（图强，勾践）：使用或打出【闪】后摸一张牌。
- `taiji`（太极，张三丰）：使用或打出【闪】后可使用一张虚拟【杀】。
- `xumou`（蓄谋，吕雉）：结束阶段摸三张牌并翻面。
- `taolue`（韬略，孙武）：手牌上限+1。
- `wumu`（武穆，岳飞）：黑色【杀】无视距离、红色【杀】不计入出杀限制。
- `wluoyan`（落雁，王昭君）：防止受到的伤害，改为失去 1 点体力。
- `xiangma`（相马，伯乐）：体力大于 2 时进攻距离+1；体力不大于 2 时防御距离+1。
- `fengyi`（凤仪，貂蝉）：出牌阶段限一次弃一张手牌，令任意角色摸两张。
- `zhijie`（知节，小乔）：红桃手牌当【无中生有】使用，每回合限一次。
- `huli`（护理，南丁格尔）：红桃手牌当【桃】使用（距离 1 限制未建模）。
- `sheshu`（射术，罗宾汉）：【杀】无视距离（体力不小于 3 的目标不可闪避未建模）。
- `chujia`（初嫁，小乔）：弃两张同色手牌，令已受伤角色摸其已损体力值张牌。
- `yixin`（医心，南丁格尔）：限定技，弃两张牌，令已受伤角色回复至多 4 点并补摸。
- `zbudao`（布道，张三丰）：摸牌阶段额外摸一张（交给他人未建模）。
- `luobi`（落笔，唐伯虎）：结束阶段摸已损体力值张牌（分配未建模）。
- `guifu`（鬼斧，鲁班）：弃目标装备区一张牌，双方各摸一张。
- `qiangyun`（强运，朱元璋）：失去最后一张手牌后摸两张。
- `zhexian`（谪仙，李白）：回合外失去牌后摸一张（每回合限一次未建模）。

## 元数据登记（无运行时行为）

其余技能依赖目标替换（门神）、标记生命周期（浴血）、动态技能复制（易装/统御/
归隐）、判定分支（刺客/强掠/演绎）、全局用牌限制（独裁）、座次交换（迁都）、
多目标分支（色诱/徭役/三板斧）、摸牌联动（布道给牌/落笔分配/霓裳）等 DSL 未移植
机制，登记元数据并文档化近似，详见 `yxs_meta.skill` 各技能注释。

## 可表达性缺口（未移植机制）

- 事件目标替换（门神：杀/决斗目标改为秦琼）
- 标记生命周期与清空规则（浴血）
- 动态技能复制/技能组切换（易装、统御、归隐）
- 判定分支（刺客、强掠、蛇噬、演绎）
- 全局花色使用禁用（独裁）
- 座次交换（迁都）
- 濒死即死（弑亲、鸩杀、刺酋）
- 按点数/花色/性别的目标约束（先驱、女权、丰艳）
- 摸牌结果联动（布道给牌、落笔分配、霓裳花色计数、婉歌取牌）

```bash
cargo run -q -p content-cli -- pack \
  content/original/ported/yxs/yxs_impl.skill \
  content/original/ported/yxs/yxs_meta.skill \
  --characters content/original/ported/yxs/characters.json \
  --metadata content/original/ported/yxs/metadata.json \
  --out content/original/ported/yxs/yxs.npack.json \
  --package-id yxs
```
