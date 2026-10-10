# U05 xianjian 第十一切片：玄霄玄炎+凝冰（规则 v152，2026-10-09）

xianjian 第十一武将切片：**玄霄**（pal_xuanxiao，qun 4 男）——玄炎
（xuanyan group：燃 xuanyan2+烬 xuanyan3）+凝冰（ningbin）三机闭环；
**xfenxin 暂缓**（changeHp 事件——掉血/回复增量抽牌，原生无 changeHp
事件种，jijia 用 commit_damage 拦截替代口径不同源）。xianjian 包 11/36
将入包（11 闭环）。前置：
[xianjian 第十切片](../2026-10-09-ow-lixiaoyao-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js
xuanyan/xuanyan2/xuanyan3/ningbin）

- `xuanyan`：locked group 壳（实际触发在 xuanyan2/3）。
- `xuanyan2`：source damageBegin forced——hasNature("fire") && notLink →
  num++（火伤 +1 非传导=飞刃同款）。
- `xuanyan3`：source damageEnd forced——hasNature("fire") → loseHp()。
- `ningbin`：player damageEnd forced——hasNature("thunder") → recover()。

## 改动（分开陈述）

- **代码已实现**：`XianjianXuanyan2`（tag 155，DamageBefore forced sou-
  rce scope）——火伤+1（非传导=from_chain 字段，飞刃同款）；`Xianjian-
  Xuanyan3`（tag 156，Damage after forced source scope）→ Act：emit Lo-
  seHp 1（LoseHp 独立事件=返灵同位）；`XianjianNingbin`（tag 157，Damage
  after forced target scope）→ Act：回复 1（满血饱和）；`XianjianXuany-
  an`（tag 158 目录标记壳 group——tianshe 系先例，xuanyan2/3 各自 map-
  ping 独立实例）。
- **登记差异**：无（三机一一对应；火/雷/传导口径与飞刃/拾火一致）。
- **登记暂缓**：xfenxin（changeHp 事件缺口）。
- **转换包**：cards.json 增 玄霄（qun 4 男，四技 mapping+xfenxin 暂缓
  reason）+ npack 资产哈希重算。无新状态字段——存档 v18 不变。
- **编译守卫**：Xuanyan 标记壳两处穷尽 match（filter/step）漏臂——E0004
  暴露后补 `=> false`/`=> Flow::Done`。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`xuanxiao_xuanyan_ningbin_fire_
  thunder`（火杀+1 6→4+烬失体力 4→3/普通杀不加成不失体力 3→2）。
- **全量通过**：engine-native **473 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v151 基线 472 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 151→152 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。

## 剩余

- xianjian 其余 25 武将逐武将纵切；ow 猎空 deferred（离场区基建立项）；
  开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
