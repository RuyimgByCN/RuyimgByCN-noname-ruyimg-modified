# U05 xianjian 第二十五批切片（规则 v166，2026-10-09）

柳梦璃（pal_liumengli，shu 3 女）润心（runxin）单技闭环；tianxian/
xjzhimeng 缺口族登记暂缓。前置：
[雪见切片](../2026-10-09-xianjian-xuejian-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js runxin/tianxian/xjzhimeng）

- `runxin`：player ["useCard","respondEnd"] direct——使用/打出红桃牌
  （get.suit=="heart" 仅红桃）→ chooseTarget 一名 hp<maxHp 角色 →
  recover 1。
- `tianxian`：mod targetInRange（杀无距离）/selectTarget（目标数
  Infinity）+useCardToBefore 逐目标判定黑取消——mod 系+逐目标窗口复
  合缺口暂缓。
- `xjzhimeng`：phaseEnd 暗置牌堆顶一张存目标 storage+xjzhimeng2
  （useCardToBegin 同类型消耗）——跨玩家持牌暗标记缺口暂缓。

## 改动（分开陈述）

- **宿主行为**：`XianjianRunxin`（tag 184，TriggerMeta **extra_events
  双事件首例**：event=CardUsed+extra CardResponded=废墟双事件同族）——
  filter（使用/打出红桃+存在已受伤者）→ Act 发问（已受伤目标，取消=
  放弃）→ TargetChoice recover 1。六表+ALL 183→184。无新状态字段。
- **登记暂缓**：tianxian（mod 系+逐目标判定取消）、xjzhimeng（暗牌标
  记）。
- **cards.json**：柳梦璃行（shu 3 女）+ rehash 两步；**版本** 165→166；
  a0 追加 184。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`liumengli_runxin_heart_use_and_
  respond`（使用红桃无中→选受伤者回复；1 号回合内杀 0 号、0 号红桃闪
  响应打出→润心再触发）。
- **全量通过**：engine-native **496 passed / 0 failed / 1 ignored**
  （v165 基线 495 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 165→166 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。
- **测试坑（登记）**：①满血不可用桃（HpFull）——红桃布阵用无中（不
  限体力）②响应段导航：跨回合后须先推到响应者回合。

## 剩余

- xianjian 其余 12/36 将（余将缺口族：虚拟用牌[姜云凡/仙卿/王小虎/
  罗招嫣/慕容紫英 qianfang]、createCard[夏侯瑾轩/苏媚 dujiang]、
  moveCard/dieBefore[月琦]、响应链[云天河 longxi]、事件面[南宫煌]、
  暗牌标记[柳梦璃 xjzhimeng]、mod 系[柳梦璃 tianxian/姜云凡 liefeng]）；
  长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（10 文件，相对 noname-next
根）。
