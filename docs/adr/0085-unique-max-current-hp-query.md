# ADR 0085：唯一最高当前体力查询

## 背景

手杀「伏枥」在自己濒死时先将体力回复至存活势力数，然后仅在回复后的当前体力为全场唯一最高时翻面。旧 JS 的 `player.isMaxHp(true)` 在角色包内还有多处使用，不应固化成「伏枥」专用分支。

Rule VM 先运行字节码生成 effect 列表，再由引擎依次提交；因此同一段字节码中的普通查询不会看到尚未提交的回复 effect。

## 决策

追加两个只读查询：

- `unique_max_current_hp(player)` 读取当前状态，仅当该存活角色的当前体力严格大于所有其他存活角色时返回真。
- `unique_max_current_hp_at(player, value)` 不修改状态，以 `value` 作为该角色的假设当前体力与其他存活角色比较。

「伏枥」显式传入 `min(alive_group_count(), max_hp(self))` 的等价 DSL 分支，从而保持旧 `recoverTo` 的体力上限截断语义，且不改变 VM 的事务提交模型。

查询 tag 19/20 为追加编码。Rule VM ABI 升至 7，只有实际使用两项查询的包写入最低 ABI 7；`alive_group_count()` 继续要求 ABI 6，旧 ABI 3–6 包无需重打。内容 Schema 保持 3，Rule IR 版本升至 13，清单 VM ABI 轴保持 3，引擎语义修订号升至 70。

## 后果

并列最高不会被误判为唯一最高，死亡角色不参与比较，非法玩家 ID 返回稳定查询错误。预测查询不发射 effect、不消耗随机数、不绕过权威回复与翻面效果。
