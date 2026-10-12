# 2026-10-11 T11 阴间系马面片：炼狱+反馈核对（规则 v256 生效批）＋马面条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁延续批次链；黑无常片后阴间系收尾）。上游依据：`boss_mamian`（马面/male/shen/hp6/skills `[boss_guiji, fankui, boss_lianyu, juece, boss_bianshen3]`）；`boss_lianyu`（9072：phaseJieshuBegin——全部其他存活各 1 火伤，无询问直伤）。

## 建模

- **炼狱**（查询型 Noop，`BOSS_LIANYU`）：End 臂——全部其他存活角色各 1 点火焰伤害（无询问直伤）。
- **鬼计共用**：v256 黑无常 def 直接复用（条目列 BOSS_GUIJI）。
- **马面条目**：def 索引 34/hp6/`[鬼计, 反馈, 炼狱]`。
- **反馈核对**：FAN_KUI 标准技原生已实现（standard.rs 反馈 SkillDef）——**入条目技能表**（与夜叉 bazhen「装备承载不入表」不同：反馈是武将技、native 有对应 SkillDef，可直接列）。
- **juece 混入勘误**：马面表列 `juece`（旧绝策）——refresh 包只有 `rejuece` 界绝策；旧绝策定义未在已勘包中定位，**未勘登记不落**。

## 登记差异

- juece 旧绝策未勘未落（refresh 包仅 rejuece 界绝策，同名非同技）。
- 势力 shen→群目录近似（先例族）。

## 坑位实录

- **漏加 SkillDef 本体**：炼狱只加了常量+条目+FROZEN+kernel 臂，SkillDef 忘加——assembly panic "catalog skill 炼狱" 当场拦截。教训：新技能四件套（常量/SkillDef/条目/臂）缺一 assembly 即炸，但若该将不被 debug 装配则静默——**四件套核对应入批收尾清单**。
- 版本序勘误：黑无常批 bump 脚本未执行（提交时仍 255+内容），本批正确顺延 **256**（非 257）；黑无常 README 已加勘误注。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_mamian_lianyu_fire_all` | 三段：①炼狱（End 后敌方 0/2/3 各 3→2、本体不受）②鬼计共用+反馈标准技入条目断言 ③炼狱实例存档往返+守恒 |
| 既有回归 | boss 全滤 53/53；机械审计：无多重注册/零属性清零；冻结表 10/10 |
| 版本递增探针 | 255+内容→256 全 14 位移；回退 255+内容复跑 **全绿**（位移唯一来源=版本字段；黑无常内容并入 255 态哈希一致） |
| 14 golden 重录 | 256 全量重录 2/2 绿（行级程序化） |
| 全量 | **603 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | cargo 真实 exit 0 |
| 版本 | 规则 **256**（黑无常内容随本批生效，255 态从未独立存在）；存档不变 v59 |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批候选：juece 旧绝策勘验（马面尾技，混入未勘）或 bianshen3 阴间变身系收尾。
