本批原授权允许仅修复默认 parent 无授予 Damage 可达性，并保留 active Sha；完整 loseEnd-use 无法忠实表达时明确 pending。不加 damage 到来源 group，不支持未提供实际声明的显式 damage grant。262/290不变。

执行中发现两个既有真实恢复缺陷，均先留下实际失败：
1. xuanying_target 解码 BadTag(prompt_unknown,0)。Parent 明确允许仅在原静态字典尾追加这个现有生产 literal；保留所有旧项/编码/格式，不能批量登记其他未知prompt。
2. 登记后 decode 成功，但 Engine::restore 报 OpenChoiceMismatch。Parent 明确允许只补 SwdXuanying+既有TargetChoice 的准确 expected-ask 派生，复用与实际producer相同的ChoiceSpec helper；其余stage不得伪派生，wrong options/observer/prompt/其他字段必须仍拒。
两项授权均是既有功能缺陷修复，未授权新ChoiceKind/HostStage/机器字段/State/协议形状/档格式/版本，也没有放宽恢复总校验。

实际字典按 UTF8 string 编码，不是数字prompt ID；验证旧数组是新数组完整前缀，且文件唯一变更为尾部一个既有literal。
未使用 typed-memory restore 代替 byte restore。实际确认/取消分别 byte decode+Engine restore，再比较完整save（auth_generation按既有规则归一化）、state hash与全部后续日志；Core中的RNG/choices/完整resolution随save一并比较。

仍缺的边界：一个可确定“全次牌损失已完成”的loss-end/batch窗口。take_hand_card、hand_to_off_court没有统一CardsLost；give_hand_cards按张发CardsLost；CardsMoved/CardsDiscarded不能无重复地拼成全部来源loseEnd。暂不把xuanying_use错误接到任意部分事件上，不分配新持久状态/接口。
