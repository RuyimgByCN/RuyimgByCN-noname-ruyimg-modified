# U05 xianjian 第十七武将切片（规则 v158，2026-10-09）

景天（pal_jingtian，wu 3 男）洒金（sajin）+聚宝（jtjubao）双技闭环，
无暂缓本体。前置：
[xianjian 第十六切片](../2026-10-09-xianjian-jiangcheng-batch/README.md)。

## 候选选择（字节表重建）

剩余 20 将（上游 36 − 已入包 16）按技能总字节排序：pal_xiaoman 1302
（lingquan/shenwu 觉醒链依赖紫萱 shuiyun+回合数+用牌历史计数——非最小，
顺延）、pal_jingtian 1668、pal_xiahoujinxuan 1794（xuanmo createCard
造牌缺口）、pal_jiangyunfan 1894（xunying/liefeng 虚拟用牌+tianjian
链）、pal_xianqing 1991（xiaoyue roundStart 事件缺口+虚拟用牌）——取
**pal_jingtian**。来源勘误登记：v157 批台账写「已入包 17 将」系计数错
误（当时 cards.json 实为 16 将），本批已订正。

## 上游语义核验（源码为准，…/character/xianjian/skill.js sajin/jtjubao）

- `sajin`：enable phaseUse usable:1 filterCard true（手牌）——弃置一
  张手牌并指定任意名角色（filterTarget hp<maxHp=已受伤；selectTarget
  [1,Infinity]）各判定，判定颜色与所弃牌相同者回复 1 点体力。
- `jtjubao`：trigger global discardAfter frequent（自动无确认）——
  其他角色于自己回合外弃非基本牌入弃堆（position "d"）时，随机获得其
  中一张；jtjubao2 临时技=本回合首次门控。

## 改动（分开陈述）

- **状态**：`TurnState.jtjubao_used: bool`（聚宝首收门控——上游
  jtjubao2 临时技等价，换回合随 TurnState 复位）。存档
  v23→**v24**（TurnState 段追加 boolean）。
- **宿主行为**：`XianjianSajin`（tag 167，主动机：SelectCard 发问→
  TargetChoice 收卡+发问→Act 收目标（self.card 兼首入标记）+弃牌+逐
  目标判定（Yield 后重查存活）同色回复；usable:1 走命令层
  once_per_turn）+ `XianjianJtjubao`（tag 168，CardsDiscarded 触发强
  制=上游 frequent 同口径；filter 他人+回合外+未用+含非基本
  （yangpijuan_class!=基本）；cards 臂带入弃牌候选池；Act 随机回收一
  张此刻离弃堆（羊皮卷同款）+置门控）——六表+ALL 166→168。
- **登记差异**：①判定为无改判窗直读（蓝格怪衣 v99 同口径）②聚宝触发
  面=CardsDiscarded 两发射点（弃牌阶段+选择弃置机器路径）——技能代价
  弃置路径（to_discard 直写，不发 CardsDiscarded）不触发，上游
  discardAfter 含全部入弃堆弃置。
- **cards.json**：景天行（wu 3 男）：sajin→host:xj_sajin、
  jtjubao→host:xj_jtjubao（双闭环无暂缓）；rehash 两步重算。
- **版本**：STANDARD_RULESET_VERSION 157→158；a0_tag_registry 追加
  167/168。

## 验收口径

- **回归已加入**：g1_wudian 2 例——`jingtian_sajin_judge_color_
  recover_and_once`（同色判定回复 2→3/满血不可指定/本回合再发动拒绝/
  下回合异色不回复）+ `jingtian_jtjubao_gains_nonbasic_and_gates`
  （他人弃牌阶段弃非基本→自动回收 1 张非基本/门控置位/换回合复位/
  自己弃牌不触发）。
- **全量通过**：engine-native **481 passed / 0 failed / 1 ignored**
  （v157 基线 479 +2）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 157→158 + jtjubao_used 入 canonical_hash——14 场景
  全 mismatch，重录后 replay_scenarios 2/2。
- **测试坑（登记）**：①主动技机器首步（activate 创建）无应答可读——
  首臂只能发问，收应答一律在后续臂（每臂一应答定式的首步特例）②阶段
  推进计数：从 Judge 起**整回合=5 次 NextPhase**（Judge→Draw→Play→
  Discard→End→收束），写 4 次会停在 End（后续 submit NotYourTurn 静默
  失败）③debug_set_deck_top_by 压顶后先被摸走的判定牌即顶——多目标异
  色判定需分回合验。

## 剩余

- xianjian 其余 19/36 将逐武将纵切（xiaoman 觉醒链待 shuiyun/紫萱批）；
  xuanmo（createCard）、xiaoyue（roundStart）、xunying/liefeng（虚拟
  用牌）等已按缺口登记暂缓；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next
根）。
