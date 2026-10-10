# ADR 0341：ABI 470 思变展示分配复合 + sp2 张温

本批为 sp2 包张温（`zhangwen`，2318285269）的「思变」（`sibian`，3021138179）
完整移植，需要 ABI 470 复合 `resolve_sibian(player)`。

## 原语设计（ABI 470）

- DSL 表面：`resolve_sibian(player)`；`trigger phase_draw owner_is_target`。
  编译为 `effect_tag::RESOLVE_SIBIAN`（tag 391）。
- 引擎：`EffectRequest::ResolveSibian { player, skill }`；阶段机（Custom 续体
  `RESOLVE_SIBIAN_CUSTOM_CHOICE`）：
  - 守卫：摸牌阶段、无既有固定摸牌数（`draw_phase_fixed_count`）；
  - 取牌堆顶至多 4 张并 `RevealCards`；按点数降序，获得点数等于最大值或最小值的
    牌（移入拥有者手牌），其余为中间牌；
  - 以 `set_draw_phase_fixed_count(0)` 跳过本次摸牌（摸牌阶段入口先查固定数）；
  - 中间牌非空时选择一名手牌数最少的角色交付；取消则置入弃牌堆。

## 文档化近似

牌堆不足四张时取现有张数（旧 JS `get.cards(4)` 会洗牌补足）。

## 打包与验证

- `sibian.skill` 加入 sp2 README 显式列表；`zhangwen`（男/群/3 体力，技能
  [sibian]）加入 sp2 `characters.json`；catalog 标记 `ported`。
- sp2 npack 重建（ABI 470，41 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/sp2/`。
- 运行时测试 `original_sp2_sibian.rs`（2 用例全绿）：极值获得 + 中间牌交付
  手牌最少者 + 固定摸牌数清零；取消时中间牌入弃牌堆。
- 旧 JavaScript 不进入 npack 或运行时。
