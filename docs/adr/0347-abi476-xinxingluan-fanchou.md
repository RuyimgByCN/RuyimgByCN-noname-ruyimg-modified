# ADR 0347：ABI 476 心星乱三点数6分支复合 + sp2 樊稠

本批为 sp2 包樊稠（`fanchou`，2704452251）的「心星乱」（`xinxingluan`，
3806437757）完整移植，需要 ABI 476 复合 `resolve_xinxingluan(player)`。

## 原语设计（ABI 476）

- DSL 表面：`resolve_xinxingluan(player)`；`trigger card_used owner_source_once_per_turn`
  （新增 registry 别名映射 `OwnerSourceOncePerTurn`，`turn_limit` 识别 → 每回合一次）。
  编译为 `effect_tag::RESOLVE_XINXINGLUAN`（tag 400）。
- 引擎：`EffectRequest::ResolveXinxingluan { player, skill }`；阶段机（Custom 续体
  `RESOLVE_XINXINGLUAN_CUSTOM_CHOICE`）三分支：
  - ①牌堆扫描至多两张点数 6 → 获其一（无则摸六张兜底）；
  - ②目标（he 非空）→ 获得其装备/判定区一张点数 6；
  - ③目标（he 非空）→ 目标自选弃一张点数 6 手牌，或（无点数 6 / 取消）交给
    拥有者一张牌。

## 打包与验证

- `xinxingluan.skill` 加入 sp2 README 显式列表；`fanchou`（男/群/4 体力，技能
  [xinxingluan]）加入 sp2 `characters.json`；catalog 标记 `ported`。
- sp2 npack 重建（ABI 476，46 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/sp2/`。
- 运行时测试 `original_sp2_xinxingluan.rs`（4 用例全绿）：夺牌分支（点数6装备到手）、
  弃牌分支（目标弃点数6手牌）、牌堆分支（点数6牌入手）、每回合一次（第二次用牌
  无分支）。
- 旧 JavaScript 不进入 npack 或运行时。
