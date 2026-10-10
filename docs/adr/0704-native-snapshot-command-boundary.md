# ADR 0704：挂起机器规范快照与命令信任边界——engine-native 摘要可区分与保存恢复

- 状态：已接受（2026-09-16）。
- 复核：第七批后的版本与实例 owner 两项缺口，已在同日第八批修复并通过回归测试（[ADR 0705](0705-native-semantics-nullify-identity.md)）；其他恢复结构校验仍未完成。
- 后续（2026-09-17 补注）：第九批增加结构检查与七案例，第十批修复三个遗漏并扩为九案例，见 [ADR 0706](0706-native-zone-conservation-restore-matrix.md)、[ADR 0707](0707-native-structure-slice-matrix.md)。本文描述与测试数为第七批历史范围，当前验收与分片失败限制统一见重写进度。
- 关联：`crates/engine-native/`（`src/snap.rs` 新增模块）；前置决策见
  [ADR 0703](0703-native-kernel-first-batch.md)；进度与剩余缺口见
  [重写进度](../architecture/native-progress.md) 与
  [覆盖清单](../native-coverage-manifest.md)。

## 背景

六批基础机制后的第七批，处理上一轮进度文档列出的两个 P0 缺口：

1. **挂起态摘要不完整、无序列化恢复**：`canonical_hash` 对挂起机器（parks）
   只编码恢复深度与归属槽位，不编码机器内部状态——两台局部状态不同的机器
   哈希相同；暂停点没有保存/恢复手段（只有 `fork()` 克隆）；摘要未与规则
   版本绑定，跨规则版本的哈希可能互认。
2. **原型命令不是外部信任边界**：`GameCommand` 不携带发起者、预期修订号与
   去重标识；应答他人选择、过期命令、重放命令均可被处理；越界目标 ID 会
   直接进入 `player()` 索引 panic；同一选项应答两次可使下游二次移除同一
   牌实例 panic。

## 决策

### 1. 机器规范快照（`MachineSnapshot`）

- 新模块 `snap.rs` 定义穷举机器类型的类型化快照枚举（第七批时为内核 8 台 +
  可选触发闸门 + 内容样例 5 台，闸门/无懈窗口的嵌套机器递归快照）。
- `SkillMachine` trait 新增 `fn snapshot(&self) -> MachineSnapshot` 契约；
  每台机器导出自己的**全部**局部状态。快照同时是：
  - 摘要编码的数据源：`canonical_hash` 对每台挂起机器编码
    「判别符 + 全字段」，不同机器的局部状态在哈希中可区分；
  - 暂停点恢复的数据源：`MachineSnapshot::restore()` 重建机器实例
    （内容机器经 `content::standard::restore_content_machine` 分派）。
- 恢复用构造器与运行时构造器共用（`pub(crate) fn new`），不存在第二套
  初始化路径。

### 2. 规则版本绑定

- `Catalog.ruleset_version`（内容目录构建期赋值，标准目录常量
  `STANDARD_RULESET_VERSION`），开引擎时固化进 `GameStateCore`。
- 版本编码进 `canonical_hash` 首段：跨规则版本的哈希不互认。
- 版本随 `StateSnapshot` 保存：`Engine::restore` 拒绝外层版本与目录不符的快照
  （`SnapshotError::RuleVersionMismatch`）；第八批补齐核心版本校验，目录、外层与核心版本须一致。

### 3. 保存/恢复（`Engine::save` / `Engine::restore`）

- `StateSnapshot = { ruleset_version, core, resolution(含机器快照), last_dedup }`，
  为内存态类型化结构；**字节级落盘序列化不在本批**（平台/持久化阶段交付）。
- 恢复前已检查外层规则版本、核心牌区/技能键/当前玩家、机器直接 ID 字段与
  开放选择的观察者/选项 ID，越界返回 `SnapshotError::IdOutOfBounds`。
  第八批补齐技能实例值 owner 与键的一致性校验；**并非全部 ID 或结构关系均已校验**，
  完整身份配置、闸门事件载荷和部分栈/选择关系仍有遗漏（第九批已补身份长度及部分结构检查）。
- 宿主日志（展示/录像）不进快照；恢复后的引擎从空日志开始。
- 命令去重高水位 `last_dedup` 属引擎会话数据：不入状态、不入哈希，但随
  快照延续，恢复件不接受恢复前已接受过的键。

### 4. 命令信任边界（`GameCommand` 信封）

- `GameCommand { actor, expected_revision, dedup_id, action }`，校验顺序
  即拒绝优先级：
  1. `dedup_id`：必须为正且严格超过已接受命令的高水位
     （`DuplicateCommand`）；被拒绝的命令不消耗键，允许同键换修订号重试；
  2. `expected_revision` 与当前修订号不符 → `StaleRevision`；
  3. `actor` 超出座位范围 → `UnknownActor`；
  4. 终局/选择挂起门禁（沿用）；
  5. 动作授权与参数：Answer 仅限选择观察者本人（`WrongPlayer`）；
     PlayCard/NextPhase 仅限当前回合玩家（`NotYourTurn`）；目标 ID 越界
     显式 `IllegalTargets`、越界牌 `CardNotInHand`；现有测试覆盖所列非法输入，
     不据此宣称所有构造状态或入口均不会 panic。
- `ChoiceSpec::accepts` 增加重复守卫：同一选项出现两次的应答拒绝——
  堵住「同牌选两次 → 下游二次移除同一实例」的 panic 洞。
- 调试入口（`debug_*`、`debug_deal_damage`）是测试/工具通道，**不在**
  信任边界内，维持上帝视角语义。

### 5. golden 全量重录（登记于覆盖清单维护约定第 4 条）

规则版本入哈希（影响全部场景）+ 挂起机器快照入哈希（影响
`rollback_invariants` 的挂起终态）属状态哈希编码扩展；14 个 golden 同批重录。
第七批时 36 项机制测试通过，只证明已断言的场景，不证明全部规则语义不变。

## 第七批验证范围（2026-09-16 历史记录）

- 命令：`cargo test --offline -p engine-native`；最新总数与结果统一见 [重写进度](../architecture/native-progress.md)。
- 36 项机制测试：通过。
- 15 项新边界/快照测试（`tests/boundary_snapshot.rs`）：错误玩家/越界
  actor/过期修订/重复命令/越界目标与牌有拒绝断言，部分用例另比对状态哈希；
  重复选项由 ChoiceSpec 单元级断言验证。一个静止点与散财链三个选择点
  的保存→恢复→继续与无中断线终态哈希一致；未穷举全部机器的暂停状态。
  外层版本不符、指定 ID 越界拒绝恢复；机器局部状态参与摘要。
- `src/engine.rs` 的 2 项单元测试：机器局部状态与规则版本变化会改变摘要，已计入总数。
- 2 项场景测试（14 个 golden，重录后）+ 1 项调试测试：通过。

## 限制与后续

- 快照为内存态结构；serde/字节落盘、跨进程恢复属阶段 4（平台与持久化）。
- 去重高水位按引擎实例单调；多宿主并发/乱序投递语义待服务器阶段细化。
- 第七批复核曾发现：仅修改快照的 `core.ruleset_version` 仍获恢复成功，
  但恢复件再次保存/恢复会因版本不符失败；4 人快照中技能实例值的 `owner=255`
  也未被拒绝。**第八批已修复这两项**，回归测试与当前剩余工作见 ADR 0705 和重写进度。
- 第九批已加入部分栈帧、深度与槽位检查；OptionalGate 事件载荷、自引用父帧和选择/机器关联仍待补齐。
  现阶段仅处理可信引擎产生且未经修改的快照；补齐一致性校验和各机器暂停状态测试，
  服务端接入时另行明确快照输入与认证边界。
- 第八批已开始模式语义收敛；调度时间片、官方内容迁移等后续工作见重写进度，勿将本批历史范围当作当前状态。
