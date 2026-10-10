# ADR 0346：ABI 475 蛮智双分支复合 + huicui 孟优

本批为 huicui 包孟优（`mengyou`，3321725449）的「蛮智」（`dcmanzhi`，
4194306664）完整移植，需要 ABI 475 复合 `resolve_dcmanzhi(player)`。

## 原语设计（ABI 475）

- DSL 表面：`resolve_dcmanzhi(player)`；`trigger phase_prepare/phase_end owner_is_target`。
  编译为 `effect_tag::RESOLVE_DCMANZHI`（tag 399）。
- 引擎：`EffectRequest::ResolveDcmanzhi { player, skill }`；阶段机（Custom 续体
  `RESOLVE_DCMANZHI_CUSTOM_CHOICE`）：
  - 分支可用性：每分支每回合一次，以技能 mark（`DCMANZHI_GIVE_MARK` 500000001 /
    `DCMANZHI_GAIN_MARK` 500000002）记录；准备阶段先清零（跨回合重置）；
  - ①授牌分支：选目标（he 非空）→ 目标自选至多两张 he 交给拥有者 →
    `UseVirtualCardOn(target, Sha, owner)`（无距离）→ 置授牌 mark；
  - ②夺牌分支：选目标（hej 非空）→ 拥有者自选 1-2 张 hej 获得 → 自选等量手牌/
    装备交回 → 摸一张 → 置夺牌 mark。

## 文档化近似

结束阶段触发前的「本回合体力未变化」频率守卫省略；目标的牌移交由目标自选（旧 JS
同）。

## 打包与验证

- `dcmanzhi.skill` 加入 huicui README 显式列表；`mengyou`（男/群/5 体力，技能
  [dcmanzhi]）加入 huicui `characters.json`；catalog 标记 `ported`。
- huicui npack 重建（ABI 475，24 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/huicui/`。
- 运行时测试 `original_huicui_dcmanzhi.rs`（3 用例全绿）：授牌分支（目标交两张 +
  虚拟【杀】命中）、夺牌分支（夺 2 交 2 摸 1）、每分支每回合一次（准备用授牌后
  结束阶段仅剩夺牌可选）。
- 旧 JavaScript 不进入 npack 或运行时。
