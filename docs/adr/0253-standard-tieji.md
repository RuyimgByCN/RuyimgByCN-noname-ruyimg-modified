# ADR 0253：标准铁骑判定后令当前杀目标不可响应

## 决策

ABI 177 增加
`judge_make_current_sha_target_unrespondable_on_red(player, card, target)`。STANDARD「铁骑」
在【杀】逐目标事件中可选发动；红色判定成功后，只关闭该目标对当前实体【杀】的响应窗口。

## 实现

效果复用统一判定和保存中的【杀】逐目标状态，不安装持久修改器。HOST_EFFECT 为 176，
`EffectRequest` 稳定哈希标签为 217，不增加 continuation。标准身份包没有「授钺」君主技；
其“非黑桃成功”分支应在国战迁移建立权威势力/君主技光环后补入，不能在标准包中伪造。
