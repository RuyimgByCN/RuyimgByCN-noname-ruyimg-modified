# 队列 1：13 来源配置/UI/渲染分类静态审查

2026-11-13；审查基线 HEAD `bd58ef06d59940baed8d7f93e61ce571b152fb81`。规则/NGSA/content schema/ABI/WS：`333 / 107 / 5 / 759 / 8`。开始时已跟踪树干净；本目录为未提交的新证据，共享工作区随后出现的其他并行改动以 [validation.json](validation.json) 为准。

本次只写本目录，没有修改清单、台账、旧证据或产品代码，没有编译、提交、推送。检查上游原始源码、当前原生实现、配置应用路径与组件消费；没有采信旧实现者/复核者的结论。没有读取官方运行库或上游打包的 Spine 文件；原始扩展中调用动画接口的周边代码只用于定位配置/表现引用。

完成 224 个唯一 key 的静态审查：115 configs、100 ui_hooks、9 surfaces。上游浏览器实际触发 **0**，原生运行场景 **0**，本轮新增技能等价签署 **0**。以下均是供主 agent 核查后选择性采纳的建议，不是已复现的运行时失败，也未应用到清单。

## 逐项结果

[findings.json](findings.json) 包含全部唯一 key、建议 status/difference/diff/note、上游行为、原生消费、具体引用、消费链、搜索词和证据级别；[recommendations.json](recommendations.json) 是同内容副本，供已引用该文件的协调消息继续使用。`difference` 与旧兼容字段 `diff` 保持一致。

| 来源 | 本次分类项 | 原等价/简化 | 建议下调 | 保留的原等价 | 若全采纳等价/简化 | 逐项表 |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| U06 | 0 | 0/0 | 0 | 0 | 0/0 | 本来源没有这三类清单项 |
| U07 | 0 | 0/0 | 0 | 0 | 0/0 | 同上 |
| L06 | 0 | 0/0 | 0 | 0 | 0/0 | 同上 |
| L22 | 3 | 3/0 | 3 | 0 | 0/3 | [L22](L22.md) |
| L29 | 4 | 1/3 | 1 | 0 | 0/4 | [L29](L29.md) |
| L10 | 13 | 10/3 | 10 | 0 | 0/13 | [L10](L10.md) |
| U01 | 0 | 0/0 | 0 | 0 | 0/0 | 本来源没有这三类清单项 |
| L28 | 8 | 7/1 | 7 | 0 | 0/8 | [L28](L28.md) |
| L27 | 23 | 22/1 | 16 | 6 | 6/17 | [L27](L27.md) |
| L19 | 1 | 1/0 | 1 | 0 | 0/1 | [L19](L19.md) |
| L20 | 64 | 61/3 | 49 | 12 | 12/52 | [L20](L20.md) |
| U08 | 0 | 0/0 | 0 | 0 | 0/0 | 本来源没有这三类清单项 |
| L25 | 108 | 108/0 | 96 | 12 | 12/96 | [L25](L25.md) |
| 合计 | **224** | **213/11** | **183** | **30** | **30/194** | [机器汇总](findings-summary.json) |

这里的 0 项仅表示这些来源没有 configs/ui_hooks/surfaces 清单项，不表示该来源全部技能已重审。

保留 30 个原等价状态的理由分开记录：15 个有真实规则消费、14 个上游不启用且原生惰性登记、1 个连杀显示计数清零。保留旧状态不是本轮重新背书；9 个 CSS 面及 2 个原有显示钩子仍维持原“简化近似”。没有因为本轮未跑浏览器一律下调。

## 183 条建议的证据性质

| 差别类型 | 建议下调 | 依据与限制 |
| --- | ---: | --- |
| 具体规则/公开信息分支差异 | 14 | 已逐段定位上下游代码，尚需真实事件反例复现；不能称为纯 UI |
| 计数/cue/flag 未接可见或可听效果 | 57 | 已读具体 handler、dispatcher 与组件入口；尚无浏览器复现 |
| 菜单模型/可见交互未接入 | 21 | 原始标题/帮助/图片/菜单回调与原生四字段模型不对应；尚无浏览器复现 |
| 表现配置仅解析、未驱动显示 | 66 | 配置 → parser → 数据/辅助函数 → 产品组件的静态消费链未闭合；属于待验证建议 |
| 规则/目录/开发工具配置应用链未闭合 | 25 | 另有技能实现不证明开关启停生效；记录包资产、具体 parser 与实际规则入口，属于待验证建议 |

“未找到消费者”是有边界的静态检索结果，不单独等同运行时缺失。每项都有 `consumer_chain`、`consumer_search_tokens` 与 `evidence_level`；[consumer-probes.json](consumer-probes.json) 保留 581 个当前产品源码文件及哈希、逐词命中和 Vue 消费结果。定义、导出、测试、无关通用标识不计为应用调用；`mode/text/balance/perspective/newui` 等通用词没有被当成有效消费证明。测试目录、内联测试尾部与所有 Spine/skeleton 路径被排除。

8 个配置消费链均写在 JSON 的 `consumer_chains`。例如：L10 的 parser 返回开关/十常侍替换计划，产品无应用调用；L20 的配置对象未控制独立绑定的规则技能；L27 调试双将只写 workbench，真实选将仍发出单将 choose；L25 的 40 项前端 dispatcher 只改状态，五类关键状态在 Vue 中命中 0，当前 envelope 仅分派四种通用事件 cue。

## 需要作为规则问题处理的 14 项

具体引用和条件在相应来源表及 JSON 中。这些是静态代码差异，不是已跑通的反例。

| 来源/条目 | 具体问题 |
| --- | --- |
| L27 mbcmqingzheng | 上游伤害条件直接比较两数组，原生比较张数；不能默认为同一个判据 |
| L27 sbqingzheng | 上游可选获得治世分支不以造成伤害为前提；原生放在弃牌数更多的伤害分支内 |
| L27 xingchong | 上游展示所选手牌并打公开标记；原生选择后只记内核 flags，未输出公开展示 |
| L27 dcxiaoxi | 上游视为使用 X 张杀，原生直接发 X 次 Damage，跳过用牌/响应链 |
| L27 twfengji | 上游按卡名从牌堆取至多 X 张；原生按 CardKind 并查弃牌堆，最后用普通牌补足 |
| L27 twzhouzu | 上游由目标自己选弃牌；原生固定弃目标前 X 张手牌/装备 |
| L20 _guozhan_marks | 先驱查看暗置武将分支只写 viewed/main/deputy flags，未形成观察者查看结果消费 |
| L20 _chongzhu | 普通弃牌摸一已实现，stone deck、spell_gold/silver 创建/替换分支未迁入 |
| L25 ssr_luoyi | 被误归 configs；上游禁防具/锦囊变决斗/双响应，原生却增加伤害并只记双响应 flag |
| L25 ssr_lieren | 被误归 configs；无武器时允许任意 he 卡、额外每回合一次限制，开局取装备代替创建专属武器 |
| L25 _anshaskill | 主公回合结束的暗杀/胜负被共享死亡窗口身份继承行为替代 |
| L25 _seadreammodeskill | 主公回合开始的骰数弃牌/封技被共享濒死阶段成长行为替代 |
| L25 _relifeskill | roundStart 处理死者倒计时/按上限复活与等级摸牌，被死亡窗口固定 4/4 摸四替代 |
| L25 _haoyunlai | 上游 goodluck 会重排牌堆顶，原生只作 UI 计数/shimmer cue |

其他可直接从代码确认的配置差异也逐条记录：L10 skillEffectsScale 原始枚举 11–15 被原生 0..10 校验拒绝，使用处还应除以 10；L10 多个默认 true 的选项变为 false；L27 gaibiaoji 默认 false 变 true；L27 jinxuanbiaoyu 原始 bool 被改为字符串三枚举。它们不依赖浏览器才可确认类型/默认值不同，但实际显示影响尚未触发。

## 注释和主动禁用项

[upstream-parse-summary.json](upstream-parse-summary.json) 用已有本地 Babel 解析 9 份原始 JS，语法错误 0，定位 215 个非 surface 引用，其中 **12 项不在活动定义中**：L20 `_shiqihujia_`、`zuoxing`；L25 `falsedelete`、`_newBackgroundUse`、六项 `_bingletexiao*`、`_thundertexiao`、`_thundertexiao2`。另 L25 `_extradamageskill`、`_extradlosehpskill` 是活动定义但 filter 恒 false。没有把这些注释内的拟议效果记作已运行上游行为。

[upstream-excerpts.json](upstream-excerpts.json) 使用 AST 定义边界，避免旧缩进启发式把相邻技能混入同一项。L29 采用原始单行 extension.js 的 line 1 与 **UTF-16 字符偏移**，不采用历史生成的 readable 文件作为实际来源。只是周边动画接口引用，不包含官方/打包运行库文件。

## 小来源的交互样本边界

L06 的两个技能都应全取：`_shengke` 的克分支包含强制弃牌选择（生分支自动摸牌），`wuxingpan_skill` 选择手牌与五行；装备卡本身不另算第三个独立技能。L22 五个钩子没有新增玩家选择，`_timeKiller` 观察已有 chooseTo* 不能凑作交互技能。U06 `mianju` 是强制翻面取消，不应算独立交互样本；应选有实际选择的其他技能。这些仅为静态样本筛选，实际触发交由主 agent 协调的浏览器任务。

## 复现本静态审查

从 noname-next 根目录执行，脚本只写本目录：

```bash
node docs/operations/evidence/2026-11-13-queue1-review/nonrule-audit/collect-upstream.cjs
python3 -B docs/operations/evidence/2026-11-13-queue1-review/nonrule-audit/collect-consumers.py
python3 -B docs/operations/evidence/2026-11-13-queue1-review/nonrule-audit/build-recommendations.py
python3 -B docs/operations/evidence/2026-11-13-queue1-review/nonrule-audit/validate-evidence.py
```

原始命令参数、退出码、标准输出/错误见 [static-commands.jsonl](static-commands.jsonl)；完整性结果见 [validation.json](validation.json)。验证覆盖 224 key 无重复/无漏项、三类数量、四种允许状态、每个源码引用的现存文件/行界、L29 原始偏移、当前源码哈希未漂移、本地证据链接与尾部空白。它不是产品测试或规则等价门禁；没有运行共享 Cargo、浏览器对照、公共生成器。

15:37 UTC 的最终校验曾因并行 U01 修改 3 个源码文件而失败（exit 1），原输出保留在命令日志；没有把旧通过结果当作当前整树结果。已读实际 diff，改动只涉及 `dddjuxian` 选择与 prompt、局部格式，不与本表任何 `native_refs` 交叉。刷新消费者扫描前保留[原快照](consumer-probes.before-u01-refresh.json)与[原校验](validation.before-u01-refresh.json)，具体哈希、范围判定见[并行漂移核对](parallel-drift-review.json)；补跑结果单独记录。

主 agent 应按证据级别逐类核查，再按完整 key 选择性修改 difference/diff/note/status。保留旧状态的项不要改写为本轮新签署，也不要用本表重新提升来源整包八项。
