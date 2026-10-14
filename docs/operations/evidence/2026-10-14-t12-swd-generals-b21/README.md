# T12 swd 武将面批 21：流虹/墨剑/道法三技四将部分转正（2026-10-14）

## 承载

三技 HostBehavior（tag 272/273/274，键数冻结表 262→**274**）：

- **流虹 swdliuhong**（拓跋玉儿）：使用杀（CardUsed+is_sha_like）后频率技
  摸 1，无询问窗（上游 `frequent: true`）。
- **墨剑 mojian**（水镜/屈娴）：**自己使用杀**指定目标后确认（可选，确认
  面仅 [Confirm]+allow_cancel），杀的首个目标摸 1、自己回 1（未满血门）。
  上游触发为 `player: "shaBegin"`，原生等价接入 CardUsed 帧（与既有转化批
  同口径）。**勘误**：前轮批 21 设计稿曾误记为「持有者成为杀目标」——勘察
  上游 skill.js:3536 后改正 filter/extract 方向（本批实测修正）。
- **道法 daofa**（壶中仙）：其他角色造成伤害（Damage 结算后，global）且来
  源有手牌时确认，来源弃 1 张手牌。**登记差异**：上游 `chooseToDiscard`
  来源自选弃哪张，原生确定性弃首张手牌。

## 武将名勘误

前轮批次计划消息中的「拓跋月儿/曲贤/胡忠仙」系勘察转写错误，上游
character/translate.js 实名为**拓跋玉儿**（swd_tuobayuer）/ **屈娴**
（swd_quxian）/ **壶中仙**（swd_huzhongxian）；intake survey 中的「流虹=
拓跋月儿」同此勘误（原始勘察文档按日期语境保留，不回改）。

cards.json generals 7→**11**（拓跋玉儿 shu hp4 / 水镜 qun hp4 / 屈娴 qun
hp3 / 壶中仙 wu hp3，未转正技 `mapping: null`）；npack 双哈希重算
（rehash_cards_asset 资产 source_hash + rehash_npack manifest）。

## 事故与修复（过程实录）

1. 前会话接线脚本在 tag()/from_tag() 各留下重复臂（unreachable pattern）；
   本批去重并补齐枚举/key()/from_tag()/ALL/trigger_meta/提取/filter/step
   九触点齐验（grep 逐技 8 处计数）。
2. **批 18–20 遗留失同步**：lib.rs 冻结键数仍为 262，而批 18/19/20 已把
   ALL 增至 271——批 20 证据日志内即含该断言失败实录（README 记 749/0/0
   与日志矛盾）。本批按实测同步为 274，`host_behavior_keys_bijective` 绿。
3. 测试布阵四轮勘误：同回合第二张杀被「杀限一次」拒绝（补 sha_used 快照
   重置）；拒绝窗=空选择取消（确认面仅 [Confirm]，无 Pass 项）；②b 布阵
   p1 已受三杀再中杀即死亡（死亡清手致摸牌断言失败，改前置回满）；③ 段
   未撤销①②所授流虹/墨剑致窗口归 p0 与手牌计数扰动（快照 remove 两技）。

## 测试（swd_generals_b21_liuhong_mojian_daofa，含反例 6 项）

| 面 | 断言 |
|---|---|
| 流虹 | 用杀自动摸 1（用 1 摸 1 净持平）+ 无询问窗 |
| 流虹反例 | 用桃不摸 |
| 墨剑 | 受伤确认 → 目标摸 1、自己回满 |
| 墨剑反例 a | 拒绝（空选择取消）→ 目标不摸、自己不回 |
| 墨剑反例 b | 满血确认 → 目标仍摸、回复饱和不溢出 |
| 道法 | 确认窗归持有者 p3 → 来源弃首张手牌 |
| 道法反例 a | 拒绝 → 来源保留手牌 |
| 道法反例 b | 来源=持有者不触发 |
| 道法反例 c | 来源空手不触发 |

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **32/32**（批 20 31/32 基础上 +1） |
| engine-native 全量 | **751/0/0**（golden 不变；含修正后的键数断言）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- swd 武将面：**12/79 名、12/241 技转正**（九整将格局不变，四将均部分转
  正：拓跋玉儿 1/3、水镜 1/3、屈娴 1/2、壶中仙 1/3）。
- 墨剑 shaBegin→CardUsed 等价接入、道法来源自选→首张直发，均为已登记差
  异（规则基准按批语义表口径，不另开条目）。
- 批 22 候选：swd 剩余 67 名按将勘察（简单触发型优先：举荐/军令等同族）。
