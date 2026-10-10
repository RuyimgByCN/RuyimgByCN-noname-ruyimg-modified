# ADR 0368：SP2 三名既有双技能武将闭包修复

SP2 的新杀 SP 关羽、张温与朱儁已标记为完整移植，但生成角色清单分别遗漏了一个旧
JavaScript 角色定义中的既有技能。

## 决策

- `dc_jsp_guanyu` 补回 REFRESH 的 `new_rewusheng`，保留本包 `dcdanji`。
- `zhangwen` 补回本包 `songshu`，保留 `sibian`。
- `zhujun` 保留 `gongjian` 并补回本包 `kuimang`。
- SP2 显式声明 `refresh` 直接依赖；不复制技能，不新增 ABI。

## 验证

黄金测试校验三名武将的完整技能顺序和直接依赖；全量角色闭包测试确认依赖提供者。
