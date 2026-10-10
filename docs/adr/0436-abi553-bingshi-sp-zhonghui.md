# ADR 0436：ABI 553 兵势钟会

## 决策

将 `sp_zhonghui` 的俟资、挟志、狱难、克昌重写为 DSL，并以单一
`resolve_mbzhonghui` 状态机承载必须串联的蓄力选择、永久成长、觉醒和升级语义。

复用既有蓄力标记、回合内【杀】使用记录、绑定伤害、势力覆盖、技能授予与当前【杀】
不可响应管线；仅新增 ABI 553 effect 与查询值源
`queried_player_qun_owner_lord_alive`，不引入 JavaScript 运行时或新扩展加载协议。

## 状态

- 俟资：技能状态保存剩余结算回合数和超体力消耗标志。
- 挟志：首个 `u32` 保存永久手牌上限及【杀】次数增量。
- 狱难：保存本轮死亡记录及觉醒位；觉醒后势力改为群并升级克昌。
- 克昌：首字节保存升级位；主公存活时群势力角色使用【杀】无距离限制。

编译统一使用单一外部 `CARGO_TARGET_DIR`，批次结束删除；仓库内 `target/` 不保留。

## 发布与验证

- `official/bingshi@0.1.0`：ABI 553，27 名武将、66 项技能、1 张卡；logic hash
  `7132da8091e85b7e8c58c7ec7b49dafac5db381bd27cf6d89a63679c81d226d4`，文件
  SHA-256 `02160b4e218474ba7398f7b833551221214c84a6dc6cdc0a5dbcb13536fc031c`。
- 三份发布包逐字节一致，独立重打包一致；26 个扩展冷启动导入成功，ContentLock
  `0968f09bc5881eb73539aa6f0315bfe93b85e3f5445f4848eecc2b9cfdb19795`。
- 钟会 3 个 Golden、全部 26 个 Bingshi 测试文件、content-compiler 479 项、
  rule-api/rule-vm/content-schema/content-runtime、conformance 与 engine-ai 检查通过。
