# ADR 0362：ABI 486 相术猜数复合 + huicui 张闿

本批为 huicui 包张闿（`zhangkai`，2895942842）的「相术」（`dcxiangshu`，
3579538404）完整移植，需要 ABI 486 双复合。

## 原语设计（ABI 486）

- `resolve_dcxiangshu(player, target)`：`trigger phase_play owner_alive_not_target`
  （vm_host 捕获 `event_player`）→ 目标手牌数 ≥ 体力值守卫 → 选择秘密数字 0-5
  （Custom，可取消）→ `AddMark(owner, DCXIANG_SHU_NUMBER_MARK, number)`。
- `resolve_dcxiangshu_lottery(player, target)`：`trigger phase_play_end
  owner_alive_not_target`（PhaseEnded 事件）→ 按 |手牌数 - 数字| 结算：
  ≤1 → 获得其一张 he（选择）；==0 → 额外造成 1 点伤害；结算后移除数字标记。

## 文档化近似

旧 JS 的「弃一张手牌不公布数字」保密步骤省略（数字以技能 mark 记录，权威性不受
影响）。

## 打包与验证

- `dcxiangshu.skill` 加入 huicui README 显式列表；`zhangkai`（男/群/4 体力，技能
  [dcxiangshu]）加入 huicui `characters.json`；catalog 标记 `ported`。
- huicui npack 重建（ABI 486，35 模块），README 构建命令逐字节复现（`cmp` 通过），
  部署至 `extensions/huicui/`。
- 运行时测试 `original_huicui_dcxiangshu.rs`（1 用例全绿）：精确猜中（数字=手牌数）
  → 获得一张 + 造成 1 点伤害。
- 旧 JavaScript 不进入 npack 或运行时。
