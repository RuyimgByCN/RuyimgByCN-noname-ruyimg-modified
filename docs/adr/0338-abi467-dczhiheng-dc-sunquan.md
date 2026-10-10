# ADR 0338：ABI 467 制衡弃牌摸牌复合 + collab 经典孙权

本批为 collab 包经典孙权（`dc_sunquan`，2675325014）的「制衡」（`dczhiheng`，
2730395750）完整移植，需要 ABI 467 复合 `resolve_dczhiheng(player)`。

## 原语设计（ABI 467）

- DSL 表面：`resolve_dczhiheng(player)`；编译为 `effect_tag::RESOLVE_DCZHIHENG`
  （tag 387）。
- 引擎：`EffectRequest::ResolveDczhiheng { player, skill }`；阶段机（Custom 续体
  `RESOLVE_DCZHIHENG_CUSTOM_CHOICE`）：打开手牌/装备区多选弃牌（SelectCards，
  1..N，不可取消）→ 续体验证弃牌可弃且不重复 → 移入弃牌堆 → 摸等量牌；若弃置前
  手牌非空且全部被弃，额外摸一张（旧 JS `num` 逻辑）。

## 文档化近似

collab 特化机制「本回合对未命中过的其他角色造成伤害后，本回合内此技能可发动次数
+1」依赖每回合多段发动次数计数（DSL 暂无 usable 计数原语），文档化延后；基础
「限一次」语义完整保留。

## 打包与验证

- `dczhiheng.skill` 加入 collab README 显式列表；`dc_sunquan`（男/吴/4 体力，
  技能 [dczhiheng]）加入 collab `characters.json`；catalog 标记 `ported`。
- collab npack 重建（ABI 467，25 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/collab/`。
- 运行时测试 `original_collab_dczhiheng.rs`（3 用例全绿）：弃尽手牌（4 张弃 → 摸
  4+1）、保留手牌（弃 1 → 摸 1）、装备牌可弃（弃武器 → 摸 1）。
- 旧 JavaScript 不进入 npack 或运行时。
