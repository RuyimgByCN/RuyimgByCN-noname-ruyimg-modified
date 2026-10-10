# ADR 0423：ABI 543 兵势邓艾

本批把旧 JavaScript 中势邓艾（`pot_dengai`）的「屯田」「凿险」「急袭」完整重写为
DSL/Rust；旧 JavaScript 只作语义证据，不进入 npack 或运行时。

## 决策

- 「屯田」以技能状态保存蓄力上限、以技能标记保存当前蓄力：每个全局回合开始在蓄力已满时
  摸一并增加上限；因非伤害牌离开个人区域且未满时增加一点蓄力；出牌阶段可消耗任意蓄力，
  按所选角色顺序从牌堆、弃牌堆依次分配红桃牌。
- 「凿险」复用屯田消耗量：至少 2/5/7 点时依次从弃牌堆获得【无中生有】、【无懈可击】、
  【五谷丰登】；不复制旧 JavaScript 状态机。
- 「急袭」仅在技能实例中记录本回合曾被持有者用牌指定的其他角色。全局回合结束时先弃置
  当前回合角色一张 HE，再可对任意记录目标使用无距离、逐目标独立结算的虚拟【顺手牵羊】。
- 包继续使用现有 `*.skill + characters.json + cards.json -> npack -> extensions/bingshi`
  加载形式，不增加 JavaScript 执行或专用扩展加载层。

## 验证

- 势邓艾 Golden 3/3；兵势 23 个测试二进制共 75 项、content-compiler 477 项全部通过。
- content-schema、engine-api、rule-vm、content-runtime 与 8 项 conformance 全部通过；完整
  `extensions/` 冷导入激活 26 个包，ContentLock
  `e0f1a4ac6171e3d2da7b1dfb937e56ac5addc3368712bbcca131a1443a0e9f76`。
- Bingshi 包为 ABI 543、24 名武将、57 项技能、1 项牌定义；logic hash
  `2d5ea06c2cba197115f63348b3df5bac7d54809620be19e48b272261221bfc3b`，SHA-256
  `b564b345c0a597fc5a48aedd02d41abab09e06f44401861515779ceff21ce7df`；三份包字节一致且复打一致。
- 编译产物按架构计划 §37.10 处理：门禁结束后删除工作区 `target/` 与本批外部
  `CARGO_TARGET_DIR`，只保留源码、文档、npack 和扩展包。
