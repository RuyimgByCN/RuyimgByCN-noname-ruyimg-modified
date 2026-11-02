# C331 五组真实调用续行：已审方案与来源／兼容补充

核验日期：2026-10-31。本目录是按裁决29准备的实现契约，**不是实现、运行、隐私或整包验收**。两个只读任务以不可变C9e 219文件overlay、其余5fc Git对象及固定上游完整body为来源；写入前必须再对实际C9f 225文件最终封存核对，不能覆盖其11组原子修复、chain出生校验或已实现的模式事实。

[原方案](guide-v1.raw.txt)、[原独立复审](review-v1.raw.txt)、[20项来源／协议／生命周期补充](wire-source-lifecycle-supplement.raw.txt)及[补充最终复审](supplement-review-final.raw.txt)均为限定No issues／OK with notes，未执行测试或改源码。[原来源清单](source-inventory-v1.json)与[摘录](source-excerpts-v1.txt)、[补充清单](supplement-source-inventory.json)与[摘录](supplement-source-excerpts.txt)保留具体路径、范围、签名、哈希及Git对象身份。

父[原方案来源审计](parent/noname-parent-five-fsm-plan-source-audit.json)重新核验219 overlay、6个不可变base文件、4个上游文件和37项读取绑定；父[补充来源审计](parent/noname-parent-five-fsm-supplement-source-audit.json)重核219文件、20项读取、5个base及27/29原审批字节。父验证器关于可选 `git_blob`／`origin` 字段的格式假设失败已分别保留并纠正，不作提案或产品失败。父曾把 `read_symbol(kind=enum)` 的未匹配误作文件缺失；[勘误](parent/noname-parent-five-fsm-choice-read-correction.json)确认219 overlay中有真实choice.rs及Skill变体，旧5fc文件只是另一明确版本，Tag的u8五行语义没有改变。

## 必须落到同一内核的行为

| 分组 | 必须实现和验证的续行 |
| --- | --- |
| F07 Qinglong | 实际可取消he获取、非首装备选择及失去装备技能；外部隐蔽手牌用私有候选绑定的opaque槽，不能公布物理CardId或用随机首牌替代选择；损失／转移和后续日志投影也须保护身份 |
| F09 Zhangwu | holder可选支付任意合法数量n的he，等待完整loss子树，再重算source的可选精确n张支付；取消或不足造成n伤害，保留自来源、实际归属、双方死亡／失牌与逐阶段恢复 |
| F12 Yuance | 两选项／取消、四种颜色分支、真正普通判定与真实改判提供者；不能直接翻牌或伪造延时锦囊，判定子树后的存活／HP操作按源执行 |
| F13 Jiding | 真实完整虚拟雷杀、闪及牌后子树结束后，才移除实际被准入定义；分组恢复须绑定真实调用／CardUsed因果、实际别名与具名Rushou，不能用牌种、名称或任意祖先猜测 |
| F16 Jidian | 真正目标选择／取消、目标判定／改判及普通雷伤害，自然允许合法嵌套反应；移除假 `jidian_chain` 静音，红判不能压掉下一次无关反应 |

普通Judge事件目前缺失；方案要求复用现有有限JudgePhase的JudgementProgress、真实改判队列、held-card和finalization，延时锦囊上下文改为可缺省而非伪造。实际Tiandao提供者可验证非首he替换、取得旧判定牌和颜色逆转；失牌后必须Yield等待完整子树，再继续最终化。held-card唯一性、停牌堆、死亡清理及byte恢复需验。不能以当前Guicai错误的旧牌去向冒作等价，也不能提前搬入后续332的GaiMing card65修复。

## 三个补充约束

**兼容与隐私**：拟新增HiddenCard仅为内核类型及private106映射；WS6继续既有 `{token:候选下标,label_key:字符串}`、现有SelectCards与索引应答，WASM使用现有字符串表示。不能把新Rust选项对象／tag、必需字段或原始隐藏CardId放上wire，也不能把Tag(u8五行)、Kind或Card冒作opaque槽来改变VM/context意义。既有形状从静态源码看可复用，**冻结旧WS6 reader的实际正向payload、索引提交、可缺省metadata及当前WASM／IPC／WS／UI互通仍未实测**。P/Q公开视图、成功选择及选择后的全部席位／旁观日志需等价；错误相等不是隐私证明。若实际兼容失败，先给精确payload／reader错误与边界，不能擅自分配新号或宣称758/6通过。

**来源不等于执行证书**：裁决27仍只允许实际普通OptionalGate Own／相应referee询问的可选source_skill。新隐蔽牌、目标、成本、普通判定及Other／inner询问即使有真实私有生产者和阶段收据，公开metadata仍为None／generic；不得扩大Own范围、放松唯一ask／完整shape／资格或应答门禁。裁决30表现事件是独立契约，现有九处None constructor不能由询问来源或C7标量SkillTriggered推定真实SkillBegin。

**完整调用生命周期**：原C7在创建非optional机器前记SkillTriggered／once_used，不能直接套进新的可取消body。固定上游direct调用仍创建真实Before／Begin，body中取消后仍按源结束End／After；Before被取消的Omitted、普通optional gate未进入事件的拒绝、body选择取消是不同路径。四个direct单位的自动日志受抑制：Qinglong成功选择才log；Zhangwu holder支付成功log，source后续取消保留该log；Yuance没有显式activation log；Jidian先排判定再排logSkillBegin。必须在既有Frame/queue/park中实现持久化的真实调用及log子事件／after事件，不能统一为“cost一律在Begin前”或“取消就没有调用”。支付、日志、body和移除收据在发子树前原子推进，已入场续行不因provider再查询丢失；新入场仍须真实资格。

实际实现仍需codec／hash／A0／Ask派生／存档校验／唯一性／全部宿主匹配更新，红→绿及每个支付、日志、判定、整张牌后的完整状态／事件／随机流／恢复轨迹。没有第二调度器、registry、VM或包管理器。规则／存档顺序仍为331/106→332/106→R05 333/107；ABI759／WS7留给既批R05，format1/API1保持。模式／provider生命周期、真正ChangeHp与DeathFlow、全部37来源仍必做；本目录不关闭任何对应门槛。

当前权威结果见[进度主稿](../../../architecture/native-progress.md)，原子修复基础见[C9f](../2026-10-31-c331-phase9f-atomic-source/README.md)，归档身份见[复制映射](../2026-10-31-post-c9e-doc-publication/post-release-archive-map.json)。
