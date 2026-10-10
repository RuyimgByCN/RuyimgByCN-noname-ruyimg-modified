# ADR 0263: STANDARD「激将」主动/响应势力代打【杀】

ABI 187 将参数化势力来源扩展为 `view_as sha any any from group_<tag>`。技能拥有者需要主动使用
或响应【杀】时，引擎从下一座起依次询问其他存活且势力匹配的角色；首名响应者按普通实体/视为
【杀】规则消耗自己的牌，拥有者随后在原上下文中使用或响应逻辑【杀】。

主动使用复用 continuation 121 并以 action 6 保存目标列表；响应继续复用已有
`SourceFreeViewAsResponse` action 2/3。全部拒绝时按南蛮入侵、决斗或主动使用的原失败分支继续。
主动失败使用技能实例状态记录下一命令修订号，只阻止同一旧事件的立即重试；其他牌、技能或阶段
动作完成后自动解锁。未新增 HOST_EFFECT、`EffectRequest` 或 continuation。刘备以「仁德、激将」
完整发布；旧 `jijiang1`、`jijiang3` 续体助手折叠进权威 `jijiang`，旧 JavaScript 不执行。
