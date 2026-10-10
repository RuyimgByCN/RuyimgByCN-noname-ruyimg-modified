# U05 swd 第十四批：带火风轮+霞云彩绫距离对（规则 v98，2026-10-08）

swd 子包第十四批转换：命名距离修饰两张（同原语成对）——带火风轮
（equip4）：进攻距离 +2、防御距离 -1；霞云彩绫（equip3）：进攻距离
-1、防御距离 +2。前置：
[swd 第十三批](../2026-10-08-u05-swd-batch13/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/杀海拾遗/card/swd/）

- `list.js:13`：`["heart", 1, "daihuofenglun", "fire"]`（♥1，fire 属性
  标记）；`list.js:15`：`["diamond", 2, "xiayuncailing"]`（♦2）。
  translate.js:288-295：带火风轮「你的进攻距离+2，你的防御距离-1」、
  霞云彩绫「你的进攻距离-1，你的防御距离+2」。
- `card.js:1621/1633`：daihuofenglun type equip/subtype equip4/
  `distance: { globalFrom: -2, globalTo: -1 }` + cardnature "fire"；
  xiayuncailing equip3/`distance: { globalFrom: 1, globalTo: 2 }`。
  **两者均无 skills 字段**——纯距离数据卡，无机器。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Daihuofenglun`（编码 118）/`Xiayuncailing`
  （编码 119）（archive enc/dec、a0 variant/ALL 120 同步）+ 装备栏
  equip4→HorsePlus、equip3→HorseMinus + `computed_distance` 扩展
  （state.rs）：攻方持有风轮 -2、守方持有风轮 -1（globalFrom/globalTo
  负号并入 plus/minus 累加，饱和下限 1 为既有路径）；攻方持有彩绫 +1、
  守方持有彩绫 +2。装配路径走 is_equip 提前分支（槽位/同 kind 替换为
  批②既有定案）；出牌区使用为穷尽性占位拒绝（并入风雪刃/御夫录臂）。
- **登记差异**：cardnature "fire" 仅为上游属性/主题标记（带火风轮非
  权属性伤害来源），原生 cards.json 无该字段、不入结算；两卡不与
  HorsePlus/HorseMinus 基础马做同栏跨 kind 替换（批②装备命名化取舍
  既有定案，本批沿用）。
- **转换包**：cards.json 增 ♥1/♦2 两张（现 24 张）+ source_hash/
  logic_hash/asset_hash 三哈希重算。

## 验收口径

- **回归已加入**：`swd_daihuo_xiayun_distance_pair`——6 人局对座
  （基础距离 3）：无装备基线 → 带火风轮攻侧 -2（3→1）/守侧 -1
  （3→2）/邻座下限饱和（1-2→1）→ 装配路径出牌霞云彩绫 → 双持双向
  （0→3：3-2+1=2；3→0：3-1+2=4）→ 装备区在场/离手断言。
- **全量通过**：engine-native **413 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 97→98 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（两卡不在标准场景，纯版本位移）。
- **发牌敏感**：+2 张后风雪刃注入测试 card_in_multiple_zones——补全
  区域 purge（既有陷阱清单同口径）；距离矩阵参照模型（native_kernel
  `equip_distance` 全对全）不受影响（两卡仅经转换包入场）。
- **剩余**：swd 其余卡种（蓝格怪衣 ♦1 判定四花色分支等）逐批转换；
  ow/xianjian/mtg/zhenfa 同前置。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（17 文件，相对 noname-next 根）。
