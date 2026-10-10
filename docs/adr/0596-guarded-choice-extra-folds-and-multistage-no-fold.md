# ADR 0596：GuardedChoice 增补折叠（鸿举/断臂）+ 多段 continuation 不折决定

## 背景

ADR 0595 引入 `GuardedChoice` builder，把「守卫 → 候选集 → 单个 `open_choice`」
三段式里的守卫与 `open_choice` 样板折叠进 builder，并折叠了 21 处**已经使用**
`GuardedChoice` 的调用点。0595 明确：候选集由调用方计算，builder 只收守卫与开选择的样板。

本 ADR 处理两件接手事项：

1. **增补折叠**：0595 只收了已用 builder 的调用点，没收**同形但仍在用裸 `open_choice`**
   的函数。扫描发现 8 个「单实质 stage + 裸 `open_choice` + `_` 报错臂」的候选，
   它们与 0595 的折叠目标完全同构。
2. **多段 continuation 不折决定**：任务原计划（架构整理 Task1）提出「参数化多段
   continuation」——为 `primitives/` 下带 `match stage` 状态机的 `resolve_*` 定义通用
   stage/payload 布局并参数化。本 ADR 经逐行审计后**否决该计划**。

## 增补折叠：逐个审计结果

8 个候选（单实质 stage + 裸 `open_choice`）逐个读完整代码：

| 函数 | 判定 | 理由 |
|---|---|---|
| `resolve_hongju` | **已折** | 业务 guard（荣标记≥3、未觉醒）外提；技能启用+存活+自回合交 builder |
| `resolve_duanbi` | **已折** | chooser=first；`Participants::distinct_pair` 守 owner/first 双存活异座；payload 续带 valid[1..] |
| `resolve_jinqu` | 不折 | stage 0 夹 `run_effects(DrawCards)` 摸牌副作用在 guard 与 choice 之间 |
| `resolve_shenshi` | 不折 | 转换技 `state_blob` 翻转副作用 + stage `0|1` 合并臂 + stage 4 独立追踪逻辑 |
| `resolve_zhenliang` | 不折 | 转换技 `state_blob` 翻转 + 阴/阳双路径各自复杂候选集过滤 |
| `resolve_zuilun` | 不折 | 摸 3 张判定 + reshuffle + 多 `move` 副作用 + count==0/3 分支 |
| `resolve_juzhan` | 不折 | 转换技翻转 + 阴/阳双分支（阴有摸牌+ban 副作用，阳有目标解析） |
| `resolve_ddd_xinxianying_choice` | 不折 | 多段续段处理器（stage 0/1/10/11），段间有 move/gain 副作用 |

**8 个候选仅 2 个可干净折叠**。含转换技翻转、段间副作用、多分支真实业务逻辑的占 6/8。
任务的「初筛候选约一半含真实逻辑」教训成立且偏保守。

### hongju 折叠要点

`resolve_hongju`（pack_shenhua_1.rs）原 stage 0：

```rust
if !originating_skill_is_enabled(tx, player, skill)
    || !tx.state().players.get(player).is_some_and(|p| p.alive)
    || tx.state().turn.current != player
{ return Ok(Vec::new()); }
// 业务 guard：荣标记≥3、未觉醒
open_choice(tx, player, 1, 1, true, vec![1,0], ChoiceKind::Custom, ...)
```

折叠后：业务 guard 外提，`originating_skill_is_enabled + alive + OwnersTurn` 交 builder：

```rust
// 业务 guard 外提
GuardedChoice::new(skill, Participants::owner(player), ContinuationKind::Custom, payload)
    .during(TurnContext::OwnersTurn)
    .shape(ChoiceShape::one_optional())
    .open_tokens(tx, vec![1, 0], ChoiceKind::Custom)
```

### duanbi 折叠要点

`resolve_duanbi`（pack_shiji.rs）chooser 是 `first`（被弃牌目标，非 owner）。原代码
外层 guard 只查 owner；first 的存活在 `valid` 过滤里查。折叠后用
`Participants::distinct_pair(player, first)` 同时守 owner/first 双存活异座，
`.chosen_by(first)` 把选择权交给 first。payload 尾部续带 `valid[1..]` 的原始语义
（供续段按座次推进下一名目标）保持不变。

外层 `originating_skill_is_enabled + alive` guard 移除——该函数只有 stage 0 一个入口
（续段经 `resolve_choice` 调度），stage≠0 是非法路径，`_ => Err(...)` 比
静默返回空更安全地暴露非法调用。

## 多段 continuation 不折决定

### 审计数据

扫描 `primitives/pack_*.rs` 全部 `resolve_*`：

- **113 个**实质多段状态机（≥2 个实质 stage 臂，排除 `_ => Err(stage_must_resume...)`
  这类纯报错兜底臂）。
- 操作序列指纹（`EffectRequest::*` / `open_choice` / `ChoiceKind` / `ContinuationKind` /
  `run_effects` 等的有序序列）完全相同的组：**仅 5 组共 18 个函数**
  （任务文档提到的「13 组 87 个」是上一轮更松口径，含单段与更多 token 种类）。

### 抽样核实

逐个读码核实空指纹组（5 个）与最大重复组（5 个）共 10 个函数，**全部含真实业务逻辑，
0 个是纯样板**：

- `resolve_mantianguohai`：3 段各调不同专用命令函数（真实分发）；
- `resolve_potlibing`：读写技能 `state_blob` + 嵌套 `match step` + `saturating_add` 累加；
- `resolve_drlt_yongsi`：stage 分支含 `if damage_dealt_count==0` 真实条件；
- `resolve_spqiai`：stage 0 给牌 + 受伤判定分支，stage 1 据选择回血/摸牌；
- 其余含转换技翻转、段间 `run_effects`/`move_cards` 副作用、多分支候选集过滤。

### 不折理由

1. **每个 `[MAGIC, stage, ...]` payload 字段顺序由各技能自定义，无统一布局**。强行定
   通用 stage/payload 约定 = 把异质业务逻辑塞进通用壳，增加间接层、降低可读性。
2. **stage 语义随技能变化**：有的 stage 0 是「设 fixed_count」，有的是「给牌+开 choice」，
   有的是「翻转转换技」。没有可参数化的固定形状。
3. **风险与收益倒挂**：113 个多段函数全在 `original_*` golden 回归覆盖下；动 ABI（已 640，
   且 omp 占 641+ ddd 段）+ golden 重钉的成本巨大，而 5 组 18 个候选里 0 个是纯样板，
   收益接近零。

架构计划 §47.3「禁止技能级硬编码旁路」的目标是阻止**新增旁路**，而非要求把已存在的
异质多段实现强行同构化。多段 continuation 的多样性是技能语义本身的多样性，不是旁路债务。

## 验证

| 检查 | 基线 | 折叠后 |
|---|---|---|
| `cargo check -p engine-core --lib` warnings | 59 | **59**（未新增）|
| `cargo test -p conformance` | 3 向量全过 | **3 向量全过** |
| `original_shenhua_guanqiujian`（hongju） | 5 passed | **5 passed** |
| `original_sixiang_stdduanbi` 行为测试 | 2 passed | **2 passed**（语义不变）|
| `original_sixiang_stdduanbi` 包元数据断言 | FAILED（既有漂移，left:562 right:468，他人加内容）| 同（**非本次引入**）|

**不动 ABI、不动 golden、不动 DSL**。折叠是普通函数调用点的替换，与 0595 同质。

## 后续

- GuardedChoice 的覆盖面经此增补后，剩余「单实质 stage + 裸 open_choice」候选已全部
  评估完毕（8 个里 2 个折叠、6 个因含真实逻辑不折）。该方向已收敛，不再有低成本标的。
- 多段 continuation 参数化正式否决，本 ADR 已固化底稿结论（底稿于 2026-09-16 清理）。
- 若未来出现新的「单段 guard + 开一个选择」型技能，优先复用 GuardedChoice（0595 + 本 ADR）。
