# 队列1：224项清单应用独立复审

日期 2026-11-14T00:33:52.479865+08:00；HEAD `bd58ef06d59940baed8d7f93e61ce571b152fb81`；dirty。规则/NGSA/schema/ABI/WS：333/107/5/759/8。

本次只新增 application-review* 证据，不改清单、产品、公共文档或旧证据。

结论：`accepted_after_corrections`。224个key均可从原始清单经首次应用与5份更正记录精确还原到当前字节；输入与源码哈希保留在 JSON。

当前这224项：已复核等价 118 / 简化近似 106；有效撤回 95。全清单计数 {'已复核等价': 634, '简化近似': 3790, '缺失': 0, '上游未定义': 0}，有效撤回 215，没有新增等价签署。

| 核对组 | 数量/结果 |
| --- | --- |
| 本表具体规则14 + cue/flag57 + 菜单21 | 92项均采纳下调 |
| 消费链待核验91 | 保留状态88；仅另有true→false默认值直接反例下调3 |
| 原保留等价30 / 原简化11 | 30 / 11均保留 |
| 注释12 + filter恒false2 | 14项均保留，没有当作活动技能 |
| 状态未变的原复核元数据 | 129项逐字段保留 |
| difference/diff、其他原字段、旧note前缀 | 224项均保持一致或保留 |

首次应用中11项撤回理由已退回并另写更正，原始审查和应用记录保留：

- `L20:ui_hooks:_xigua_zhuwanzhongji_@archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:2290`：首次下调把真实主亡忠继身份交换误判为 L20BatchA flags；实际优先绑定 ZhuWanzhongJi，已恢复原状态/复核人/日期。mingzhong/zhuUpdate/死亡时序仍待实际同场景核验。
- `L19:configs:zuan_brawl@archive/upstream/noname-liuli/noname/extension/末日乱斗/extension.js:3363`：首次文案误称枚举未实现；完整五值解析和默认 all 已存在。未找到消费者不足以单独确认缺失，已恢复原状态/复核人/日期，off/all/daily 与跨日重启继续待核验。
- `L10:configs:cardFold_proportion@archive/upstream/noname-liuli/noname/extension/史诗卡牌/extension.js:2215`：恢复原状态/复核人/日期；字段、旧键与静态消费链差别继续待核验，不等同已证实产品行为缺失。
- `L10:configs:分割线02@archive/upstream/noname-liuli/noname/extension/史诗卡牌/extension.js:2258`：恢复原状态/复核人/日期；字段、旧键与静态消费链差别继续待核验，不等同已证实产品行为缺失。
- `L25:configs:bizhi@archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:50762`：恢复原状态/复核人/日期；字段、旧键与静态消费链差别继续待核验，不等同已证实产品行为缺失。
- `L25:configs:hidessr@archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:51168`：恢复原状态/复核人/日期；字段、旧键与静态消费链差别继续待核验，不等同已证实产品行为缺失。
- `L25:configs:newhuihe@archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:50572`：恢复原状态/复核人/日期；字段、旧键与静态消费链差别继续待核验，不等同已证实产品行为缺失。
- `L25:configs:perspective@archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:50757`：恢复原状态/复核人/日期；字段、旧键与静态消费链差别继续待核验，不等同已证实产品行为缺失。
- `L25:configs:showboss@archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:51163`：恢复原状态/复核人/日期；字段、旧键与静态消费链差别继续待核验，不等同已证实产品行为缺失。
- `L25:configs:text@archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:51404`：恢复原状态/复核人/日期；字段、旧键与静态消费链差别继续待核验，不等同已证实产品行为缺失。
- `L25:configs:windowHigh@archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:51344`：恢复原状态/复核人/日期；字段、旧键与静态消费链差别继续待核验，不等同已证实产品行为缺失。

另保留的3条撤回只采纳cardFold/cardPhantom/shichangshi的上游init=true而包资产/parser默认false这一直接值反例。其余9条字段/旧键/模型观察保持待核验，不据此下调；L19枚举解析存在、L20身份交换存在的误判也已撤回。没有只沿用其他审查的confirmed标记。

差别文案保留220条逐key上游行为；L20免伤/伤害牌动画与两张CSS另按实际源精化4条，承认已有原生事件/过滤，限定未实现或未对照的可见效果。correction04/05均未变更status/reviewer/date。

原生/上游运行场景、Cargo、浏览器均为0；没有重验整技能等价。L25 _guanmodeskill 只核对correction02记录的字节链，不在本224项规则复核范围。

复核脚本先后因未兼容correction04/05紧凑差别格式、把新增日期段也视作历史改写而退出1；已补字段级读取并将守卫限定为旧原文不得删除/替换，原stderr与补跑分开保留在命令日志。这不属于产品测试结果。

源码引用界限核对 1004 处、独立文件哈希 62 个；历史evidence旧原文删除/替换0，另有1个文件被主agent插入新日期段（本agent未写）。未解决应用问题 0。

完整输出：[application-review-final.json](application-review-final.json)。实际命令与输出见 [application-review-commands.jsonl](application-review-commands.jsonl)。
