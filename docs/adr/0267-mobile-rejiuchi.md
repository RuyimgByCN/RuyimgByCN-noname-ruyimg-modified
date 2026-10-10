# ADR 0267: MOBILE「酒池」按定义临时禁用技能

ABI 191 增加通用效果 `disable_skill_by_def_until_turn_end(player, skill_def)`。效果在保留技能
实例、定义和状态的前提下设置回合禁用位，并立即重建触发器/修改器索引；下一回合开始清除
禁用位并再次注册。未采用“移除后重加”，因为那会丢失实例状态并错误恢复本应被其他效果移除的
技能。

参数化过滤器 `owner_source_jiu_boosted_sha_damage_skill_<def>_enabled` 只在拥有者以酒加成的
逻辑【杀】实际造成伤害且指定技能仍有效时触发。MOBILE「酒池」复用既有黑桃视为【酒】协议，
临时禁用 SHENHUA「崩坏」，并与「肉林、暴虐」组成界董卓完整闭包；旧 JavaScript 只作语义
证据。
