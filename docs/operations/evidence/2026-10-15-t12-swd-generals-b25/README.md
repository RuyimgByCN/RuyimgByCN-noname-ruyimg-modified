# T12 swd 武将面批 25：协律（壶中仙）+朱羽（芝茵）双技部分转正（2026-10-15）

## 承载

两技 HostBehavior（tag 284/285，键数冻结表 283→**285**）：

- **协律 xielv**（壶中仙，2/3）：弃牌阶段结束=End 帧**入场**（PhaseBegin
  +phase==End+begin:true，自愈/炮台「结束阶段」通道同口径；上游
  phaseDiscardEnd）——手牌 ≥2 且同色（可选确认门）→ 回复 1（未满血门）
  + 弃置**场上全部判定牌**（座位序遍历确定性）。
- **朱羽 zhuyu**（芝茵，2/3）：横置角色即将受到**非传导**伤害（DamageBefore
  帧直读 d.from_chain；可选确认门）→ 弃 1 张红色牌（手/装备区）→ 伤害
  +1 且改火属性（缚斧 current_damage_mut 帧直写同位，nature=Fire 直改）。
  取消=伤害照常。登记差异：上游 damageBegin global 全体可触发，原生同
  global（filter 不限 owner 与受伤者关系）。

## 结构性依赖登记（不在本批消化）

- **壶境 hujing**（壶中仙第 3 技）：依赖**炼妖壶 lianyaohu** 卡（equip5
  宝物+onEquip/onLose 标记技 lianyaohu_skill+锁定再装备+手牌上限+2）——
  该卡不在转换包 45 张内。**勘察发现**：上游 swd list.js 实有 **92 条活
  动卡牌行**而转换包仅 45 张——批 16「卡牌 45/45」口径需独立勘验批对账
  （机关牌/装备/锦囊/食物族之外尚有一半条目未转换），壶境随炼妖壶卡入
  包批一并承接。
- **雪煌 xuehuang**（芝茵第 3 技）：limited 一次（PlayerState bool=存档
  bump）+手牌全红弃全部+逐张视为对 2 名随机敌人使用火杀（虚拟牌 use_
  virtual_card 循环）——中批规模，随芝茵余技批承接。
- 屈娴 huanxia（viewAs 红牌当杀+expire/draw/gain 三子技）：viewAs 族+子
  技组，独立批。

## 测试（swd_generals_b25_xielv_zhuyu）

| 技 | 正例 | 反例 |
|---|---|---|
| 协律 | 2 红手牌 End 入场确认 → 回 1+p1 判定区乐被清 | 异色无窗；单张无窗 |
| 朱羽 | 横置 p1 受杀确认弃红 → 2 伤（8-2）+红牌离手 | 取消 1 伤；未横置无窗；无红牌无窗 |

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **36/36**（首跑即绿） |
| engine-native 全量 | **755/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| Gate 1b | 首跑拦未用变量 red（已修）→ 复跑 exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- swd 武将面：**21/79 名、23/241 技转正；整将维持十五**（壶中仙 2/3、
  芝茵 2/3 部分推进）。
- **新挂账**：swd 卡牌面对账批（上游 92 条 vs 转换包 45 张，含炼妖壶/
  壶境依赖链）——建议列 T12 卡牌面续批前置。
- 批 26 候选：雪煌（芝茵整将收口，limited+虚拟火杀循环）或 huanxia
  （屈娴整将收口，viewAs+子技组）。
