# U05 ow 纵切批二十四：D.Va 机甲（规则 v141，2026-10-09）

ow 纵切批二十四：**D.Va**（ow_dva，shu 2 女）——机甲（jijia，标记技
+commit_damage 拦截）单技闭环；**tuijin/zihui/chongzhuang 暂缓**（推
移距离限定子技/自爆 AOE/重装累计复活——均依赖机甲血系子技链）。上游
25 将已闭环第 23 将。前置：
[ow 纵切批二十三](../2026-10-09-ow-sishen-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js jijia）

- init：storage.jijia=4（初始机甲血）。
- changeHp 拦截 forced：机甲血>0 且伤害掉血——体力回补+机甲血同扣
  （观察等价=受伤由机甲血 1:1 抵扣、体力不损；归零 unmark 后照常扣体
  力）。
- mod maxHandcard：机甲血>0 时手牌上限 +storage.jijia（原生 +4=初始
  上限；登记差异：机甲血被打掉后上限不随之递减——上游按剩余值递减，
  原生常数 +4 简化）。

## 改动（分开陈述）

- **代码已实现**：`PlayerState.jijia_hp: u8`（存档 **v14→v15**）+
  `WudianJijia`（tag 140 目录标记技）+ **commit_damage 护甲块后拦截**
  （持技判定 WUDIAN_JIJIA_KEY+机甲血>0 → 1:1 抵扣、体力不损、
  HujiaAbsorbed 日志复用=护甲同型二层吸收）+ 弃牌阶段 limit 钩（机甲
  血>0 时 +4，春饼/克己同位）+ `debug_set_jijia` 测试助手。
- **登记差异**：手牌上限 +4 为常数简化（上游随剩余机甲血递减）；初始
  4 点经由测试助手/授予方设定（上游 init 在武将加载期——生产授予路径
  待开局事件基建）。
- **登记暂缓**：tuijin/zihui/chongzhuang。
- **转换包**：cards.json 增 D.Va（shu 2 女，jijia mapping+三暂缓
  reason）+ npack 资产哈希重算。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`dva_jijia_absorbs_damage_and_hand_
  limit`（机甲血抵扣体力不变 4→3/机甲血>0 时 5 张牌免弃）。
- **全量通过**：engine-native **462 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v140 基线 461 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 140→141 + jijia_hp 零值字段入 canonical_hash——14
  场景全 mismatch，重录后 replay_scenarios 2/2。
- **测试坑（登记）**：无新坑（0 号收束三段既有纪律复用）。

## 剩余

- ow 其余 1 武将（猎空——区域换牌重依赖）逐武将纵切；xianjian 36 武
  将；开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长
  期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（16 文件，相对 noname-next 根）。
