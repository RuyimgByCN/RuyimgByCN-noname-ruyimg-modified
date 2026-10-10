# STANDARD 确定性移植

标准包 M1 canonical pack（`standard_m1.skill`，ABI 261，content_schema_version 3）。
规则、角色和卡牌数据语义重写自 archive/upstream/noname（GNU GPL v3）；未包含外部皮肤、
音频或图片资产，旧 JavaScript 不进入运行时。

## 武将

| 武将 | 技能 | 状态 |
|---|---|---|
| `caocao`（曹操） | `jianxiong`（奸雄）、`hujia`（护驾） | 完整武将；受伤后可获得仍在处理区的伤害实体牌；护驾由主公技框架表达 |
| `simayi`（司马懿） | `fankui`（反馈）、`guicai`（鬼才） | 完整武将；受伤后可从伤害来源处获得一张牌；判定前改判复用判定改写管线 |
| `xiahoudun`（夏侯惇） | `ganglie`（刚烈） | 完整单技能武将；受伤后可判定，非红桃时来源弃两张手牌或受 1 点伤害 |
| `zhangliao`（张辽） | `tuxi`（突袭） | 完整单技能武将；摸牌阶段可改为从一至两名其他角色处各获得一张手牌 |
| `xuzhu`（许褚） | `luoyi`（裸衣） | 完整单技能武将；摸牌阶段可少摸一张，本回合【杀】和【决斗】伤害 +1 |
| `guojia`（郭嘉） | `tiandu`（天妒）、`yiji`（遗计） | 完整武将；判定后可获得最终实体判定牌；每受一点伤害可分配牌堆顶两张牌 |
| `zhenji`（甄姬） | `qingguo`（倾国）、`luoshen`（洛神） | 完整武将；黑色手牌当【闪】；判定阶段可反复判定黑色牌并收入手牌 |
| `liubei`（刘备） | `rende`（仁德）、`jijiang`（激将） | 完整武将；出牌阶段可给其他角色任意张手牌，累计给出两张后回复；激将为蜀势力主公技 |
| `guanyu`（关羽） | `wusheng`（武圣） | 完整单技能武将；红色牌可当【杀】使用或打出 |
| `zhangfei`（张飞） | `paoxiao`（咆哮） | 完整单技能武将；使用【杀】无次数限制 |
| `zhugeliang`（诸葛亮） | `guanxing`（观星）、`kongcheng`（空城） | 完整武将；准备阶段观看牌堆顶牌并分配；无手牌时不能成为【杀】或【决斗】目标 |
| `zhaoyun`（赵云） | `longdan`（龙胆） | 完整单技能武将；【杀】与【闪】互换使用或打出 |
| `machao`（马超） | `mashu`（马术）、`tieji`（铁骑） | 完整武将；与其他角色距离 -1；【杀】指定目标后判定，红色则目标不可响应 |
| `huangyueying`（黄月英） | `jizhi`（集智）、`qicai`（奇才） | 完整武将；使用普通锦囊后摸一张；锦囊牌无距离限制 |
| `sunquan`（孙权） | `zhiheng`（制衡）、`jiuyuan`（救援） | 完整武将；出牌阶段限一次弃任意张牌摸等量；救援为吴势力主公技 |
| `ganning`（甘宁） | `qixi`（奇袭） | 完整单技能武将；黑色牌可当【过河拆桥】使用 |
| `lvmeng`（吕蒙） | `keji`（克己） | 完整单技能武将；未使用或打出【杀】时弃牌阶段可不弃牌 |
| `huanggai`（黄盖） | `kurou`（苦肉） | 完整单技能武将；出牌阶段可反复失去 1 点体力并摸两张牌 |
| `zhouyu`（周瑜） | `yingzi`（英姿）、`fanjian`（反间） | 完整武将；摸牌阶段多摸一张；每阶段一次令目标猜花色并获得自己一张手牌 |
| `daqiao`（大乔） | `guose`（国色）、`liuli`（流离） | 完整武将；方块牌当【乐不思蜀】；成为【杀】目标时可弃一张牌转移给攻击范围内其他角色 |
| `luxun`（陆逊） | `qianxun`（谦逊）、`lianying`（连营） | 完整武将；不能成为【顺手牵羊】或【乐不思蜀】目标；失去最后手牌后摸一张 |
| `sunshangxiang`（孙尚香） | `jieyin`（结姻）、`xiaoji`（枭姬） | 完整武将；弃两张手牌与受伤男性角色各回复 1；失去装备后每张摸两张 |
| `huatuo`（华佗） | `qingnang`（青囊）、`jijiu`（急救） | 完整武将；弃一张手牌令受伤角色回复 1；回合外红色手牌或装备当【桃】 |
| `lvbu`（吕布） | `wushuang`（无双） | 完整单技能武将；【杀】需两张【闪】，【决斗】中对手每轮需两张【杀】 |
| `diaochan`（貂蝉） | `lijian`（离间）、`biyue`（闭月） | 完整武将；出牌阶段弃一张牌令两名男性角色【决斗】；结束阶段摸一张 |
| `huaxiong`（华雄） | `yaowu`（耀武） | 完整单技能武将；受到红色【杀】伤害时来源回复 1 或摸一张 |
| `std_panfeng`（标潘凤） | `stdkuangfu`（标狂斧） | 完整单技能武将；【杀】命中后弃置目标一张装备 |
| `re_yuanshu`（界袁术） | `retongji`（界同疾）、`rewangzun`（界妄尊） | 完整武将；锁定技与主公技变体由既有 modifier 管线表达 |
| `xf_yiji`（新法遗计） | `xinfu_jijie`（新法集解）、`xinfu_jiyuan`（新法救援） | 完整武将；集解/救援为新法变体 |
| `std_yuejin`（标乐进） | `shenji`（神机） | 完整单技能武将 |

## 技能清单（65）

jianxiong、hujia、mashu、tieji、qicao、lijian、biyue、stdxiaoguo、fankui、tiandu、
ganglie、zhiheng、jiuyuan、rende、jijiang、kurou、qingnang、jijiu、jizhi、tuxi、wusheng、
longdan、wushuang、guicai、yingzi、fanjian、lianying、xiaoji、paoxiao、qingguo、luoshen、
jieyin、yiji、qixi、guose、liuli、luoyi、luoyi2、qianxun、keji、guanxing、kongcheng、
feiying、qicai、new_jiangchi、new_jiangchi3、rewangzun、rewangzun2、retongji、xinfu_jijie、
xinfu_jiyuan、shenji、stdqingjiao、yaowu、stdkuangfu、stdshushen、ganglie_three、zhongyi、
zhongyi2、zhanshen、guanxing_fail、jijiu_re_huatuo、longdan_sha_re_zhaoyun、
paoxiao_xiahouba、wusheng_re_guanyu

## 打包命令

```bash
cargo run -q -p content-cli -- pack \
  content/standard/standard_m1.skill \
  --characters content/standard/characters.json \
  --metadata content/standard/metadata.json \
  --out content/standard/standard.npack.json \
  --package-id standard
```

## 测试

```bash
cargo test -p engine-core --test phase_m1_standard_content
cargo test -p engine-core --test phase_m0_unified_skills
```
