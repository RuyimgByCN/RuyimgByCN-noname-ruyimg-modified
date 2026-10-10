# ADR 0281：EXTRA 神关羽「新武魂」

旧 JavaScript 仅作为语义证据，不进入运行时。伤害结束后的等量“梦魇”使用既有 `add_mark`；
死亡结算新增一个共享效果，强制从正标记数并列最高的其他存活角色中选择一名，屏障恢复时重验
目标后执行实体牌判定。判定牌不是【桃】或【桃园结义】时，通过既有无来源 `KillPlayer` 结算。

该效果使用 HOST_EFFECT 199、续体 130、效果哈希标签 239；Rule VM ABI 为 205，
Rule IR 为 91，Engine semantics revision 为 179。
