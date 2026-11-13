# 聚贤 J06 私有候选排列补审

补审时刻：2026-11-13 23:50:16 +08:00（输入捕获时刻）。复核者`/root/queue0_independent_review`；HEAD `bd58ef06d59940baed8d7f93e61ce571b152fb81`＋工作区改动，tracked dirty。规则333／NGSA107／schema5／ABI759／WS8。只读实际6文件diff及连接源码，没有读取实现者说明或本批README作正确性依据。Cargo0、浏览器0、上游实际触发0。

**限定结论：J06所述“暗槽固定对应原hand数组索引”在当前源码中已修，已读范围没有发现该补丁新引入的阻断。** 私有排列在开窗前由引擎RNG生成一次，保存在既有Host载荷，答案及恢复沿同一排列。新增测试已写出相关断言，复核者未运行。聚贤仍建议「简化近似」，J01–J05的确定差异保留；没有整技能或整来源等价签署。

## 输入与原退回记录

26份实际输入的SHA256、时刻及副本见 [j06-followup-inputs.json](j06-followup-inputs.json)；已跟踪diff见 [j06-followup.diff](j06-followup.diff)，新增测试全文在该manifest的副本内。与第一次局部输入相比，只有A实现、A询问辅助和新增聚贤测试3份文件变动；原有提示、分组登记和eight兼容修改继续逐项核对，其hash未再变化。

初次J06退回正文、结论及hash完整保存在 [退回报告](implementation-history/initial-unshuffled/j06-rejection-review.md)与[退回JSON](implementation-history/initial-unshuffled/j06-rejection-conclusion.json)，顶层原 [implementation-review.md](implementation-review.md)和[implementation-conclusion.json](implementation-conclusion.json)也保留其当时结论。这里新增补审记录，不把初次退回改写成初次通过。

## 对照实际源码

| 核对点 | 源码依据与结论 |
| --- | --- |
| 上游随机入口 | 实际`content.ts:8955–8963`先取`getGainableCards(player,"h")`，调用`hs.randomSort()`后创建可见/暗牌按钮。不是仅按目标原hand次序给暗槽编号。 |
| 一次生成 | 当前`u01_batch_a.rs:146`在top完成支付、压牌顶后克隆目标hand，147行`ctx.core.rng.shuffle`，再追加至`machine.cards`。只在TargetChoice进入CardsAsk时生成；后续CardsAsk处理与active_ask重建都不调用shuffle。当前RuleRng的shuffle为确定性Fisher–Yates，状态及调用数在core内。 |
| 固定载荷 | `machine.cards[0]`仍是支付牌，其后是私有候选排列。没有修改Host结构、stage枚举或存档字段。`HiddenCard(slot)`从`cards[slot+1]`取得实体，不再从当前hand数组同索引取得；还核对其确实仍在目标hand。装备答案另验证在目标equips。 |
| 候选校验 | `u01_batch_a_asks.rs:693`对ask28要求CardsAsk阶段、mode0、目标非owner；取cards后缀与目标hand分别排序后比较完整多重集。缺牌、外来牌或重复某牌导致与真实hand不符时，active_ask返回None，开放选择的承接检查拒绝档案。owner/target/物理牌越界仍先由通用machine校验拒绝。 |
| 不重洗恢复 | Host快照保存card/cards/target/stage；content hooks按原字段重建。active_ask只排序克隆副本作多重集核对，不改变保存顺序，不消费RNG。原open_choice仍需observer/prompt/kind/options顺序/min/max/allow_cancel逐字段相同且唯一承接。 |
| 可见性 | 实际ChoiceSpec继续只给`HiddenCard(0..n)`与公开装备ID；qid/observer应答权限不变。新私有实体排列在Host载荷中，未放进候选。已检查PlayerView构造、choice_source及公开选择建议路径，没有新增导出该排列的分支；客户端实际渲染未在本补审执行。 |
| 持牌归属 | 目标候选是仍在目标手中的引用，支付牌仍在deck。`machine_held_cards`仅把末焱/调兵等特定Host载荷当暂离牌，U01 batch A不在其中，故没有把新后缀再计一次实体归属。 |
| 非法答复 | `ChoiceSpec`仍一张强制，命令先核对observer、choice_id和候选/数量；非法空答、物理暗手牌ID、越界暗槽不会进入步骤。新映射查找也使用get及归属filter，没有直接越界索引。 |
| 原登记文件 | `known_prompts.rs`继续只追加`dddjuxian_gain`；`groups/g1.rs`登记新测试；`u01_ddd_eight.rs`在旧唯一牌场景显式回答HiddenCard(0)。这些没有因J06再改变规则元数据或旧断言期待。 |

## 测试实际写到哪里

当前`u01_juxian_gain.rs`共4个测试函数。新增`juxian_hidden_order_uses_rng_is_archived_and_rejects_invalid_mapping`将规则RNG设置为8个固定种子，各经真实ActivateSkill流程进入gain，读取私有载荷并检查：首元素是支付牌、后缀包含两张目标手牌且各一次；不同种子出现两种首候选；存在规则随机调用；恢复后parks和RNG完全保持；把一候选改为重复另一候选的档案被拒。

原三个gain分支不再把暗槽0/1写死为black/red，而验证两槽取得不同目标实体、装备分支取指定装备，并据实际所获颜色检查ban。三分支仍经NGSA字节encode/decode/restore后同选对照state_hash与新增日志。非法应答与公开选项不变、AI换序不读牌面的断言继续存在。这些检查有实际行为目的，不能只由生产函数名或注册数量推得。

本补审只读测试及源码，未报告4/0的独立运行结果。单手牌、仅装备、更多手牌、已明牌、gainable屏蔽和额外出牌阶段没有被这8个双手牌种子代替验证。原生与JS逐位随机流对照也未执行。现在新增随机消耗会影响本技能此路径的RNG调用数及后继轨迹，root的exact-head完整门禁与golden核验需另记。

## 保留边界

J01（阶段次数仍按turn）、J02（无其他he时top预先排除及先选目标后支付）、J03（gainable/已明牌/观牌可见性）、J04（失牌子触发晚于gain选择）、J05（完整gain/give/move/reveal事件）均未由该补丁消除。原无他人he的测试仍只是当前近似兼容断言。

本结论仅确认J06的私有排列生成、保存、映射和结构校验在已读源码中有落实。机器结果见 [j06-followup-conclusion.json](j06-followup-conclusion.json)；补审结构/链接/输入检查见 [j06-followup-validation.json](j06-followup-validation.json)。64项冻结基线及62／2分类均保持原记录，inventory未由复核者编辑。
