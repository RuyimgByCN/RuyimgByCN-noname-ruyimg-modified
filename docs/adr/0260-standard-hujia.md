# ADR 0260: STANDARD「护驾」势力协作响应

ABI 184 增加参数化视为牌来源 `from group_<tag>`。首个支持的声明为
`view_as shan response any from group_wei`。

技能拥有者先通过已有无来源响应令牌选择发动；引擎随后从其下一座开始依次询问其他存活且势力
匹配的角色。首名响应者按普通实体/视为【闪】规则消耗自己的牌并产生响应事件，之后拥有者在原
【杀】或【万箭齐发】响应上下文中视为打出【闪】。全部拒绝则沿原上下文失败分支继续。
选择使用 continuation 121；未新增 HOST_EFFECT 或 `EffectRequest`。曹操以「奸雄、护驾」完整
发布，`lord_only` 由运行时身份校验，旧 JavaScript 不执行。
