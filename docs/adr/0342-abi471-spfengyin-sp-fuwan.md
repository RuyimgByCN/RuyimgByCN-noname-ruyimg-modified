# ADR 0342：ABI 471 封印授杀跳阶段复合 + collab SP伏完

本批为 collab 包 SP伏完（`sp_fuwan`，3665805971）的「封印」（`spfengyin`，
3552373198）完整移植，需要 ABI 471 两个新原语：`skip_next_discard_phase(player)`
效果与 `resolve_spfengyin(player, target)` 复合。

## 原语设计（ABI 471）

- `skip_next_discard_phase(player)`：DSL 效果，设置 `turn_flag::SKIP_DISCARD`
  （弃牌阶段入口由回合机消费）；阶段合法域 Prepare/Judge/Draw/Play。
  编译为 `effect_tag::SKIP_NEXT_DISCARD_PHASE`（tag 392）。
- `resolve_spfengyin(player, target)`：`trigger phase_prepare owner_alive_not_target`
  （他人准备阶段；`event_player` 由 vm_host 捕获）。阶段机（Custom 续体
  `RESOLVE_SPFENGYIN_CUSTOM_CHOICE`）：
  - 守卫：目标体力 ≥ 拥有者体力（DSL 无 <= 比较，改在复合体内校验）；
  - 拥有者从手牌选一张【杀】（SelectCards，可取消）；
  - 交付该牌 + `SkipNextPlayPhase(target)` + `SkipNextDiscardPhase(target)`。

## 打包与验证

- `spfengyin.skill` 加入 collab README 显式列表；`sp_fuwan`（男/群/3 体力，技能
  [spfengyin]）加入 collab `characters.json`；catalog 标记 `ported`。
- collab npack 重建（ABI 471，26 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/collab/`。
- 运行时测试 `original_collab_spfengyin.rs`（3 用例全绿）：授杀后跳过出牌/弃牌阶段
  （SKIP_PLAY/SKIP_DISCARD 置位）、取消不授不跳、目标体力低于拥有者不触发。
- 旧 JavaScript 不进入 npack 或运行时。
