# ADR 0783：Boss 关卡流程子系统（D4）

日期：2026-10-16　|　状态：已采纳（用户决定 D4；实现分批推进）
关联：[进度主稿 T11](../architecture/native-progress.md)、[规则基准](../architecture/native-rules-baseline.md)、[D4–D8 决定](../operations/evidence/2026-10-16-d4-d8-decisions/README.md)、[待做登记](../operations/evidence/2026-10-16-todo-register/README.md)、[隐藏链处置](../operations/evidence/2026-10-11-t11-boss-hidechain-disposition/README.md)、[bianshen 勘验](../operations/evidence/2026-10-11-t11-boss-bianshen-survey/README.md)、[秦广王 v276](../operations/evidence/2026-10-14-t11-boss-qinguangwang-v276/README.md)、[ADR 0781](0781-mutable-seat-ring.md)、[ADR 0753](0753-native-n11-guozhan-kof-mode-design.md)

## 背景与 D4 裁决

截至 2026-10-14，Boss 模式主线对抗链条目与大量内容技已入库；十殿阎罗 12/13 内容面落包（四季映姬排除）后，仍有一族机制被统一记为「关卡流程面登记不落」：

| 族 | 代表技能 / 状态 | 既往处置 |
|---|---|---|
| 十殿阎罗「还人」 | `boss_newhuanren` / `boss_newhuanren2`；`_status.shidianyanluo_level`、孟婆乱入旗、禁将表 | 秦广王条目不含还人（v276） |
| 四象隐藏链八技 | `boss_qingmu2/3`、`boss_baimang2/3`、`boss_chiyan2/3`、`boss_xuanlin2/3` | 2026-10-11 整族处置不落 |
| 青/赤/白翼轮次编排 | `boss_qingyi`、`boss_chiyi`、`boss_baiyi` | 青翼先例延伸不落 |
| bianshen 死亡换 Boss | `boss_bianshen4`（及同构 `bianshen3→4`） | 隐藏链同族不落 |

用户 2026-10-16 裁决 **D4**：**建原生 Boss 关卡流程子系统，并把上述技能逐个落包**（见[决定证据](../operations/evidence/2026-10-16-d4-d8-decisions/README.md)）。本 ADR 定子系统边界、数据模型草图、版本/golden 口径、与 D5/映种/庞统的边界，以及分批落地与验收顺序。本文件本身不改变代码语义。

## 决策

### 1. 子系统定位

在 `engine-native` 的 **Boss 模式层**新增可序列化的 **关卡流程面（Boss Stage Flow）**：统一承载「换将 / 禁将 / 乱入 / 隐藏链换阶段 / 轮次门控编排 / 死亡换 Boss」等跨技能共享的状态与确定性转移，而不是把每个流程技各自硬编码成互不共享的独立机器。

已有可复用先例（须对齐其存档/assembly 口径，不另造第二套）：

- Boss 座位扩张与 fellows（v84 `addFellow`）
- versus 替补换将 / 年兽 `nbianshen` 形态轮换（assembly 重挂 + 属性保留）
- 座位环原语（[ADR 0781](0781-mutable-seat-ring.md)）——隐藏链 `changeSeat` 若触碰环序，必须走环访问器

### 2. 边界：内核流程面 vs HostBehavior

| 进子系统（内核流程面） | 仍归 HostBehavior / 内容技 |
|---|---|
| 关卡等级、阶段指针、乱入一次性旗、禁将集合 | 伤害/摸牌/弃牌/判定改写/距离与可用牌 mod |
| `changeBoss` / 座位 hide 语义的确定性结果（装配重挂、Boss 指针更新、fellow 入队） | 触发后「内容效果」（如赤炎补给回满、授标记技造成的增伤）——由内容技在流程转移完成后执行，或由流程转移发出的效果表驱动 |
| 轮次门控表（round∈{3,5,7} 等）与「本轮已编排」标记 | 门控通过后发放的具体标记技 / 火伤数值等 |
| 死亡换 Boss 的候选池与选中规则（种子化随机） | 被换上将自带的非流程技 |
| 流程相关询问（十殿选下一殿 Boss）的挂起/快照变体 | 与流程无关的普通濒死/无懈等询问 |

**原则**：技能表仍可挂名（便于目录与触发登记），但流程技的「状态转移」必须调用子系统 API；禁止在 HostBehavior 里散落第二套 `_status.shidianyanluo_*` 等价物。内容技不得直接改写关卡等级或禁将表。

**试炼态 init 壳**（`boss_qingmu` 等开局 `player.init` 视觉壳）维持既有结论：**不入子系统、不落包**——条目以目标将代位。年兽 **形态轮换**（`boss_nbianshen`）已有承载先例，属「装配轮换」而非本 ADR 的十殿/隐藏链/翼编排核心；若后续要统一 API，可复用同一 `reinit_boss` 原语，但不阻塞 D4 首批。

### 3. 数据模型与状态机草图

#### 3.1 建议状态（草图名，实现可改名）

挂在 `GameMode::Boss`（或并列的 `BossStageState`）上，**全部进快照 / canonical hash**：

```text
BossStageState {
  kind: StageKind,           // None | ShidianYanluo | SixiangChain | WingGate | …
  level: u8,                 // 十殿：0..3 对应关卡说明四关；其他 kind 自解释
  mengpo_spawned: bool,      // _status.shidianyanluo_mengpo
  mengpo_died: bool,         // _status.shidianyanluo_mengpodie
  banned_generals: Vec<SymbolId>,  // 还人禁将表（上游字面名单）
  wing_armed: BitFlags,      // 青/赤/白翼本局已触发的 round 门
  pending: Option<StagePending>,   // 选将询问 / 延迟换将队列
}
```

上游对照（只读）：`archive/upstream/noname/apps/core/mode/boss.js` 中 `boss_newhuanren*`、`changeBoss`、`addFellow`，以及隐藏链/翼/bianshen4 各技定义——勘点见[本 ADR 证据目录](../operations/evidence/2026-10-16-d4-boss-adr/README.md)。

#### 3.2 转移草图

```text
[十殿·还人]
  Boss 死亡 / 关卡事件
    → (若 level 未终) 创建选将询问（候选池按 level）
    → changeBoss(选中)
    → level++
    → 概率/条件孟婆 addFellow（mengpo_spawned 置位）
    → 应用 banned_generals（选将/变将入口）
  特殊：level==2 且 phase≤50 且 mengpo_died → 地藏王分支；
        「阴间将」占比门槛 → 四季映姬候选（见边界：映种仍排除）

[四象·隐藏链]
  dieBegin(filter: 死者==boss 且持有 *2/*3)
    → hide（原生：座位不可交互 / 表现事件；不等同 UI 观战）
  dieAfter
    → changeSeat 调度（经 seat_ring）
    → changeBoss(下一阶段将) [±侧座换将 / addBossFellow / 回满 / 复活挑战者]
    → 链终止于阶段 3 目标将

[翼·轮次编排]
  round 锚点（phaseZhunbei / roundStart 等按上游）
    → 若 round 命中表且未 armed → 执行编排效果表 → armed

[bianshen 死亡换 Boss]
  同隐藏链 dieBegin/dieAfter 骨架
    → changeBoss(候选池随机，种子化)
```

随机一律走引擎 RNG；禁止 `Math.random` 语义直译而不记账。

### 4. 与 STANDARD_RULESET_VERSION / ARCHIVE_FORMAT_VERSION / golden

| 变更类型 | 规则版本 | 存档版本 | 14 golden |
|---|---|---|---|
| 仅 ADR/证据/基准「登记不落→待实现」措辞 | 不递增 | 不递增 | 不动 |
| 引入 `BossStageState` 字段并编入快照/哈希 | 通常 **同步 +1**（探针证明位移源） | **必须 +1** | 查明原因为版本/字段后重录 |
| 首个流程技改变对局可达语义（选将链/乱入/禁将生效） | **必须 +1** | 若无新字段则可不变；有字段则跟字段批 | 回退探针全绿后重录或证明无位移 |
| 仅补 HostBehavior、不改编码与可达规则 | 视是否改语义；无语义则不增 | 不变 | 不动 |

口径与 AGENTS.md / 进度主稿一致：**先查位移原因与语义，再决定重录**；不得只改期望值消失败。子系统落地首批若只加空壳状态且默认 `kind=None`、对现有回归零可达，允许「存档 +1 + 规则 +1 + 探针后重录」一次付清编码债，再在后续批只升规则版本。

### 5. 与 D5 / 映种 / 庞统的边界

| 项 | 归属 | 本 ADR |
|---|---|---|
| **D5 龙血玄黄 / `ylyg*`** | T9 轨迹采集 → 规则面迁移；Boss 诸葛亮四技依赖 | **不在 D4 实现范围**。关卡子系统不得为加载 `ylyg*` 开特例。诸葛亮整将与 hp Infinity 承载随 D5 解锁后另批（待做 `T11-BOSS-ZHUGE-LIANG`） |
| **hp Infinity** | 数值模型扩展 | D4 不引入；若十殿/隐藏链不依赖 Infinity，首批禁止顺手改 hp 表达 |
| **四季映姬「映种」`boss_yingzhong`** | 外部武将池结构性依赖（`get.character(name,3)`） | **维持目录排除**。还人终关「映姬候选」若实现，只能在池可用时入候选；池基建未就绪则登记差异「候选恒不可达」，不借 D4 偷建外部将库 |
| **庞统涅槃/浴火** | 外部包技（shenhua `oldniepan` / offline `ylygniepan` 等） | **不在 D4**。庞统已落天狱/栖梧；外部技随 D5/外部包推进（`T11-BOSS-PANGTONG-EXTERNAL`） |
| **孟婆「往事」打出面等零散差异** | 内容技余项 | 可不阻塞子系统；`T11-BOSS-MENGPO-WANGSHI-PLAY` 独立收口 |
| **地藏王「大愿」AI 态度面** | 内容技 | 不纳入 D4 关卡子系统（模糊项已登记） |
| **Boss `loopType==2` 等模式变体** | 模式入口余项 | 可依赖子系统原语，但本 ADR 不承诺一次建模全部 mode 变体 |

### 6. 分批落地顺序与验收口径

依赖：子系统骨架（状态 + `change_boss` / `add_boss_fellow` / 禁将查询 API + 存档）→ 各技落包。

| 批 | 内容 | 建议证据目录名 | 验收 |
|---|---|---|---|
| **B0** | 子系统骨架 + 默认空状态；存档/哈希/往返；无内容语义或仅内部 API 单测 | 开工清单：[batch-0.md](../operations/evidence/2026-10-16-d4-boss-adr/batch-0.md) / [batch-0.json](../operations/evidence/2026-10-16-d4-boss-adr/batch-0.json)；实现证据建议 `docs/operations/evidence/YYYY-MM-DD-d4-boss-stage-b0-skeleton/` | 定向往返；规则/存档按 §4；全量 `cargo test --offline -p engine-native --no-fail-fast`；Gate 1b；规则基准改「待实现→本批已承载」 |
| **B1** | 隐藏链八技（结构最同构，无十殿选将 UI） | `…-d4-boss-hidechain-…` | 每链至少：死亡→换将/fellow、守恒、暂停恢复（若有询问则无）；四象条目补挂流程技名 |
| **B2** | bianshen4 死亡换 Boss（复用 B1 骨架） | `…-d4-boss-bianshen-changeboss-…` | 候选池种子化；阴间系条目补挂 |
| **B3** | 还人：等级链 + 选将询问 + 禁将表 + 孟婆乱入旗 | `…-d4-boss-huanren-…` | 多关推进；禁将入口拒绝；乱入一次性；与忘魂/孟婆已落技共存 |
| **B4** | 青/赤/白翼轮次编排 | `…-d4-boss-wing-rotation-…` | round 门控表；武装旗往返；太昊/炎帝/句芒等条目补挂 |

每批最低证据：定向日志、版本回退探针（若升规则版本）、golden 处置说明、全量计数与 Gate 1b、规则基准补注。**禁止**本 ADR 批次改 `cards.json` / npack 作为流程承载（流程在内核模式态，不在牌数据）。

待做 ID 映射：`T11-BOSS-STAGE-SUBSYSTEM`（B0）→ `T11-BOSS-HIDECHAIN-8`（B1）→ `T11-BOSS-BIANSHEN-CHANGEBOSS`（B2）→ `T11-BOSS-HUANREN`（B3）→ `T11-BOSS-WING-ROTATION`（B4）。

**建议首批实现技能列表（B1，在 B0 之后）**：

1. `boss_qingmu2` / `boss_qingmu2x`（青龙 1→2，链最短对照）
2. `boss_qingmu3` / `boss_qingmu3x`（含三人 hide + 侧座换将）
3. 其余六技按同构表参数化落包（白芒/赤炎/玄麟 2/3），避免八份拷贝逻辑

### 7. 非目标

- 不在本 ADR 实现龙血玄黄、`ylyg*`、诸葛亮、庞统外部涅槃/浴火、映种外部将池
- 不承诺旧 JS `mode/boss.js` 零改动运行（[ADR 0782](0782-extension-backend-and-legacy-js-scope.md)）
- 不把 KOF 多局会话、国战双将模型并入本子系统（见 ADR 0753）
- 不把 UI 观战位移、smoothAvatar、changeSkin 视觉面纳入规则等价；原生以座位交互门控 + 表现事件近似，差异须登记
- 不在本 ADR 批次修改 `crates/**` 之外的牌包数据；实现 worker 另开批次
- 不因本设计文档单独宣称 T11/U02/L03 整包验收完成

## 后果

- 规则基准与进度主稿中「关卡流程面登记不落」条目转为 **待实现（依赖本 ADR）**；正文回填由进度协调者合并（建议文案见证据 README）
- 实现须按 B0→B4 顺序；跳批落内容技若未调用子系统 API，视为违反本 ADR
- U02/L03 boss 来源整包仍受扩展台账 G2–G3 约束；本子系统是内容面前置，不是整包验收本身

## 验收（本 ADR 文档批）

- 本文件落盘且 [INDEX](INDEX.md) 已登记
- 证据目录链到本 ADR 与上游勘点
- B0 引擎开工清单 [batch-0.md](../operations/evidence/2026-10-16-d4-boss-adr/batch-0.md) / [batch-0.json](../operations/evidence/2026-10-16-d4-boss-adr/batch-0.json) 已落盘
- **无** `crates/**`、cards.json、npack 改动；不跑长时间全量 cargo（实现批另证）
