# ADR 0276：YIJIANG 新当先与 REFRESH 界廖化

ABI 200 增加参数化效果
`lose_hp_gain_discard_card_for_opening_extra_play(player, card_kind, mandatory_skill_def)`。
效果只匹配 originating skill 创建的回合开头额外出牌阶段；指定技能定义仍由角色拥有时强制
失去一点体力，否则先提供可取消确认，接受后再失去体力，并从弃牌堆获得指定类别的一张牌。

「新伏枥」发动后已有 `remove_skill(self_skill)`，因此「新当先」直接用该技能实例是否仍存在
判断强制/可选分支，不新增旧 JavaScript `storage.xinfuli` 的平行状态。体力变化和取牌复用
现有 typed effect 管线；普通出牌阶段及其他技能创建的额外阶段为空操作。REFRESH 仅新增
`re_liaohua` 武将索引并跨包引用 YIJIANG 两个技能，旧 JavaScript 不进入 npack 或运行时。
