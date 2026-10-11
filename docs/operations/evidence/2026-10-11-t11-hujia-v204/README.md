# 2026-10-11 T11 护驾子项：自选代出+倾国转化（规则 v204）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（本批为 v202/v203 标准差异消除批延续）。上游依据：`standard/skill.js` hujia:246——需闪时逐魏将 `chooseToRespond`（filterCard: shan，转化技可用），任一人打出即置 `trigger.result`，牌经响应管线消耗。

## 语义修正

- 原生改前：逐魏将 Confirm/Pass 询问 → 确认后**自动弃置第一张真闪**（持牌者无选择权、转化技不可用）——登记差异「弃置+计数而非完整 respond 通道」。
- 改后（v204）：询问改 SelectCards——持牌者**自选**一张可用闪（真闪 + 倾国转化黑手牌；选项只含持牌者自己的手牌，他人暗牌 id 不出现），空选=拒绝本位继续问下一位（`allow_cancel=false`+`min=0`，空选择非取消语义）；选中仍经 `discard_from_hand`+`emit CardResponded`，无双计数不变。
- 撤销登记差异（选择权+转化面）；保留既有惯例：代出转化牌以实体牌响应（上游 result 为虚拟闪——虚拟→实体惯例同族）。

## 验证

| 项 | 结果 |
|---|---|
| 回归（native_kernel 护驾 6/6） | `n08_hujia_borrows_shan_from_others`（自选真闪/倾国黑手牌入选项+红色牌不入/空选拒绝→伤害照常）、`hujia_asks_wei_helpers_in_seat_order_and_continues_after_pass`（座位序+空选继续）、`hujia_shan_counts_toward_wushuang_requirement`（自选代出计无双第一张）、`hujia_pending_ask_byte_roundtrip_restores`（挂起态字节往返+自选应答恢复）、`hujia_skips_non_wei_helpers`、`hujia_public_asks_do_not_leak_hidden_shan`（视图等价不变） |
| 版本回退探针（204→203 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段；护驾不在标准场景可达路径） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 `cargo test --offline -p engine-native --no-fail-fast` | **549/0/1**（2026-10-11 本机） |
| Gate 1b | `RUSTFLAGS="-D warnings" cargo check -p engine-native --lib` exit 0 |
| 规则版本 | 203→**204**（存档格式不变 v47——无状态新增，仅询问形态变化） |
