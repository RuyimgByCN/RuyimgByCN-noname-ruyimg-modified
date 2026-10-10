# ADR 0413：ABI 535 兵势鲁肃「缔盟」

本批将旧 JavaScript 中的兵势鲁肃「缔盟」（`potdimeng`，2916721654）重写为
DSL/Rust 规则并发布进兵势 npack；旧 JavaScript 只作语义证据，不进入运行时。
兵势鲁肃武将暂不发布，待「好施」共享手牌语义闭环后再加入角色表。

## 决策

- 主动目标增加通用 `pair_hand_difference_lte_three` 约束，服务端在提交时重新校验
  恰好两名角色且手牌数差不超过三。
- 复用现有实体移牌管线原子交换两名目标的全部手牌；交换后按拥有者已损失体力，
  打开精确弃置手牌/装备牌或令手牌较少目标摸牌的选择屏障。
- Choice 恢复时重新校验技能实例、弃置牌所有权和可弃置性；目标死亡时不再摸牌。

## 验证

- Golden 覆盖非法手牌差拒绝、整手交换、取消弃牌改为目标摸牌、选择弃牌四条路径。
- content-compiler 474/474、兵势 21 个测试二进制 66 项、engine-api 4/4、
  rule-vm 10+5、conformance 8/8 全部通过。
- `bingshi.npack.json` 为 ABI 535、21 名武将、50 项技能，logic hash
  `9d86d72c2dfa676e7d03e38e87e8ebc78ae4f23922fd91a4fd6225633aa0027f`，SHA-256
  `417edfaf5a96d3744ee48995ea1808e62120a371ead87f300f7dfbcb7f1ce556`；同步到
  `extensions/bingshi/`，不包含 JavaScript。
- 完整 `extensions/` 冷导入激活 26 个包，ContentLock
  `1b298bc67c75593be8fbe35864bf14e4153960bd03db4a8229712e8e33b4163f`。
- 编译产物按架构计划 §37.10 使用统一外部 target，门禁结束后删除。
