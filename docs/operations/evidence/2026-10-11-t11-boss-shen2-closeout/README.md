# 2026-10-11 T11 星宿阶段 2 系收尾清点（mode_boss 全量勘验路线图）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（自润 v247 后的收尾清点批；领取链延续）。本批为**勘验登记批**：全量枚举 `mode/boss.js` 武将条目与技能引用，标记逐将状态并登记剩余路线图。无引擎代码改动——规则 247/存档 v57/golden 不动（无语义变化先例同 versus 4v4）。

## 已入库（26 条目，v207–v247）

| 族 | 条目（规则版本） |
|---|---|
| 四凶+凶兽系 | 烛阴 v208、混沌 v209/v210、穷奇 v211–v214、饕餮 v215/v216、梼杌 v217–v219 |
| 神兽系 | 书精 v220、相柳 v221–v223、朱厌 v224–v226、毕方 v227–v229、应昭 v230–v232 |
| 四象阶段 1 | 青龙 v233/v234、白虎 v235、朱雀 v236、玄武 v237 |
| 星宿阶段 2 | 太昊 v238、蓐收 v239、祝融 v242、颛顼 v246、炎帝/少昊 v243、共工 v244、明星 v245、玄冥 v247 |
| 陪将 | 焰灵 v241（神裔外的独立陪将） |
| 运行时授予技 | 中毒/蛇毒（v220/v223）、造异异变（v246）、急叮（v245 授予面） |

## 已处置登记（不落内核）

- 隐藏链八技（青木2/3、白芒2/3、赤炎2/3、玄麟2/3——dieBegin hide+dieAfter 换将重生链，2026-10-11 处置批）。
- 青翼/赤翼/白翼（回合 [3,5,7] 编排+轮次门控族——青翼先例口径）。
- boss_buchun 不臣（复活书精+相邻节点流——阶段流程耦合，v246 前勘验登记）。

## 未落清单（后续批路线图，按族分组）

1. **试炼态四将**（boss_qingmushilian/chiyanshilian/baimangshilian/xuanlinshilian）：`boss_qingmu/boss_chiyan/boss_baimang/boss_xuanlin` 本技 + `*_intro1-3` UI 导引技——intro 面为 UI（不落），本技待勘验（疑为阶段 0 弱化形态技）。
2. **年兽系**（boss_zhuoguiquxie 捉鬼祛邪 + 五个年兽形态条目）：`boss_nianrui/boss_mengtai/boss_jingjue/boss_renxing/boss_ruizhi/boss_nbaonu/boss_shouyi/boss_nbianshen(x)`——变身/形态切换族（bianshen 机制族）。
3. **阴间系**：黑白无常（boss_baolian 爆敛/boss_qiangzheng 强征/boss_zuijiu 醉酒/boss_guiji 鬼计/boss_taiping 太平/boss_suoming 索命/boss_xixing 吸行+标准技 juece 混入）、罗刹/夜叉（boss_modao 魔道/boss_yushou 御兽/boss_moyan 魔眼/boss_mojian 魔剑/boss_danshu 胆术+标准技 yizhong/bazhen 混入）、牛头马面（niepan 涅槃/boss_manjia 蛮甲/boss_xiaoshou 枭首+boss_bianshen3/4 变身）——`bianshen` 变身机制为全族共同依赖（上游 changeSkin+重整形态），建议先行勘验变身机制承载面再逐将。
4. **杂项**：标准技混入条目（juece/yizhong/bazhen/niepan 为上游标准/扩展技引用——入库时按标准包既有实现核对）。

## 承接条件备注

- 年兽/阴间系的 `bianshen` 变身族依赖形态切换（changeSkin=视觉+技能组重挂）——native 无皮肤概念，技能组重挂可经运行时授技承载（中毒/造异异变先例），视觉面登记。
- 试炼态 intro 技为 UI 引导（mark/intro 字段）——登记不落先例（神恩 AI 面同口径）。

## 验证

| 项 | 结果 |
|---|---|
| 引擎改动 | 无（勘验登记批） |
| 版本/存档/golden | 规则 247/存档 v57/14 golden 均不动 |
| 全量 | 本批无代码改动，沿用 v247 实测 **593/0/1**（同源码） |
