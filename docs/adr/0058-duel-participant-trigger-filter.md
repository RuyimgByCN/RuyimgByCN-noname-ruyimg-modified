# ADR 0058：通用【决斗】参与者触发过滤器

- Engine semantics：42

旧版技能可同时监听自己使用【决斗】与成为【决斗】目标。该条件属于卡牌事件的通用关系判断，不应由技能 ID 或旧 JS 运行时实现。

引擎增加 `owner_source_or_targeted_by_juedou` 过滤器：拥有者存活，且是逻辑【决斗】的使用者或声明目标时匹配。过滤器复用 `CardUsed` 的逻辑牌类型、使用者和目标集合；多目标事件对使用者仍只产生一次触发。

`nsshuangxiong` 仅用 DSL 组合此过滤器与通用 `turn_over(self)` effect。旧版 `check` 是 AI 倾向，不改变玩家的合法发动资格。
