# U07 技能批四：抗yin（留赞）（规则 v114，2026-10-08）

玩点论杀武将技能声明批四：**抗yin**（kangyin）映射
`host:wudian_kangyin`——出牌阶段限一次：失去 1 点体力 → 弃目标一张
牌（被弃者自选）→ 已受伤时按弃牌类别分支：基本=选至多 X 名角色各摸
一 / 非基本=授 kangyin2（到他人距离-X、杀额外目标 +X，X=发动时
maxHp-体力，回合内）。未映射声明 6→5。前置：
[U07 技能批三](../2026-10-08-u07-skill-batch3/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/玩点论杀/character/skill.js:344）

- `kangyin`：enable phaseUse、usable 1、filterTarget 其他角色 he>0；
  content：loseHp → discardPlayerCard(target, true)（被弃者自选）→
  isDamaged 时按弃牌 get.type 分支——basic：chooseTarget
  [1, maxHp-hp] 名各摸一；否则 storage.kangyin2=maxHp-hp +
  addTempSkill("kangyin2")。
- `kangyin2`：mod globalFrom（到他人距离-X）+ selectTarget（杀额外目
  标上限 +X）；mark 到回合结束。

## 改动（分开陈述）

- **代码已实现**：`WudianKangyin`（tag 101，HostBehavior::ALL 101 守
  卫过）四段机（目标 → loseHp 同步提交 + 被弃者自选弃 → 分支：基本
  CardsAsk 多选各摸一（min1 max=X）/ 非基本授 `TurnState.kangyin2:
  Option<(PlayerId, u32)>`）+ `computed_distance` 抗yin2 距离项（minus
  转 mut）+ 方天臂杀额外目标上限 +X（kangyin_extra）+ once_per_turn +
  activate 守卫 + snap 派生三段。`TurnState.kangyin2` 入存档/哈希（对
  象三元组 enc/dec）/换回合清空全套。mapping 接线（留赞 kangyin）。
- **实现要点**：失去体力改**同步提交**（`commit_lose_hp` pub(crate)）
  ——异步子事件会让分支读到旧体力判定 isDamaged。
- **登记差异**：上游 discardPlayerCard(target, true) 被弃者自选（同
  口径）；kangyin2 上游为 mark+双 mod——原生 computed_distance 状态项
  +方天同款杀上限扩展；无专用日志（登记）。
- **转换包**：cards.json 留赞 kangyin mapping（skills 段登记），三哈
  希重算。

## 验收口径

- **回归已加入**：`liuzan_kangyin_losehp_discard_and_branch`——发动→
  失去 1 体力（2→1）→ 被弃者弃基本牌 → 基本分支多选各摸 1 → 限一次
  （SkillAlreadyUsed）→ kangyin2 状态通道：杀双目标放行+两目标各掉血。
- **全量通过**：engine-native **431 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 113→114 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（抗yin 不在标准场景，纯版本位移）。
- **剩余**：U07 未映射 5 条（夺启暂缓+绝道/割据/车阵/诱战）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（17 文件，相对 noname-next 根）。
