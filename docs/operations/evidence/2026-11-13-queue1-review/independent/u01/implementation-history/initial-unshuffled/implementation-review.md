# 聚贤gain选择修补：独立代码复核

2026-11-13；复核者`/root/queue0_independent_review`。HEAD `bd58ef06d59940baed8d7f93e61ce571b152fb81`＋工作区改动，tracked dirty；规则333／NGSA107／schema5／ABI759／WS8。此增量与64技能冻结基线分开。复核者没有参与实现，没有阅读实现者说明或本批README；只读实际diff、连接源码及上游原始skill.js。本增量Cargo0、浏览器0、游戏实际触发0，未引用root的运行计数作为独立执行结果。

**限定结论：新增取牌窗口消除了“top固定拿目标第一张he”的实现路径，已读范围没有发现该局部修补新引入的阻断问题。** 源码已增加取目标暗手牌或公开装备的真实选择，现有Host续行、档案提示注册和应答校验均有承接。下述测试是已读的测试代码，尚待root完整门禁结果；不能据本报告签整个聚贤等价。严格降级建议仍保留，因为阶段次数、gainable与失牌事件时序仍有差异。

## 输入

实际检查6个文件：`u01_batch_a.rs`、`u01_batch_a_asks.rs`、`archive/known_prompts.rs`、新增`u01_juxian_gain.rs`、`u01_ddd_eight.rs`、`tests/groups/g1.rs`。精确路径、SHA256、21份代码副本见 [implementation-inputs.json](implementation-inputs.json)；[implementation.diff](implementation.diff)保存实际已跟踪diff，新增测试全文另存副本。没有改写原27份冻结输入。

上游核对skill.js:244–397：`usable:2`及模式临时技能分别到`phaseUseAfter`；top展示付牌、`await player.lose`后找`countGainableCards`目标，331行强制`gainPlayerCard(...,"he",true)`，仅所获牌与支付牌异色才加ban。hand交牌给别人后自己摸1展示。进一步读实际`player.js:7246`和`content.ts:8896`的gainPlayerCard入口，确认一张强制选择、手牌暗示/已明牌处理和gainable候选来源。

## 逐项核对

| 核对点 | 实际代码与结论 |
| --- | --- |
| 选择分支 | A组原`he_cards(target).first()`路径被移除。top支付并压牌顶后设置`target`，转`HostStage::CardsAsk`，ask28。答案为`HiddenCard(slot)`时只映射目标手牌该位置；答案为物理`Card`时只接受目标装备。hand仍交给所选角色、自己摸1，经同一`finish_juxian`比较颜色。 |
| 续行载荷 | `HostMachine`原有`card=mode`、`cards=[paid]`、`target`、`stage`都进入现有`MachineSnapshot::Host`；`CardsAsk`是已有阶段tag，未增加存档字段。支付牌已在deck，机器里的paid是引用而非第二份持牌；普通Host不被`machine_held_cards`重复计入。 |
| 选择挂起 | `wait_ask`保存`u01_ask_id=28`，再保存阶段。内容hooks恢复相同Host字段；`snap::active_ask`经`host_ask`与A组`active_ask_u01_batch_a`重新用目标当前手牌/装备生成同序候选。`Engine::restore_with_catalog`逐字段比较observer、prompt、kind、options顺序、min/max/allow_cancel，并重建唯一答题机器。 |
| 选项可见性 | 复用`qinglong_options`：owner≠target时手牌仅`HiddenCard(0..n)`，装备以公开物理ID列出；observer是owner。新窗口没有把对方暗手牌物理ID或牌面放进ChoiceSpec。源码没有因此证明已明牌、观牌标签或所有产品客户端显示都等价；这些另列剩余边界。 |
| 非法应答 | ask28为`SelectCards`，min=max=1、不可取消。`ChoiceSpec::accepts`拒绝空答、非候选、越界暗槽、多选和重复；命令层先核对choice_id及observer，再校验selection，之后才取走open_choice写answer。步骤另核对装备归属及`u01_remove_owned`成功才入主人手。 |
| 保存恢复注册 | `dddjuxian_gain`只追加到KNOWN_PROMPTS；原列表字节及阶段/行为枚举未重排。通用owner/target/card/cards越界检查仍适用，重建候选也限制暗槽数量。未独立执行恶意档案变异或客户端序列化测试。 |
| AI信息范围 | 已读`Engine::plan_action`：自家开放选择走只读PolicyInput及公开选项的建议路径，通用`projected_choice_action`无对方手牌读入；本prompt没有新增私有牌面策略。新增换序手牌测试只检查动作一致，不能证明全部AI取向已复刻上游。 |
| 注册与兼容 | `tests/groups/g1.rs`登记新增测试模块；原eight测试增加回答gain窗口后再检查取牌/ban。其余新增diff中的君归、示烈换行及EOF属于格式变化，未见逻辑变动；没有触发元数据、规则号或NGSA字段修改。 |

## 已读测试及其界限

`u01_juxian_gain.rs`有3个测试函数。第1个从真实`ActivateSkill`经mode/card/target进入gain，分3次选择两张暗手牌的两个位置或公开装备，检查支付牌顶、禁止取消、物理ID猜测/暗槽越界被拒且state_hash不变；实际调用encode/decode_archive/restore，在两端回答同一选项，比较state_hash及新增日志，最后检查实体归属、异色ban及used=1。保存后的target手牌在等待期间不因其他命令变化，因为开放选择暂停结算栈。

第2个检查当前实现无他人he时不提供top，伪造Tag0被拒，取消mode不支付/不耗次。**其中“不提供top”固化了当前近似，不是上游负例证明**：上游模式过滤只看模式临时技能，支付后才检查gainable目标。第3个交换对方两张暗手牌次序，检查规划动作相同且答案在候选中。未测试所有装备类型、gainable屏蔽、已明手牌、第二出牌阶段或失牌触发先后。

`u01_ddd_eight.rs`只增加一次合法`HiddenCard(0)`回答，没有改旧结果期待；它检查原来唯一目标牌的兼容。新测试源中没有完整NGSA快照结构逐字段比较或错误observer专项断言；通用源码守卫可读到，不能把它们冒充本批新增回归。复核者未运行这些测试，因此不报告3/0或任何本增量游戏测试通过数。

## 明确保留的源码差异

1. 每模式本出牌阶段一次和总usable2仍用`turn_stamp`保存。相同回合额外出牌阶段不会像源`phaseUseAfter`清模式；本轮没有修这个边界。
2. 原生在付牌前要求有其他角色he，先选目标再支付；源先支付再以gainable构造目标集合，故无他人牌时源仍可置顶支付后结束。新测试只证明现有入口，不能当成等价。
3. `qinglong_options`枚举目标全部手牌/装备，没有source `getGainableCards`的不可获得过滤，也没有source已明牌或viewHandcard时展示牌面的分支。普通隐藏手牌/公开装备场景得到改善，完整候选语义仍有差异。
4. 源`await player.lose`结算完成后再询问目标及gain；原生`u01_remove_owned`发射CardsLost子帧后立即`WaitChoice`。`resolution.rs:598`开窗就返回Pending、暂停栈，所以gain候选和实际取牌发生在该失牌子触发之前。冻结版自动取牌也已在该子触发前执行；这是保留的事件链差异，不能声称此次新增选择已修复。例如支付失牌的其他技能会改变目标候选或手牌，源先反映变化，当前窗口不会先等这些触发。
5. 手续本身仍直接修改牌区，未补完整gain/give/move及展示历史事件。新窗口没有扩大这项来源等价签署；本报告也不验证跨来源增益触发与失去装备后的全部组合。

这些差异在 [implementation-conclusion.json](implementation-conclusion.json)按`retained_source_difference`登记；没有将未覆盖测试单独当成新阻断。64表中S04仍描述冻结HEAD，当前局部改进以本报告及其输入hash为准。
