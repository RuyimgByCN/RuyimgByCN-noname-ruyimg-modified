# U01 独立复核

核验日期：2026-11-13。复核者：`/root/queue0_independent_review`，未参与本批生产实现。基准 HEAD 为 `bd58ef06d59940baed8d7f93e61ce571b152fb81`；最初捕获时 tracked clean。规则333／NGSA107／content schema5／ABI759／WS8。后续工作区出现 root 所有的聚贤修补和进度文档改动，另存增量输入，不把该状态写成 clean。

本次逐项重新阅读实际 U01 上游定义和原生落点，完成64技能、31武将数据的表格。62技能已有具体源码差异，其中61项列入新增降级候选，暗织1项保持既有「简化近似」；远振、齐策2项列为 `potential_gap`，不以本次疑点要求改状态。清单未由本复核者改写。**本次新增等价签署0；Cargo运行0；上游游戏实际触发0；浏览器运行0。** 反例均是待执行建议，不能当作运行复现。

严格集合及逐项依据见 [classification.md](classification.md)，真实 inventory key 和机器分类见 [findings.json](findings.json)。远振已检查单目标、目标非本人、存活、距离不等于1和弃牌/取消摸牌代码；尚未证实 `CardUsed` 对 `useCardToPlayered` 的适配在后续改目标时产生差异。齐策内容中的摸2、禁用和本人用锦囊解除确有落点，延时锦囊也在当前 `is_trick_kind` 中；其元数据差异单列，未扩大成未证实的内容错误。初稿把64全部建议降级的结论撤回；初稿原文保存在 [draft-before-strict-classification/](draft-before-strict-classification/)，含尚未纠正的草稿表述，最终采用本目录顶层文件。

| 范围 | 具体记录 |
| --- | --- |
| 15技能：A组及3个既有Host | [skills-a.md](skills-a.md) |
| 16技能：B组 | [skills-b.md](skills-b.md) |
| 16技能：C组 | [skills-c.md](skills-c.md) |
| 17技能：D组 | [skills-d.md](skills-d.md) |
| 31将四静态字段与额外边界 | [characters.md](characters.md)、[characters.json](characters.json) |
| 64技能结构化字段 | [skills.json](skills.json) |
| 旧表行号与标识对应 | [historical-row-map.json](historical-row-map.json) |
| root浏览器样本候选 | [browser-candidates.md](browser-candidates.md) |
| 工作区聚贤局部实现独立复核 | [implementation-review.md](implementation-review.md)、[implementation-conclusion.json](implementation-conclusion.json) |

每技能表分别写触发、条件与次数、选择及取消、上游步骤、原生执行、确定差异或边界、反例建议和现有测试实际断言。旧复核仅提取标识与行号；完整95明细行（31将＋64技能）及重复抽样均已对应，因此覆盖原72模板行涉及的全部标识。没有用旧表、实现者解释或实现者README判断正确性。

31将性别、势力、最大体力与技能列表逐项相同；张闿初始hp3/maxHp4有真实装配分支。主公标记、特殊姓名、替换表、图像与3D显示未验证；没有仅凭原生缺少同名字段就断言这些行为一定错误。资产中文`ddd刘冲`1将与augment的31个`ddd_*`名称分别记录，未把不同ID混成一个计数。`characterFilter.js`当前空对象没有规则效果。

测试检查区分当前Host真实事件与旧VM直调。`u01_ddd_eight.rs`实际4个函数覆盖注册和若干代表分支；注册64个名字不等于64技能行为通过。旧`decision34_pack9_closure.rs`多项以`invoke_p9_op(StorageSet)`设置旧VM状态，不能当当前Host验证。已读的实际测试文件及字面引用索引见 [test-reference-index.json](test-reference-index.json)；该索引不声称别名、间接调用或全部测试路径已覆盖。暗织20测试本轮只读断言，队列0的运行结果不重算成本轮执行。

冻结原始输入27份及SHA256见 [inputs.json](inputs.json)；补充读取的30份原生/上游源码和测试见 [inputs-additional.json](inputs-additional.json)，原生补充副本均取上述HEAD。原27份未覆盖或改写，额外manifest逐个验证其原SHA256及原生HEAD一致。原生当前Host装载确有64个落点，但这只确认执行类型。

实际执行的轻量命令为：

```bash
python3 -B docs/operations/evidence/2026-11-13-queue1-review/independent/u01/capture-inputs.py
python3 -B docs/operations/evidence/2026-11-13-queue1-review/independent/u01/audit-data.py
python3 -B docs/operations/evidence/2026-11-13-queue1-review/independent/u01/validate-review.py
```

生成结果为64技能／31将／旧95明细行，分组15＋16＋16＋17；文档与结构检查结果存 [validation.json](validation.json)。这些是证据结构及输入完整性检查，不是游戏测试。没有占用共享target，没有修改产品、公共台账、inventory或旧日期文档，没有提交或推送。

暗织仍保留Prepare与整回合开始的差异、priority/选序差异，以及原生桃园无懈模型和既有Duel实体CardId0收尾限制。其余尚未实际执行的反例、各选择窗口保存恢复、AI取向与跨来源组合都保留为后续验证；本报告不签U01整来源或13来源等价。

## 2026-11-13 23:50 +08:00 聚贤J06补审

新增暗牌候选一次性RNG排列、保存映射和非法多重集档案校验已按实际源码补审；限定结论及26输入hash见 [j06-followup-review.md](j06-followup-review.md)。J06在当前源码中已修，已读范围未见本补丁新增阻断；原退回记录保留。J01–J05仍存在，聚贤保持「简化近似」建议。本补审Cargo／浏览器／上游实跑均0，具体测试断言与未验范围分别列出，不签整技能等价。
