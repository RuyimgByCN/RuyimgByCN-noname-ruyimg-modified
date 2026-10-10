# ADR 0437：ABI 554 兵势辛宪英

## 决策

将 `pot_xinxianying` 的诫节、清识完整重写为 DSL；旧 JavaScript 只保留为语义证据，
不进入 npack 或运行时。

ABI 554 新增 `resolve_potjiejie`、`resolve_potqingshi` 与出牌阶段调用过滤器。复用既有
私有手牌展示、临时技能、花色次数豁免、实体移牌、阵营关系和 Choice continuation；不新增
扩展加载协议。

## 状态与边界

- 诫节技能实例按轮保存每名角色历次展示花色的并集和本轮清识奖励次数；只比较当前存活角色，
  唯一最多时奖励，每轮至多两次。
- 当前出牌阶段角色调用一次后获得回合末自动清理的 helper；命中所选花色时授予对应花色的
  使用次数豁免并弃置其余花色手牌，未命中时从牌堆取得一张该花色牌。
- 清识使用身份模式权威阵营关系：同阵营双方各摸一张，异阵营依次弃置双方各一张手牌/装备牌。
- 编译统一使用外部 `CARGO_TARGET_DIR`，批次结束删除；仓库内 `target/` 不保留。

## 发布与验证

- `official/bingshi@0.1.0`：ABI 554，28 名武将、73 项技能、1 张卡；logic hash
  `f6c3250bf75688e1309c3d847e36abe9d255e43489537e377420b034295191b3`，文件 SHA-256
  `d34c0278d452cc10d809485400d1c638c242f4d9f54041bd77b4b019b5a7eff5`。
- 三份发布包逐字节一致，独立重打包一致；26 个扩展冷启动导入成功，ContentLock
  `83d8b7b5a626d38b5a138ecbfcaeaabf0e94f03686913381c4ccde30e623e61b`。
- 辛宪英 3 个 Golden、全部 27 个 Bingshi 测试文件、content-compiler 480 项、
  engine-api/rule-api/rule-vm/content-schema/content-runtime、conformance 与 engine-ai 检查通过。
