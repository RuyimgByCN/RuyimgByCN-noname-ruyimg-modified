# U05 mtg 第二批：海岛（规则 v101，2026-10-08）

mtg 子包第二批转换：海岛（♦5 land 地）——换地后满血直接 +1 护甲；
受伤则弃一张（he，可取消）回 1 体力或取消得 1 护甲（temp 抑制转换）；
地图效果全局锁定：获护甲时若已受伤改为回复等量体力。前置：
[mtg 首切片](../2026-10-08-u05-mtg-slice/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/杀海拾遗/card/mtg/）

- `list.js:6`：`["diamond", 5, "mtg_haidao"]`（♦5 一张）；
  `translate.js:15-18`：海岛——「选项一：获得1点护甲（无视地图效果）；
  选项二：弃置一张牌并回复1点体力。地图效果：锁定技，当你获得护甲
  时，若你已受伤，改为回复等量体力」。
- `card.js:240-280`：content 换地 → `isHealthy()` 满血 `changeHujia()`
  收尾；受伤分支 `_temp_mtg_haidao = true`（抑制转换）+
  `chooseToDiscard("he","海岛")` 可取消——弃则 `recover()`、取消则
  `changeHujia()`，step 2 清 temp。`mtg_haidao_skill`：
  trigger changeHujiaBefore + filter isDamaged && !temp && num>0 →
  `recover(trigger.num); trigger.cancel()`（经 addGlobalSkill 全局生效）。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Haidao`（编码 122）+
  `HostBehavior::MtgHaidaoLand`（tag 85，HostBehavior::ALL 85 守卫过）
  两段机：Act 换地（current_land/land_used）→ 满血 `change_hujia(1)`
  （不受伤转换不触发）/ 受伤问弃一张（he=手牌∪装备区，可取消）→
  SelectCard 弃（手牌直弃/装备离装撤销技）则 `recover(1)`、取消直写
  护甲字段（上游 `_temp` 标记=路径级抑制口径）+ kernel intrinsic 臂
  （无无懈窗口）+ PlayCard 地臂并入（Longlushanfeng|Haidao）+ snap
  派生臂。**地图效果落在 `change_hujia` 原语**（machine.rs）：正增量
  且 current_land=海岛且目标已受伤 → `recover(delta)`（全局口径，
  紫阳丹/盾牌格挡等全部加甲路径统一转换）。
- **登记差异**：上游 chooseToDiscard 的取消经 result.bool 分支，原生
  以空选择（allow_cancel 既有口径）表达；temp 抑制以「取消路线直写
  字段」实现（路径级而非事件级）；其余无。
- **转换包**：cards.json 增 ♦5 一张（现 2 张）+ 三哈希重算。

## 验收口径

- **回归已加入**：`mtg_haidao_land_effect`——①满血直接 +1 护甲无询问
  ②受伤弃一张回 1（存档往返；弃牌候选询问前布阵入选项）③受伤取消得
  1 护甲（不转换）④换地覆盖（龙颅山峰→海岛跨回合：地持续/每回合限
  一张地/覆盖后杀加成消失——第二张杀即拒）⑤地图效果全局（海岛在场
  时受伤玩家盾牌格挡获甲转为回血，护甲不增；mtg+hearth 双包同局）。
- **全量通过**：engine-native **417 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 100→101 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（海岛不在标准场景，纯版本位移）。
- **剩余**：mtg 其余 10 张地（依夏兰/水没墓穴/废墟/幽林/沙漠绿洲 +
  五镜像）逐批；zhenfa 14 张；ow/xianjian 武将管线。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（17 文件，相对 noname-next 根）。
