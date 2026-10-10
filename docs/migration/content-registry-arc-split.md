# ContentRegistry Arc 化与克隆路径（第②部分第一步）

- 状态：共享目录与单副本已落地，缓存/日志仅部分覆盖；2026-09-16 复核。转场自 [ADR 0703](../adr/0703-native-kernel-first-batch.md) 第②部分
- 关联：`crates/engine-core`（旧线，原地改造）；验收记录见本文

## 影响面评估（动手前完成）

| 项 | 结论 | 依据 |
|---|---|---|
| canonical hash 是否编码 registry 本体 | **否** | `hash.rs` 只编码 `ruleset.logic_hash` 与实例级数据（`content_card_tags`/`skill content_storage`）；registry 的 `combined_logic_hash` 经由 ruleset 定义间接进入 |
| Arc 化对 golden 的影响 | **原改造记录为零漂移** | registry 本体不进哈希；原 agent 记录旧线 3,773 项通过，本轮未重跑全套；单平台测试不证明跨目标一致性 |
| ContentRegistry 的 `&mut` 面 | 仅 3 个方法 | `load_package` / `load_package_inner` / `recompute_combined_hash`，全部位于包安装路径（低频） |
| 可变调用点 | 状态字段上仅 `Engine::load_package` / `load_standard_package` 与结算内 `ensure_*_package`（3 处）+ 670 个测试文件的布阵语句 | 见下改造 |
| serde/快照 | 透明桥 | `#[serde(serialize_with/deserialize_with)]` 把 `Arc<CR>` 编码为 `CR` 本体——快照格式与历史兼容 |

## 已落地改造

1. **`GameState.content: Arc<ContentRegistry>`**：`GameState::clone` 时目录字节码/技能元数据**不再深拷贝**（引用计数 +1）。
2. **写时复制**：全部可变调用点改为 `Arc::make_mut(&mut ...content)`——安装时引用计数 >1 才复制，安装低频、结算只读，稳态零复制。
3. **serde 透明桥**：快照/录像格式不变，`restore_snapshot` 等测试原样通过。
4. **双克隆路径 → 单克隆**：`Engine::submit` 以 `mem::replace` 接管所有权传入 `apply_command`（消除克隆 #1），命令结束后无条件写回（接受 = 新状态；拒绝 = Transaction 回滚返回的基线，与原状态逐位等价）。`Transaction::begin` 内保留的**一次**克隆即评审认可的「紧凑对局状态的一份工作副本」——Arc 化后该克隆不再包含目录字节。
5. **缓存边界（部分）**：`query_dep_version()` 聚合提交版本与伤害/恢复/选择/RNG 哨兵；不覆盖未推动哨兵的字段写入，不能作为完整依赖声明。

## 验证

- 原 agent 的 2026-09-15 记录：`cargo test --offline -p engine-core` 3,773 passed；本轮未复验全套，不据此宣称跨目标通过。
- 2026-09-16 第七批前的历史复核：`cargo test --offline -p engine-native` 39 passed（38 项正式测试 + 1 项调试测试）；最新原生线结果见 [重写进度](../architecture/native-progress.md)。
- 同日实跑：`cargo test --offline -p engine-core --lib journal_equivalence_tests` 4 passed。
- 同日实跑：`cargo test --offline -p engine-core --lib cache_boundary_tests` 2 passed。

## 剩余边界与后续路线

- `Transaction::begin` 内的单克隆为回滚语义必需；**撤销日志（journal replay）**
  已于 2026-09-15 落最小实现与等价性测试（见下节），替代整份工作副本仍需按
  矩阵扩面后才可在生产路径启用。
- 实际使用 `std::sync::Arc`；引用计数可跨线程共享，但内容类型是否满足 `Send/Sync` 及多房间接入仍需验证。当前没有证明全进程共用一个目录。
- 快照反序列化会重建独立 `Arc`（无跨快照共享），符合预期。

## 撤销日志（journal replay）完备性评估与最小实现（2026-09-15）

### 完备性矩阵（journal 条目 × GameState 可变面）

| JournalEntry 变体 | 记录点现状 | 可精确撤销 | 状态 |
|---|---|---|---|
| `PlayerHp` / `PlayerMaxHp` / `PlayerLife` / `PlayerMarks` | primitives 分散记录（约 28 处） | 是（前值字段还原） | **已接 undo** |
| `CardMove` | 有记录 | 是（当前区移除 + prev 区按 prev_index 插回 + zone/owner/face_up 还原） | **已接 undo**（2026-09-15，等价性测试 `journal_rollback_equivalent_for_card_move`） |
| `ResolutionCheckpoint` / `TurnCheckpoint` / `OutcomeCheckpoint` / `RngCheckpoint` | **零记录点（死枚举项）** | 否（无前值载荷） | **降级报错**（`CheckpointEntry`） |
| `Marker` | 测试/内容用 | 无副作用 | 直通 |
| 其余全部可变面（hand_limit_bonus、group_overrides、judge 区、turn 旗标、zones 结构性变化、face_down_hand_cards 等） | **不经 journal** | — | **缺口**（生产路径必须保留基线回滚） |

### 评估结论

1. 现有 journal **不完备**：仅覆盖生命值/标记类少数路径，证实评审原文
   「现有日志尚不能直接替代完整回滚」；4 个 Checkpoint 变体为死枚举项。
2. 顺带发现：`hand_limit_bonus` **不在 canonical hash 编码内**
   （hash.rs 无该字段编码）——已登记为旧线哈希覆盖待核对项。
3. 因此 journal-only 回滚定位为**影子校验与逐步扩面机制**，生产回滚路径
   仍为基线克隆；两者由等价性测试持续对照。

### 最小实现与等价性测试

- `Transaction::rollback_via_journal()`：逆序撤销精确条目；
  Checkpoint 类条目显式返回 `JournalRollbackError`
  （方法消费事务，调用方必须另持完整基线才能回退）；未跟踪写入仍可能静默漏撤销，不能作为生产通用回滚。
  `CardMove` 已于 2026-09-15 补齐精确撤销（当前区移除 + prev 区按
  prev_index 插回 + zone/owner/face_up 还原）并退出降级清单。
- 等价性测试 4 项（`transaction::journal_equivalence_tests`）：
  ① 被跟踪字段（hp/marks）的 journal 重放与基线回滚**终态哈希相等**；
  ② 未跟踪字段缺口**必须**被等价性对照发现（防框架退化恒真）；
  ③ Checkpoint 条目阻断 journal 回滚并显式报错；
  ④ 卡牌移动精确撤销（区域列表位置/卡字段/哈希三方断言）。
- 扩面流程：新增/确认一条变更路径的 journal 记录 → 等价性测试扩大 →
  从矩阵移入「已接 undo」→ 全部移完后方可将生产回滚默认切至 journal-only。
