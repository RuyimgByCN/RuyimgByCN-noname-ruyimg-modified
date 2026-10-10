# ADR 0359：ABI 484 驰应循环弃牌夺基复合 + huicui 高翔

本批为 huicui 包高翔（`gaoxiang`，2285365687）的「驰应」（`dcchiying`，
4106504641）完整移植，需要 ABI 484 复合 `resolve_dcchiying(player)`。

## 原语设计（ABI 484）

- DSL 表面：`resolve_dcchiying(player)`；`trigger phase_play owner_is_target`。
  编译为 `effect_tag::RESOLVE_DCCHIYING`（tag 415）。
- 引擎：`EffectRequest::ResolveDcchiying { player, target, stage, accum, skill }`；
  阶段机（Custom 续体 `RESOLVE_DCCHIYING_CUSTOM_CHOICE`）：
  - 阶段 1：选目标（体力 ≤ 你，可取消）；
  - 阶段 2：目标在其攻击范围内（`AttackRange`/`Distance` 查询）选择一名未选过且有
    可弃牌的存活角色（可取消停止）→ 阶段 3 选择其一张牌弃置并累计 → 重发阶段 2；
  - 停止：目标获得累计中被弃置的全部基本牌（`is_basic` 过滤）。

## 打包与验证

- `dcchiying.skill` 加入 huicui README 显式列表；`gaoxiang`（男/蜀/4 体力，技能
  [dcchiying]）加入 huicui `characters.json`；catalog 标记 `ported`。
- huicui npack 重建（ABI 484，32 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/huicui/`。
- 运行时测试 `original_huicui_dcchiying.rs`（1 用例全绿）：目标循环弃其他玩家之
  【杀】并停止后获得该【杀】（【桃】留在原处）。
- 旧 JavaScript 不进入 npack 或运行时。
