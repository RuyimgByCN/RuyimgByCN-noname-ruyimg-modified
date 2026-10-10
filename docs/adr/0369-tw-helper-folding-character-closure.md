# ADR 0369：TW 周处、桥蕤 helper 折叠与角色闭包

TW 周处与桥蕤的生成角色清单额外挂载了仅为拆分触发器创建的内部 helper 技能，导致包
内角色技能列表与旧 JavaScript 源定义不一致，也让 helper 暴露为可见、可获得技能。

## 决策

- 将 `twchuhai_lose` 的移动牌触发折叠进源技能 `twchuhai`。
- 将 `twqiongji_draw` 的用牌/响应触发折叠进源技能 `twqiongji`。
- 从 `tw_zhouchu`、`tw_qiaorui` 的角色技能列表删除两个内部 helper ID。
- 不新增 ABI；复用同一技能内多触发器的既有 DSL 能力。

## 验证

黄金测试校验两名角色的技能列表与源定义完全一致，并确认包内不再发布两个 helper。
