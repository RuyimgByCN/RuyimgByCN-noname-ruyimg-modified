# U05 mtg 第五批：依夏兰（规则 v104，2026-10-08）

mtg 子包第五批转换：依夏兰（♣12 land 地）——换地后选类型随机获得基
本/锦囊牌一张；地图效果=**地附主动技**「重铸」（出牌阶段限一次：一张
基本牌放回牌堆→获得牌堆一张锦囊牌，无锦囊摸一）——首例地附主动技基
建（`sync_land_skill` 换地授予/撤销）。前置：
[mtg 第四批](../2026-10-08-u05-mtg-batch4/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/杀海拾遗/card/mtg/）

- `list.js:1`：`["club", 12, "mtg_yixialan"]`（♣12 一张）；
  `translate.js:3-5`：依夏兰——「选项一：随机获得一张基本牌；选项二：
  随机获得一张锦囊牌。地图效果：出牌阶段限一次，你可以将一张基本牌
  重铸为锦囊牌」。
- `card.js`：content 换地 + chooseControl("基本牌","锦囊牌") →
  `game.createCard(get.inpile(类别).randomGet())` 凭空造牌；
  `skill.js mtg_yixialan_skill`：enable phaseUse/usable 1/filterCard
  basic + `canRecast`/content `recast` 自定义回调——重铸牌逐张从牌堆
  找锦囊 gain，不足 `draw()` 补偿。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Yixialan`（编码 125）+
  `HostBehavior::MtgYixialanLand`（tag 88）两段机（换地 → PickTag 类
  型 → 牌堆随机该类一张入手，羊皮卷类别池 `yangpijuan_class` 复用）+
  `HostBehavior::MtgYixialanRecast`（tag 89）重铸机（选手牌基本牌 →
  放回牌堆 → 获得牌堆一张锦囊/无则摸一补偿）。**地附主动技基建**：
  `sync_land_skill`（host.rs，pub(crate)）——五台既有地机器 Act 首行
  统一调用：撤销旧地附技实例、授予新地附技（当前仅依夏兰附重铸技；
  once_used 随拥有者回合开始重置为既有路径）；重铸 activate 守卫读
  `current_land`（QueryCtx）。once_per_turn 并入 host_skill_def；
  card_kind_host_skills 依夏兰双 decl（地机器 def_key 解析 + 重铸技
  ActivateSkill）；snap 派生两臂。
- **登记差异**：上游 createCard 凭空造牌 → 原生牌堆实体取（随机抽
  取）；`canRecast` 为引擎内置归档不可核验——原生取手牌基本牌；重铸
  放回牌堆顶（上游 recast 洗回同义）；补偿分支（牌堆无锦囊摸一）实现
  同上游、回归未覆盖（构造牌堆无锦囊不可靠）。
- **转换包**：cards.json 增 ♣12 一张（现 5 张）+ 三哈希重算。

## 验收口径

- **回归已加入**：`mtg_yixialan_land_and_recast`——无地无实例
  （NoSuchSkill）→ 出地类型询问（存档往返）→ Tag1 获得锦囊 → 重铸主
  路径（基本牌放回牌堆 + 锦囊入手）→ 限一次（SkillAlreadyUsed）→ 换
  地撤销（出海岛后重铸 NoSuchSkill）。
- **全量通过**：engine-native **420 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 103→104 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（依夏兰不在标准场景，纯版本位移）。
- **剩余**：mtg 其余 7 张地（幽林/沙漠绿洲 + 五镜像）逐批；zhenfa
  14 张；ow/xianjian 武将管线。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（18 文件，相对 noname-next 根）。
