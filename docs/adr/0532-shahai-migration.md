# ADR 0532：杀海拾遗归档扩展迁移（shahai）

- 领取：C 段 omp
- 来源：`archive/upstream/noname/apps/core/extension/杀海拾遗`（GNU GPL v3）
- 交付：`extensions/shahai`（332 武将 / 1092 技能 / 41 装备卡）

## 内容

杀海拾遗（shahai）是含 8 系跨界武将的大型内容扩展：gujian 古剑奇谭 / gwent 巫师 /
hearth 炉石 / mtg 万智牌 / ow 守望先锋 / swd 轩辕剑 / xianjian 仙剑奇侠传 /
yunchou 云愁·DIY。332 武将全部登记入角色索引，1092 项技能定义（39 项真实 DSL
运行时 + 1053 项元数据登记），41 张装备卡注册，依赖 standard / shenhua / extra /
yijiang / mobile / refresh / sp 七包（跨包技能按旧 JS 同名义引用解析）。

## 真实运行时技能（39 项，节选）

- gujian：醉饮/魅狐/黑龙鳞片/木杀
- gwent：狼噬
- hearth：火球术/奥术/雷欧克/法反/交汇/新希/图腾系列
- mtg：冰河要塞/龙颅山峰
- ow：夕阳/目镜/守护
- swd：凌波/流虹/疾影/暗恋影/血衣/断岳/乾坤袋/青龙之诡/刑天改造
- xianjian：寻影/探囊
- yunchou：吸才/蛊惑

各技能近似已逐条注明于 `shahai_impl.skill`。

## 元数据登记（1053 项，无运行时行为）

跨界包专属机制（发现/地图牌/秘术/法力水晶/连击/随从/阵型等）未移植，登记元数据
（描述取自上游译文原文）并文档化近似。跨界专属卡牌种类（法术/随从/地图牌/机巧）
与自定义基本/锦囊牌不可表达，未注册。

## 验收

content-compiler 515/515、conformance 8/8、31 包空 store 冷导入通过。
