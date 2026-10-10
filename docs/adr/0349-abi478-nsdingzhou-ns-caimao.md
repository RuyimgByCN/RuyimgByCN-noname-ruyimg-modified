# ADR 0349：ABI 478 定州取牌摸牌比色复合 + diy 蔡瑁

本批为 diy 包蔡瑁（`ns_caimao`，2895316778）的「定州」（`nsdingzhou`，
2793593832）完整移植，需要 ABI 478 复合 `resolve_nsdingzhou(player)`。

## 原语设计（ABI 478）

- DSL 表面：`resolve_nsdingzhou(player)`；`trigger phase_play owner_is_target`（每回合
  一次）。编译为 `effect_tag::RESOLVE_NSDINGZHOU`（tag 404）。
- 引擎：`EffectRequest::ResolveNsdingzhou { player, skill }`；阶段机（Custom 续体
  `RESOLVE_NSDINGZHOU_CUSTOM_CHOICE`）：选目标（hej 非空）→ 获得其 hej 首张
  （`discardable_target_cards` 确定性取首，近似旧 JS `randomGet`）→ 摸一张 → 比色
  （花色奇偶），颜色不同则失去 1 点体力（`DealDamage is_lose_hp`）。

## 打包与验证

- `nsdingzhou.skill` 由 diy README 的 `*.skill` glob 收录；`ns_caimao`（男/群/4 体力，
  技能 [nsdingzhou]）已在 diy `characters.json`；catalog 标记 `ported`。
- diy npack 重建（ABI 478，24 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/diy/`。
- 运行时测试 `original_diy_nsdingzhou.rs`（2 用例全绿）：获得目标首张（黑桃）并
  摸一张（体力按比色结果至多 -1）；取消无效果。
- 旧 JavaScript 不进入 npack 或运行时。
