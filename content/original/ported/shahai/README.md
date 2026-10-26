# 杀海拾遗（shahai）归档扩展迁移

- 来源：`archive/upstream/noname/apps/core/extension/杀海拾遗`（GNU GPL v3）。
- 本包登记 8 系跨界武将 332 名（gujian 古剑奇谭 / gwent 巫师 / hearth 炉石 / mtg
  万智牌 / ow 守望先锋 / swd 轩辕剑 / xianjian 仙剑奇侠传 / yunchou 云愁·DIY），
  技能 1092 项（含占位），装备卡 41 张；依赖 standard / shenhua / extra / yijiang /
  mobile / refresh / sp 七包（跨包技能按旧 JS 同名义引用解析）。

## 发布状态

- 全部 332 名武将登记入角色索引，技能语义完整登记（按来源包拆分至 `shahai_meta_*.skill`，轩辕剑另分角色与战斗／卡牌；描述取自
  上游译文原文）。
- 41 张装备卡按引擎可表达种类（weapon/armor/horse/treasure）注册；跨界包专属卡牌
  种类（法术/随从/地图牌/机巧等）与自定义基本/锦囊牌不可表达，未注册。
- 真实 DSL 实现 39 项（`shahai_impl.skill`）：醉饮/魅狐/黑龙鳞片/木杀（gujian）、
  狼噬（gwent）、火球术/奥术/雷欧克/法反/交汇/新希/图腾伤害抽牌/图腾回合结束抽牌/
  图腾进攻距离/图腾手牌上限（hearth）、冰河要塞/龙颅山峰（mtg）、夕阳/目镜/守护
  （ow）、凌波/流虹/疾影/暗恋影/血衣/断岳/乾坤袋/青龙之诡×5/刑天改造×3（swd）、
  寻影/探囊（xianjian）、吸才/蛊惑（yunchou）；各技能近似已逐条注明。

## 可表达性缺口（未移植机制）

- 发现/地图牌/秘术/法力水晶/连击/随从/阵型等跨界包专属机制
- 动态复制他人技能（血轮眼等）
- 多阶段目标侧分支状态机（预言、二选一链、循环选择）
- 自定义卡牌种类与派生牌
- 装备技触发（装备牌携带触发事件）
- 判定分支、整张牌使用取消、目标替换

## 已知跨包引用

`wushuang/mashu/longdan/biyue/feiying/liuli/qianxun`（standard）、
`qiangxi/lieren/luanji/beige/jiang/jiuchi/niepan/tianxiang/xinliegong`（shenhua）、
`gongxin/lianpo`（extra）、`jueqing/shangshi/xinwuyan/zhenlie`（yijiang）、
`rejieming/xinfu_jingxie`（mobile）、`reyingzi/yijue/zhuhai`（refresh）、
`songci/yicong`（sp）；`hujia` 为上游子技能引用，本包以占位登记。

```bash
cargo run -q -p content-cli -- pack \
  content/original/ported/shahai/*.skill \
  --characters content/original/ported/shahai/characters.json \
  --cards content/original/ported/shahai/cards.json \
  --metadata content/original/ported/shahai/metadata.json \
  --out content/original/ported/shahai/shahai.npack.json \
  --package-id shahai
```
