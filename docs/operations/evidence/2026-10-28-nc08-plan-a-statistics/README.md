# 2026-10-28 NC08 方案 A 差分统计：触发顺序玩家询问的命中面（只统计，不改语义）

范围：NC08 触发全序复核（[复核证据](../2026-10-28-nc08-nc09-review-items/README.md)）的选项 A「维持确定性全序 + 差分统计」；本轮指令领取顺序第 3 项。**零代码/语义/版本改动**——只产出命中面数据，交用户在 A（维持现状）／B（OpenChoice 玩家选序）之间裁决。

## 1. 方法与判据

[scan.py](scan.py) 静态扫描 37 来源（`archive/upstream/noname/apps/core/extension` 8 个 + `archive/upstream/noname-liuli/noname/extension` 29 个）的全部 `.js`（剔除 `extension_old.js`/`extension_mess.js` 与图片目录），复刻上游 `arrangeTrigger`（`content.ts:4016-4062`）开 `chooseControl` 的条件：

- 同一 `doing`＝**同一武将**持有 ≥2 个**互异**技能（名字去重后 >1）；
- 监听**同一事件名**（新式 `trigger: {player|global|source|target: "事件"}` 与旧式 `trigger: {事件名: [... ]}` 都解析）；
- **有效优先级相等**——公式照抄 `game/index.js` `_priority`：`priority*100`（未声明=0）、silent +1、equipSkill −25、cardSkill −50、ruleSkill −75；
- 组内**均非 silent** 且**均有翻译**（`translate/translates` 对象顶层键或 `lib.translate.NAME` 赋值；上游以 `!lib.translate[skill]` 判「无描述技」优先直行）。

解析方式：等长结构掩码（注释与字符串内容置空、保留定界符）判定花括号/条目结构，键值内容从原文同坐标读取。**开发期两次勘误已留痕**：值前导空格导致 `trigger` 提取失败（修复后 1986/1986 触发技全解析）；`const translates = {`（等号）形式漏采导致全量 `translated=False`、命中为 0 的伪影（修复后命中恢复 9）。

## 2. 结果（[hits.json](hits.json)、[scan-output.log](scan-output.log)、[loose-groups.log](loose-groups.log)）

**总量**：37 来源、**1986** 个触发技、**700** 个武将（只统计带 `trigger` 的技能与带 `skills` 表的武将定义）。

**宽松全集**（同武将同事件 ≥2 技，不做排除）共 **11 组**；逐组裁决：

- 1 组被**优先级差**排除（杀海 swd_kangnalishi：yinguo 声明 priority 6 → 600 vs xuying 0）；
- 1 组被**无翻译**排除（杀海 diy_caocao：hujia 不在 translate 表 → 上游按「无描述技」优先直行，不开询问）；
- 其余 **9 组命中**（9 对、9 武将；5 组监听域一致、5 组含锁定技）：

| # | 来源 | 武将 | 事件 | 技能（监听域） | 含锁定技 |
|---|---|---|---|---|---|
| 1 | 3D精选 | ddd_xujing | useCardAfter | dddshijian(global) / dddxuyu(player) | 否 |
| 2 | 3D精选 | ddd_liuhong | phaseBefore | dddfuzong(global) / dddshixing(global) | 是 |
| 3 | 杀海拾遗 | ow_baolei | phaseEnd | bshaowei(player) / zhencha(player) | 否 |
| 4 | 杀海拾遗 | swd_tuobayuer | useCard | poyue(player) / swdliuhong(player) | 是 |
| 5 | 杀海拾遗 | pal_xuejian | loseEnd | duci(player) / xshuangren(player) | 否 |
| 6 | 杀海拾遗 | pal_zixuan | phaseEnd | shuiyun(player) / wangyou(global) | 否 |
| 7 | 杀海拾遗 | pal_leiyuange | damageBegin | ya(player) / song(source) | 是 |
| 8 | 杀海拾遗 | pal_xiaoman | phaseEnd | lingquan(player) / shenwu(global) | 是 |
| 9 | 英雄杀 | yxs_lanlinwang | damageEnd | guimian(source) / lyuxue(source) | 是 |

人工抽查 3 组对照源码均吻合（bshaowei/zhencha、guimian/lyuxue、yinguo/xuying 的排除）。

**命中面占比**：9/700 武将（1.3%）、9 对 / 1986 触发技的 C(1986,2) 量级组合（<0.001%）；集中在 3 个来源（杀海 6、3D精选 2、英雄杀 1）。

## 3. 口径与限制（读数必读）

1. **上界**：静态扫描不执行 `filter`。命中组的两个 filter 若互斥（如 #3 bshaowei 的 filter 显式检查 `player.hasSkill("zhencha2")`），实战不会并列。逐组需人工核对 filter 才能得到精确值。
2. 只统计**武将自带技能**；运行时动态授予（技能授予/卡牌/装备/标记）造成的跨武将并列不在表内（原生此类授予面本身有限）。
3. firstDo/lastDo 时机上游不做优先选择；静态未区分时机类别（上表事件名均常规时机，phaseEnd/damageEnd 等不属 firstDo/lastDo 命名族，风险低）。
4. liuli 侧 UI/工具型扩展（十周年UI 306 技、祖安 244 技等）多为改写/皮肤技，0 命中与直觉一致。
5. 37 来源为**扩展包**范围；两仓**核心 character 包**（standard/offline/jsrg 等非扩展内容）不在本统计内——若 B 立项需另行统计。

## 4. 供裁决

- **选项 A（维持确定性全序）**：命中面数据支撑——真实同武将同事件同优先级的非 silent 有描述技对在 37 来源扩展面内仅 **9 组**（且全部可用 filter 互斥性进一步收窄）；现状的确定性收益（AI/回放/跨端零交互依赖）不变，9 组按 def 序结算与上游 `chooseControl` 的差异逐组登记为已登记差异即可。
- **选项 B（OpenChoice 玩家选序）**：影响面见[复核证据](../2026-10-28-nc08-nc09-review-items/README.md)选项 B 段（触发全序键变更、规则升号、golden 重录、三端投影一致、恢复轨迹）；命中面小意味着收益集中在上述 9 组。

本批不实施任何一项；裁决后按需另立 ADR／授权（方案 B 须升规则版本，先行请示）。

结束时间见 [end-time.txt](end-time.txt)。
