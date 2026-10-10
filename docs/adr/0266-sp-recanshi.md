# ADR 0266: SP 孙皓「残蚀」人数查询与回合临时惩罚

ABI 190 增加只读查询 `alive_wounded_or_guiming_wu_count(player)`，返回存活受伤角色与启用
主公技「归命」时其他存活吴势力角色的并集人数。查询复用标孙皓「残蚀」已有集合逻辑，但
返回真实零值；配套触发过滤器只在集合非空时开放可选摸牌窗口。

接受后复用 `increase_current_draw_count`、`add_skill`、`discard_one_hand_or_equip` 与
`remove_skill`，不增加专用效果或续体。临时技能监听拥有者使用逻辑【杀】或普通锦囊并强制
自弃一张，回合结束移除。SP 以「残蚀、仇海、归命」发布孙皓完整闭包；旧 JavaScript 只作
语义证据。
