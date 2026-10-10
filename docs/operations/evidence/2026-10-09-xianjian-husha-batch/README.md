# U05 xianjian 第二十八批切片（规则 v169，2026-10-09）

**虚拟 useCard 原语基建落地**（台账待建排序①首项）+ 王小虎
（pal_wangxiaohu，qun 4 男）虎煞（husha_count+husha）全闭环——原语首
个消费者。前置：
[南宫煌摄灵](../2026-10-09-xianjian-sheling-batch/README.md)。

## 基建：use_virtual_card 原语（machine.rs）

上游 game.useCard({name}) 凭空造牌使用。原生以 **use_as 声明+载体实
体**实现（武圣/龙胆 view-as 分派同路径）：从牌堆底取一张实体作载体、
经使用者手牌通道取出、`emit CardUsed { use_as: Some(as_kind) }` 发起
完整使用管线（响应/伤害/弃置全走 use_as 分派=commit effective 分派既
有机制）。**登记差异**：载体实体结算后入弃堆（上游无实体不入弃堆）；
载体取自牌堆底（确定非随机）。

## 上游语义核验（源码为准，…/character/xianjian/skill.js husha）

- `husha_count`（subSkill）：source damageEnd forced——自己出牌回合内
  伤人 +trigger.num 标记（≤3；storage.husha 同口径）。
- `husha`：player phaseEnd direct chooseControl（cancel2 可放弃）——
  1 枚视为使用杀（chooseTarget）；2 枚视为使用南蛮入侵（AOE）；3 枚
  元素毁灭（lib.card.yuansuhuimie 不存在于目录=**登记不转**）。

## 改动（分开陈述）

- **原语**：machine.rs `use_virtual_card(source, as_kind, targets)`。
- **状态**：`PlayerState.husha_mark: u8`（≤3）。存档 v31→**v32**。
- **宿主行为**：`XianjianHushaCount`（tag 188，source Damage forced：
  +伤害量标记，上限钳制）+ `XianjianHusha`（tag 189，PhaseEnd direct
  机内取消：PickTag 模式选择[mark≥1 杀/mark≥2 南蛮]→虚拟杀选目标
  /虚拟南蛮 AOE=use_virtual_card 首例）——六表+ALL 187→189。
- **cards.json**：王小虎行（qun 4 男；husha_count 子技常驻实例化）+
  rehash 两步；**版本** 168→169；a0 追加 188/189。

## 验收口径

- **回归已加入**：g1_wudian 2 例——`wangxiaohu_husha_marks_and_virtual_
  cards`（回合内伤人集标记 2+1=3 上限→Tag0 虚拟杀选目标直伤+扣 1 枚+
  回合推进不再集）+ `wangxiaohu_husha_virtual_nanman`（注入标记 2→双
  选项→Tag1 虚拟南蛮 AOE 1/2 号各-1+扣 2 枚）。
- **全量通过**：engine-native **500 passed / 0 failed / 1 ignored**
  （v168 基线 498 +2）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 168→169 + husha_mark 入 canonical_hash——14 场景全
  mismatch，重录后 replay_scenarios 2/2。
- **测试坑（登记）**：①血量断言链——先前 debug 伤害已扣，南蛮/杀断言
  从当前值算②虎煞询问点=End 阶段末帧（第 3 次 NP）③阶段末触发后回合
  已推进——后续伤害不再集标记（turn.current 门控）。

## 剩余

- 虚拟 useCard 原语后续消费者：仙卿 xiaoyue（还需 roundStart）、姜云
  凡 liefeng 授 tianjian（AOE 已通）、慕容紫英 qianfang（万箭 AOE 同
  款）——按缺口族逐批；xianjian 其余 11/36 将；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next
根）。
