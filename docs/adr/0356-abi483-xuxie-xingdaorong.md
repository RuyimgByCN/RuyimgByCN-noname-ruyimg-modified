# ADR 0356：ABI 483 虚泄上限牺牲复合 + huicui 邢道荣

本批为 huicui 包邢道荣（`xingdaorong`，3475770596）的「虚泄」（`xuxie`，
3207964441）完整移植，需要 ABI 483 双复合。

## 原语设计（ABI 483）

- `resolve_xuxie(player)`：`trigger phase_play owner_is_target` → `ChangeMaxHp(-1)` →
  相邻判定（`QueryRequest::Distance ≤ 1`）→ 二选一 [0=弃各一张, 1=各摸一张]；
  弃牌分支按目标逐张弃置（Custom 续体 `RESOLVE_XUXIE_CUSTOM_CHOICE`，标记位推进）。
- `resolve_xuxie_add(player)`：`trigger phase_play_end owner_is_target`（PhaseEnded 事件）
  → 若存在 maxHp 大于你的存活角色 → `ChangeMaxHp(+1)`。

## 打包与验证

- `xuxie.skill` 加入 huicui README 显式列表；`xingdaorong`（男/群/6 体力，技能
  [xuxie]）加入 huicui `characters.json`；catalog 标记 `ported`。
- huicui npack 重建（ABI 483，31 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/huicui/`。
- 运行时测试 `original_huicui_xuxie.rs`（2 用例全绿）：弃牌分支（失 1 上限 + 逐目标
  弃一张 + 无更大上限者不恢复）；摸牌分支（相邻各摸一张）。
- 旧 JavaScript 不进入 npack 或运行时。
