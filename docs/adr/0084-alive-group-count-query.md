# ADR 0084：存活势力数查询

## 决策

DSL 新增整数查询 `alive_group_count()`，返回所有存活角色当前武将所属的非空势力去重数。查询复用引擎既有 `AliveGroupCount` 动态修正值的同一计算函数；旧版 JS 的 `game.countGroup()` 只作为语义依据，不进入新运行时。

`fuli` 使用该查询组合通用的回复、翻面和移除技能效果，不增加技能 ID 特判。

## 兼容性

新宿主查询使用追加的 query tag 18。Rule VM ABI 升至 6，ABI 6 主机继续加载 ABI 3/4/5 包；编译器仅在模块实际使用 `alive_group_count()` 时写入最低 ABI 6，现有包保持原最低 ABI。内容 Schema 保持 3，Rule IR 版本升至 12，清单 VM ABI 轴升至 3，引擎语义修订号升至 69。
