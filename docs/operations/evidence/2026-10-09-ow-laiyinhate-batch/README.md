# U05 ow 纵切批十七：莱因哈特猛击（规则 v134，2026-10-09）

ow 纵切批十七：**莱因哈特**（ow_laiyinhate，qun 4 男）——猛击（mengji，
伤害前置窗口触发锁定）单技闭环；**zhongdun 重盾暂缓**（init2 开局 +8
护甲——无 GameStart 事件种类=开局事件基建缺口；主动转甲技部分原语已
备：change_hujia×2+弃牌+目标选择）。mengji 的 filter
`storage.zhongdun` 置位以**持技等价**（登记）。上游 25 将已闭环第 16
将。前置：
[ow 纵切批十六](../2026-10-09-ow-kuangshu-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js mengji）

- trigger source damageBegin forced unique：filter storage.zhongdun &&
  !player.hujia && event.card 为杀 && notLink → num++。

## 改动（分开陈述）

- **代码已实现**：`WudianMengji`（tag 131，EventKind::DamageBefore 前
  置窗口触发、optional:false=上游 forced 同口径）——filter（来源=持
  有者 && 持有者无护甲 && 未防止 && 非传导（from_chain 字段，源氏同口
  径）&& 杀实体牌口径）→ Act：`current_damage_mut().amount += 1`（前
  置窗口直写帧量——伊籍改目标同机制；上游 trigger.num++ 同口径）。
- **登记等价**：上游 filter 的 storage.zhongdun（重盾 init2 置位）以
  持技等价——莱因哈特装配即视为已初始化。
- **登记暂缓**：zhongdun（开局甲=GameStart 事件基建缺口+主动转甲）。
- **测试基建**：新增 `Engine::debug_set_hujia`（debug_set_hp 先例，
  测试侧护甲直设；此前测试只读 hujia——save().core 途径）。
- **转换包**：cards.json 增 莱因哈特（qun 4 男，mengji mapping+
  zhongdun 暂缓 reason）+ npack 资产哈希重算。无新状态字段——存档
  v9 不变。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`laiyinhate_mengji_no_armor_bonus`
  （无甲杀 +1（4→2）/有甲不加成（4→3））。
- **全量通过**：engine-native **455 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v133 基线 454 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 133→134 入 canonical_hash——14 场景全 mismatch
  （纯版本位移），重录后 replay_scenarios 2/2。
- **测试坑（登记）**：无距离豁免（第二张杀打距离 2 被拒——改距离 1）；
  测试侧此前无护甲 setter（新增 debug_set_hujia）。

## 剩余

- ow 其余 9 武将逐武将纵切；xianjian 36 武将；开局事件基建（zhongdun
  init2 类）、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next 根）。
