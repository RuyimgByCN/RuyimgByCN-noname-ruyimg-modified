# ADR 0261: STANDARD「救援」濒死响应增量回复

ABI 185 增加参数化过滤器
`owner_dying_other_tao_responder_group_<tag>` 与
`recover_hp_as(source, player, amount)`（HOST_EFFECT 182）。

共享求桃链路现在为实体牌和无来源视为牌统一发出 `CardResponded`，并把濒死响应视作中断事件，
使响应触发先于基础回复及后续求桃结算。孙权的锁定主公技监听其他吴势力角色对自己打出的【桃】，
以该响应者为来源额外回复 1；随后仍执行原本的 1 点回复。未新增 `EffectRequest` 或 continuation，
旧 JavaScript 不执行。
