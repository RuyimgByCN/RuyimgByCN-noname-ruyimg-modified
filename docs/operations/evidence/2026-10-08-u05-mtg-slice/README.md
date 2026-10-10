# U05 mtg 首切片：龙颅山峰（规则 v100，2026-10-08）

杀海拾遗 mtg 子包（万智牌联动 12 张，全部为新牌类 **land 地**）首切片：
新转换包 shahai-mtg-upstream-noname + 龙颅山峰（♣10）——使用时换地 +
二选一获得杀/闪；地图效果全局出杀次数上限 +1。子包来源级第七例
（gwent/古剑/炉石/运筹/wuxing/swd 之后）。前置：
[swd 第十五批](../2026-10-08-u05-swd-batch15/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/杀海拾遗/card/mtg/）

- `list.js`：12 张 land（♣12 依夏兰/♠11 水没墓穴/♦5 海岛/♣10 幽林/
  ♣8 废墟/♥6 沙漠绿洲 + 六张镜像），type.js `land: 0.6`。
- `game.changeLand`（upstream noname game/index.js:1674）：UI 覆盖 +
  `game.addGlobalSkill(skill)`（**全场生效**）+ 使用者
  `addTempSkill("land_used")`（本回合不能再出地）；新地替换旧地。
- `card.js` 龙颅山峰（translate.js:40 **龙颅山峰**，此前工作记录误作
  「龙颅山脊」）：content 换地 + chooseControl「获得杀/获得闪」→
  `game.createCard("sha"/"shan")` 凭空造牌；`mtg_longlushanfeng_skill`
  = mod cardUsable（杀）+1。
- **来源勘误**：台账所记 ow/xianjian 为 `character/`（武将）子目录，
  非卡牌子包；mtg/zhenfa 才是卡牌子包。

## 改动（分开陈述）

- **代码已实现**：land 类基建——`CardKind::Longlushanfeng`（编码 121）
  + `GameStateCore.current_land: Option<CardKind>`（换地覆盖；跨回合
  持续；入存档/哈希）+ `TurnState.land_used/land_sha_used`（回合级门
  与额外杀标记；换回合重置——上游 land_used 临时技同口径，仅当前回合
  玩家可出牌故回合级布尔等价）+ PlayCard 地臂（无目标 + 本回合限一张）
  + kernel intrinsic 地机器（无无懈窗口，地非锦囊）+
  `HostBehavior::MtgLongshanfengLand`（tag 84）两段机：Act 换地+置
  land_used → 问获得杀/闪（PickTag 0/1，上游 chooseControl 顺序）→
  SelectCard 从牌堆取一张该类牌入手 + card_kind_host_skills 注册
  （存档 def_key 解析用；kernel 直接建机不经 ActivateSkill）+ 杀次数
  门/提交扩展（有地时第二张杀放行并置 land_sha_used；提交仅在常规配额
  已用时才标记额外杀）+ snap 派生臂 + 全套 archive/哈希接线。
- **登记差异**：上游 createCard 凭空造牌 → 原生从牌堆实体取（羊皮卷
  同口径）；「获得杀/闪」取牌堆内首张该类（确定性）而非上游 cardPile
  顶侧抽取；地图效果全局（全场出杀上限 +1）与上游 addGlobalSkill 同
  口径；专用 LandChanged 日志暂缓（登记，状态可观察）。
- **转换包**：`packages/extensions/shahai-mtg-upstream-noname`（♣1 张；
  npack 按 swd 模板，三哈希重算；学习项：asset_manifest.package_id 须
  与 manifest.package_id 一致、pack 须为对象字段）。

## 验收口径

- **回归已加入**：`mtg_longshanfeng_land_flow`——出地→获得询问挂起
  （current_land 已换；存档往返恢复同一询问）→ 获得杀（手牌 +1 杀）→
  本回合第二张地拒绝 → 全局杀上限 +1（两杀放行第三张 ShaLimit 拒）→
  换回合地持续；`mtg_real_host_full_lifecycle` 六步（安装+激活→建局→
  对局中 revoke_slot 卸载→锁定副本续玩→重装 Revoked→import 被拒）。
- **全量通过**：engine-native **416 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 99→100 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（mtg 不在标准场景，纯版本位移）。
- **剩余**：mtg 其余 11 张地逐批；zhenfa 14 张（阵法类）；ow/xianjian
  武将包（character 管线，非卡牌通道）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（19 文件，相对 noname-next 根）。
