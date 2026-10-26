# 原生收敛剩余工作台帐（2026-10-24 建立，2026-10-26 扫描更新）

本文件是 **NC01–NC12 / UP37-R01–R10 / UP37-N01–N07 / UP37-I01–I05 / PA01–PA12 / 37 来源** 的统一剩余清单，供逐项领取与验收。状态只按当前源码和可重跑证据填写；`ported`、包数量、历史绿灯不等于完成。当前基线 HEAD `afdc39ab5f5df38b25f81e455a6fb9109774c039`（2026-10-26 17:10 扫描；工作树另有约 312 项未提交改动，见下文「当前阻断」），规则 **v322**、存档 **v105**。扫描当时实跑：engine-native 全量 **1559/0/71**、前端 `pnpm test` **80/0/0** 与 typecheck 通过；**engine-core 与 game-server 编译失败**（见阻断 B1）。**2026-10-26 19:40 更新**：B1 编译失败已修（干净 HEAD + 本批补丁 `cargo check --offline --workspace` 通过，PA06 遗留 216 处调用点全部同步），首批时三个全量套件因磁盘耗尽未跑完；**20:15 补跑完成**（用户裁决清理磁盘后，干净 HEAD `f49e9f89` 导出）：engine-core **3779/0/1**、game-server **115/0/0**、engine-native **1546/7/71**（7 失败全部归因为 B5 未提交包数据），B1 → **定向通过**；另新增阻断 B4（shahai 未跟踪内容文件）、B5（mark-supplement 包数据未提交）。

状态词：**已实现**（代码/内容存在）、**定向通过**（有回归）、**产品通过**（正式入口/恢复/联机）、**上游等价**（逐条语义核对）。未达后两者不得写“完成”。

## NC01–NC12

| 编号 | 当前状态 | 剩余与完成条件 | 依赖 | 证据 |
|---|---|---|---|---|
| NC01 座位与触发语义 | 部分 | 多座位选将/随机/私有选择/触发顺序全矩阵；完成需逐模式开局、触发顺序、RNG、恢复轨迹 | NC08 | 进度主稿 |
| NC02 安装包身份与执行内容一致 | **完成（2026-10-25）** | 完整 VM 包正式安装/选将/开局/停用启用/升级回滚/停用式卸载/物理卸载/旧档错版拒绝/冻结副本原锁恢复；第三方免客户端重编译；真实服务端+浏览器同链 E2E | NC03/NC04 | [Worker VM](../operations/evidence/2026-10-24-worker-vm-package/README.md)、[VM E2E](../operations/evidence/2026-10-24-worker-vm-package/ui-e2e-vm/README.md) |
| NC03 统一原生产品 API | 部分 | Worker 真实事件/hash 已做；坏档、慢连接、重连已做；仍需产品 UI 全流程与错误码覆盖 | NC01 | [P1](../operations/evidence/2026-10-24-p1-trust-boundaries-sync/README.md) |
| NC04 VM 故障与事务语义 | 部分 | 未映射事件（如 `cards_given`）阻断完整 shiji；VM 故障/回滚/恢复矩阵未全 | NC02 | [Worker VM](../operations/evidence/2026-10-24-worker-vm-package/README.md) |
| NC05 可独立发布的扩展宿主 | 未完成 | 三类真实案例全生命周期；混合包、依赖、卸载、旧锁恢复 | NC02/NC04 | 进度主稿 |
| NC06 核心与内容私有状态分离 | 部分 | 已修非空二元键 Snapshot 编码，直接回归两张地图 + 技能 storage 往返；私有字段编码/解码/校验/hash 收敛仍需更多真实内容试点 | NC07 | [保存成本](../operations/evidence/2026-10-24-native-save-cost/README.md) |
| NC07 状态定义与持久化同源 | 部分 | 手工字段登记仍扩张；版本/归档/迁移策略未统一 | NC06 | 进度主稿 |
| NC08 规则对照与原生验收门禁 | 部分 | 1 人标准 40 步、huanle 20 步已对照；多座位/选将/随机/私有选择/37 来源未覆盖 | NC01 | [跨目标](../operations/evidence/2026-10-24-native-wasm-cross-target/README.md) |
| NC09 原生客户端与联机闭环 | 部分 | 慢 WS/重连/单写者、跨刷新持久化、UI 取消、旧档不确定提示已做；桌面/移动目标平台、完整产品 UI 全流程未验收；客户端本地内置包安装入口（`apps/web-client/src/engine/bundledNativePackages.ts`，未提交在途文件）仍拒绝含 VM 模块的包，服务端 `/native/packages` 与浏览器面板路径已由 NC02 打通 | NC03 | [P1](../operations/evidence/2026-10-24-p1-trust-boundaries-sync/README.md) |
| NC10 AI 信息边界与有界排程 | 部分 | 合法座位、有界排程、长驻池复用/降级/取消/过期、p50/p95/p99 尾延迟基线、主动技候选与技能威胁度加权均已有测试；非静止手牌身份自省与质量基线未做 | NC09 | [AI 边界](../operations/evidence/2026-10-24-nc10-ai-boundary/README.md)、[R10](../operations/evidence/2026-10-24-up37-r10-metadata/README.md) |
| NC11 验收数据与文档收敛 | 部分 | 本文与 progress/baseline/ledger 已互链；失效引用与历史证据仍需逐批清理 | 全部 | 本文 |
| NC12 效能验收与旧线退出 | 部分 | 保存成本基线已有；收据已限；多房间 save 已优化至中位 15.5ms/p95 35.9ms；WAL/批提交未做；旧线退出须替代能力+数据兼容验收；内核性能结构项 PA01–PA03、PA05、PA06 已定向通过、表现层 PA08 基本完成（见 PA 段） | NC06/NC08 | 保存成本证据 |

## UP37-R01–R10

| 编号 | 状态 | 剩余 | 证据 |
|---|---|---|---|
| R01 涤仇 | 旧线已修、原生未注册 | 原生同名、恢复与全量清标记完整对局 | [R01](../operations/evidence/2026-10-24-up37-r01-dichou/README.md) |
| R02 讲武 | 未实现 | 权威扣款/授予/重试/恢复；商店入口 | 进度主稿 |
| R03 冯河/英霸 | 旧线部分 | 交牌/重铸、授标分支、手牌上限、maxHp>1、平定目标限制已实现；仍缺原生同名；标记前用牌计数已补；防伤时序误读已更正 | [R03](../operations/evidence/2026-10-24-up37-r03-recast/README.md) |
| R04 弼士 | 旧线 noop | 额外阶段/受益者/来源绑定/首张不可响应 | [R04 旧证据](../operations/evidence/2026-10-24-upstream-37-impact/README.md) |
| R05 涕泣/宝梳/避祸 | 三份 noop | 摸牌事件、翻面/横置、跨回合待生效 | 同台账 |
| R06 纵害/明节/击戈 | noop/近似 | dying 祖先、一次性防伤、手牌外杀 | 同台账 |
| R07 举战/自固/荡尘 | noop/部分 | 取消付费、禁弃/禁获得、实体牌归属 | 同台账 |
| R08 清势/神赋/顾曲/争功 | noop/近似 | 取消/空 cards/无 currentPhase、装备实际使用 | 同台账 |
| R09 讨州/相谶/摧锋 | noop/占位 | 无实体牌、目标有效性、技能授予恢复 | 同台账 |
| R10 AI/显示/元数据 | 元数据 5 测通过；AI 有限值已修；原生 AI 变体≥2 已纳入主动技候选、技能威胁度加权（含测试） | 应机/共护/自矜/勇剑/化伤估值、RNG、Planner、N01 旧线 AI | [R10](../operations/evidence/2026-10-24-up37-r10-metadata/README.md) |

## UP37-N01–N07

| 编号 | 状态 | 剩余 |
|---|---|---|
| N01 界朱然/界朱桓 | **旧线规则完成** | `ol_zhuhuan`+`ol_fenli` 三窗口、`ol_pingkou` 伤害/按实际受伤摸牌；**新增 `ol_jie_zhuran`/`ol_jie_danshou` 四选项**、序号弃牌、每阶段一次、取消/非法/phase 清零/ActivateSkill 共 24 测。onlyOL 0.1.4/ABI 757；旧 0.1.2 npack 存档。未验收：AI 估值、原生（757>753）、旧 0.1.2 局恢复 |
| N02 唐棠 | 未实现 | `tt_zexing`/`tt_zhiyi` 摸牌事件、颜色、目标恢复 |
| N03 TW魔张梁 | 未实现 | 技能/标记/随机/配置 |
| N04 OL孙寒华 | 未实现 | 装备形态、花色、回合历史 |
| N05 OL赤兔 | 未实现 | 手牌顺序权威同步/恢复 |
| N06 OL乐孟获 | 未实现 | 随机效果、保存恢复 |
| N07 OL许劭 | 未实现 | 最终重写版动态技能、保存恢复 |

## UP37-I01–I05

| 编号 | 状态 | 剩余 |
|---|---|---|
| I01 内容/目录同步 | 已同步 + `--verify-only` 通过 | 分包差异审计、旧局内容锁反例 |
| I02 资源/画像 | 部分 | 4 张新画像已入；语音/皮肤/别名缺 |
| I03 配置/依赖 | 部分 | 新内容依赖/配置/交互未全 |
| I04 兼容/旧锁 | 未完成 | 旧 0.1.x/0.2.x 存档拒绝与恢复矩阵 |
| I05 验收数据 | 部分 | 每批证据目录已有；37 来源 G0–G5 未逐项 |

## PA01–PA12 性能架构调整（2026-10-26）

来源：[内核性能结构复核](../operations/evidence/2026-10-25-native-arch-perf-review/README.md)、[表现层帧间隔与输入延迟基线](../operations/evidence/2026-10-26-web-frame-latency/README.md)。单写规则核心、不可变目录、显式续行和内置 HostBehavior + 第三方 rule-vm 路线**保留不变**；下表只登记性能面需要调整的项。每项都要用对应证据里的测量程序在同机同负载下取前后数据；规则、存档或状态摘要字节变化须走版本门禁，并先解释 golden 变化原因再重录。标「需裁决」的项先由用户决定，不得自行改变范围。

建议顺序：PA01、PA06 可独立先做；PA02→PA03 依次改同一提交路径；PA05 在 PA01–PA03 之后；表现层先 PA08 扩面，再按数据决定 PA07、PA09–PA11；PA04 随 NC06/NC07；PA12 待裁决。

| 编号 | 状态 | 问题（实测或静态核对） | 调整与完成条件 | 依赖 | 证据 |
|---|---|---|---|---|---|
| PA01 活跃触发索引 | **定向通过（2026-10-26，ZCode A道）** | 目录追加 6000 个**未被持有**的触发技能后，8 座提交 p50 13.3→44.4 µs、p95 24→215 µs，规划 p95 0.21→1.15 ms；实测共两处根因：`resolution.rs` `plan_triggers` 按候选定义遍历实例（复核已登记），以及 `skill_by_name` 全表线性扫描被 `computed_distance` 热路径反复调用（本批采样新发现）。已改：`plan_triggers` 从实例侧单遍直接比对 `TriggerMeta.event/extra_events`（与 dispatch 成员严格等价，守护测试锁面），`Catalog` 构建期 `by_name` 哈希索引使 `skill_by_name` O(1)（首个匹配语义不变）；未新增每局可变状态，不进哈希与存档。完成条件三项全达成：golden/状态摘要不变（engine-native 全量 **1555/0/71** 两次、各档轨迹逐字一致）；archbench `scale` 6215 档 vs 215 档提交 p50/p95 相差 **≤3.2%**（门限 10%）、规划 p50 ≤2.6%。Gate 1b 被他人未提交 rule-vm 改动的既有 unreachable_patterns 阻塞，本批两文件零警告（如实记录） | — | [PA01 证据](../operations/evidence/2026-10-26-pa01-trigger-index/README.md)；内核复核 表 A |
| PA02 AI 模拟与权威会话分离 | **定向通过（2026-10-26，ZCode A道；growth 末/首比残余挂 PA05）** | `Engine` 整体 `Clone`（含整局日志）；AI 每个候选节点克隆一次，8 座每节点 p50 从 18.8 µs 涨到 46.9 µs、规划分配 309→1068 次；模拟提交还做检查点与全量哈希。**已改**：`Engine.simulation` 会话标志（不入状态/哈希/快照）+ `clone_for_simulation()`（手工构造不走 derive clone，日志从空开始、检查点弃置）；模拟提交跳过 save 检查点/日志写入/哈希（Accepted 全零哈希），结算代码与正式提交共用；`probe_action` 探测克隆同享（消除 legal_actions 回退路径每候选的整克隆+哈希）。**完成条件**：①固定种子 4/8 座 722/1718 步 (seed,step,action,nodes) 改动前后逐行一致 ✔；②nc10/nc03/ai_planner 定向 **37/0**、engine-native 全量 **1558/0/71**、wasm 编译 exit 0 ✔；③growth 末/首比 8 座 2.60→**1.27**、4 座 2.42→**1.55**，每节点绝对值首档 −65%/末档 −83%、每步分配 −66%/−76%——**未达 ≤1.2 的残余经双判别实验证明与权威日志无关（restore 空日志 −2%）、与进程堆无关（新进程同值），属晚期状态固有结算复杂度，非会话耦合**，登记为 PA05 归因范畴残余 | PA03 | [PA02 证据](../operations/evidence/2026-10-26-pa02-ai-sim-session-split/README.md) |
| PA03 状态哈希按需计算 | **定向通过（2026-10-26，ZCode A道；按「内部缓冲」选项收口）** | `canonical_hash` 每条被接受命令执行一次，4 座 5.5 µs、8 座 8.6–9.1 µs，约占提交 p50 的 60%–70%；逐字段约 800 次小块写入，比缓冲后一次哈希慢 1.4–1.7 倍。**已改**：`engine/hash.rs` 全部 `push_*` 助手改写 `HashSink`（`Vec` 缓冲），`canonical_hash` 末尾一次 blake3 update——**摘要字节恒等**（golden 未改期望即全过），哈希 p50 −53%～−54%（8 座 8,733→4,150）、提交 p50 −44%～−45% 且随日志档走平（档间比 ≤1.02）。**边界缓存方案已实测后撤销**：状态原地修改面无法收敛到统一失效点（18 测试暴露缓存陈旧，详证）；engine-native 全量 **1558/0/71**、wasm 编译 exit 0、行数门禁通过 | — | [PA03 证据](../operations/evidence/2026-10-26-pa03-hash-buffered/README.md)；内核复核 表 C |
| PA04 内容私有状态迁出核心 | 未开始（并入 NC06/NC07） | `TurnState` 61 个、`PlayerState` 105 个字段，多数是具体技能专用；每局都为它们付复制、哈希、归档成本 | 按 NC06/NC07 逐职责迁入技能实例存储。性能面完成条件：标准局 `Engine::clone`、`state_hash` 与归档字节随迁移下降，有前后数据 | NC06/NC07 | 内核复核 表 D |
| PA05 分配热点 | **定向通过（2026-10-26 收尾，ZCode A道）** | 每步（规划+提交）平均分配 539 次（4 座）/ 1136 次（8 座）；长程 200 万命令约 730 次/命令。PA02 已消其中约 2/3（会话侧）；**另接收 PA02 残余**：growth 末/首每节点比 1.55（4 座）/1.27（8 座），已证明与日志/进程堆无关，属晚期状态固有结算复杂度——归因时一并定位 | PA01–PA03 完成后先做分配归因（计数分配器 + 调用栈），再改造热点（如 `BTreeMap<(u8, String), _>` 字符串键、每次触发的 `Box<dyn SkillMachine>`）。**已做**：归因（allocbench/clonebench）定位最大热点=Engine::clone 的日志复制（LogEvent 内嵌 Vec，198/386 次随日志增长）→ ai_pool 池克隆改 clone_for_simulation（每轮 792→36 / 3088→96，−95%/−97%）；legal_actions 去重键去字符串化（GameAction: Ord）。收尾批：①惰性段检查点（time_slice=None 时段不可能暂停，save() 快照跳过）；②`play_card` 纯校验前缀抽取为 `common_play_validation`（正式路径同代码调用，防漂移）+ `legal_actions` 保守快拒（只拒「同代码判定必拒」候选，接受集单调不变）——固定种子 4/8 座 722/1718 行动作序列与原始行为逐行一致。**growth 同负载每步 184→175（4 座）/ 275→262（8 座）实测下降**；全弧线 539/1136→175/262（−68%/−77%）。engine-native 全量 **1558/0/71**、wasm 编译 exit 0；game-server 套件当时未能编译——2026-10-26 扫描在干净 HEAD 复现并定位为 PA06 的 `RuleVm::execute` 签名变更未同步旧线 `engine-core/src/vm_host.rs`（见阻断 B1），原记「他人 engine-core 在途改动」归因有误。**登记为后续**：状态 Arena 化/探针免克隆（每步进一步下限）、实目录字符串键测量、池端到端收益。完成条件：同负载每步分配次数下降，行为与摘要不变 | PA01–PA03 | 内核复核 表 B；[长程内存](../operations/evidence/2026-10-18-nc12-long-memory/README.md)、[PA02 证据 §3](../operations/evidence/2026-10-26-pa02-ai-sim-session-split/README.md) |
| PA06 rule-vm 重复校验 | **engine-native 侧定向通过（2026-10-26，ZCode A道）；旧线调用方已同步（B1，2026-10-26 ZCode A道：registry 装载期持有 `VerifiedModule`，工作区 216 处调用点全部改收已校验模块，干净 HEAD `cargo check --workspace` 通过）；旧线全量 engine-core **3779/0/1**、game-server **115/0/0**（B1 补跑，2026-10-26 20:15）** | `RuleVm::execute`/`resume` 每次调用都执行 `verify_module`（静态核对）。**已改**：新增 `rule_vm::VerifiedModule`（只能经 `verify` 构造），`execute`/`resume` 只收已校验模块——执行期零重复；engine-native 装载点（`VmSkillProgram::new` 建目录、快照恢复）装载期校验一次，畸形模块装载期拒绝（新 `ExtensionError::InvalidVmModule`）。微基准：**每次 VM 触发解释器工作量 −49%**（verify 41ns+execute 42ns→42ns），装载期每模块一次性 +41ns。**语义变化已登记**：坏字节码拒绝时机从首触发前移到装载期（t8 拒收②改写）。rule-vm **15/0**、engine-native **1558/0/71**、loader **32/0**、wasm 编译 exit 0 | — | [PA06 证据](../operations/evidence/2026-10-26-pa06-vm-load-verify/README.md) |
| PA07 点击后回包推迟一帧 | 复现完成＋缓解原型落地（2026-10-26）：SAB 镜像+rAF 提前解析机制 145/145 打通、transfer -13%，但端到端 i2p 未改善（16.3ms）——收益被 patch→frame 段吸收，待帧标记语义/trace 甄别后定缓解上限；WKWebView 已达标无需缓解 | Worker 每步 p50 0.4–0.6 ms，但真实点击后回包约 16 ms 才送达主线程；同局无输入推进只要约 1 ms；点击到绘制 p50 约 33 ms（60 Hz 两帧）；GPU 开关不影响 | 先在有头 Chrome、Tauri WebView（macOS WKWebView、Windows WebView2）、Safari、Firefox 复现；复现后比较缓解方案（如共享内存 + Atomics 通知、调整回包任务优先级、本地即时反馈），规则不得移到主线程，除非另有实测与 ADR。完成：目标环境点击到绘制 p50 不超过 1 个刷新周期 + 2 ms，有前后数据 | PA08 | [PA08-1 有头复现](../operations/evidence/2026-10-26-pa08-headed-baseline/README.md)、[PA08-2 WKWebView](../operations/evidence/2026-10-26-pa08-2-wkwebview/README.md)、[PA07 缓解原型](../operations/evidence/2026-10-26-pa07-mitigation-prototype/README.md)（Worker 侧 `stepTimings.ts` 等仍未提交，见阻断 B3） |
| PA08 表现层采样扩面 | 基本完成（2026-10-26，ZCode B道；有头 Chrome 120Hz、WKWebView 真实点击 n=14、三浏览器整局、大包/满手牌、≥30 分钟长时与内存、iOS 模拟器整局、Android 运行验证全部落档；余：Safari/Firefox 真实点击、Android 整局量化、pile 载荷大包） | 首轮（2026-10-26 凌晨）只覆盖 headless、60 Hz、「自动一步」；现已扩到有头 Chrome 120 Hz、Tauri WKWebView、三浏览器、满手牌/大包、32 分钟长时、iOS/Android 模拟器（外链驱动 `scripts/web-frame-external.mjs`） | 用同一采样层与驱动覆盖：有头 Chrome、Tauri 桌面、Safari/Firefox、Android/iOS、120 Hz 显示、出牌/选目标/应答/取消/选将等人工路径、大扩展包与满手牌、≥30 分钟长时与内存。完成：每个目标一个证据目录，含原始 CSV 与 summary | — | [PA08-1](../operations/evidence/2026-10-26-pa08-headed-baseline/README.md)、[PA08-2](../operations/evidence/2026-10-26-pa08-2-wkwebview/README.md)、[PA08-3](../operations/evidence/2026-10-26-pa08-3-browsers/README.md)、[PA08-4](../operations/evidence/2026-10-26-pa08-4-stress/README.md)、[PA08-5](../operations/evidence/2026-10-26-pa08-5-longrun/README.md)、[PA08-6](../operations/evidence/2026-10-26-pa08-6-mobile/README.md) |
| PA09 桌面端原生执行引擎 | 设计草案（[ADR 0784](../adr/0784-desktop-native-engine-prototype.md)，2026-10-26）；原型未开始 | Tauri 桌面在 WebView 里跑 WASM 引擎；wasm 下 AI 时限静默失效（`engine-native/src/ai.rs` 中 `started = None`），没有原生线程池 | 原型：引擎在 Tauri Rust 进程（或 sidecar）原生运行，沿用现有 Worker 协议；对比 PA08 指标、AI 时限是否生效与线程池收益；用 ADR 记录是否切换默认。完成：原型 + 同负载对照 + ADR | PA08 | 内核复核、帧间隔证据 |
| PA10 视图协议增量化 | 已裁决维持现状（2026-10-26）：满手牌 30 张 view_json+parse+adapt p95 ≤1ms、p8 序列化 p99 ≤0.2ms、VM 大包零长任务——未触发预算条件 | 现每步传整份 `view_json`：8 座约 1.6 K 字符，解析 p99 ≤0.2 ms，**目前不是瓶颈** | 仅当 PA08 在大包、满手牌、8 座长局等场景显示序列化或 Vue 刷新超出预算时，按性能架构 §4 实施版本化二进制增量；否则维持现状并随 PA08 复测 | PA08 | [PA08-4 压力场景](../operations/evidence/2026-10-26-pa08-4-stress/README.md)、[PA08-1](../operations/evidence/2026-10-26-pa08-headed-baseline/README.md) |
| PA11 原生 GPU 牌桌原型 | 草案（2026-10-26）：PA08 全场景实测 DOM 余量充足（帧 p99 ≤10.3ms@120Hz、满手牌序列化 ≤1ms、32min 零长任务），性能动机不成立；投入待用户拍板（ADR 0785 草案） | 当前表现层为 Vue + DOM；性能架构 §8 的原生保留式牌桌尚无原型 | 用等价素材实现一个牌桌场景的 wgpu 原型，与 WebView 版在同设备比较帧间隔、点击到出帧与功耗；用 ADR 决定是否投入。完成：原型 + 对照数据 + ADR | PA08、用户裁决 | [ADR 0785 草案](../adr/0785-wgpu-table-prototype.md)、[性能架构 §8](../../noname-performance-architecture.md) |
| PA12 单一引擎收敛 | 需裁决 | 旧线 `engine-core` 约 48.5 万行，承载 6152 技能目录；原生标准目录 215 技能；新上游内容仍先进旧线 | ① 由用户裁决是否冻结旧线新增规则、新内容直接进原生；② 原型：从现有 `.skill` DSL/IR 生成原生实现（架构提案 B1），选含选择与续行的代表技能，对照上游行为与保存恢复，成功后给出批量迁移计划；旧线退出仍按 NC12。完成：裁决记录 + 原型证据 + 迁移计划 | 用户裁决、NC08 | 内核复核 |

## 当前阻断（2026-10-26 扫描，按优先级）

| 编号 | 问题 | 处理与完成条件 | 证据 |
|---|---|---|---|
| B1 / P0 | **定向通过（2026-10-26 20:15，ZCode A道）**。原问题：**旧线编译失败**——PA06（`3eb67194`）把 `rule_vm::RuleVm::execute`/`resume` 改为只收 `VerifiedModule`，但 `crates/engine-core/src/vm_host.rs` 三处调用（699/764/805 行）仍传 `&BytecodeModule`；干净 HEAD `afdc39ab` 上 `cargo check -p engine-core` 报 E0308，engine-core、engine-wasm、game-server 均无法构建。`--all-targets` 下 PA06 实际遗留 **216 处**（engine-core 3、content-compiler 测试 205、示例 8），连带 conformance 2、devtools 1+4、phase4 测试 1 | **已做**：旧线 registry 原本装载期已 `verify_module` 再 `insert(m.clone())`，改为持有 `VerifiedModule`（`rule-vm` 给 `VerifiedModule` 补 `PartialEq/Eq` 与字节相同的透明 serde——反序列化即校验）；`vm_host` 六个 `run_module*`/`resume_module` 签名改收 `&VerifiedModule`；工作区全部调用方同步，PA06 装载期校验语义未回退。干净 HEAD + 补丁：`cargo check --offline --workspace` **通过**；`--all-targets` 只剩 B4；rule-vm **15/0**、content-compiler **533/0**、conformance **4/0**、devtools **14/0**。首批三大套件被磁盘耗尽打断；用户裁决清理后在干净 HEAD `f49e9f89` 导出上**补跑完成**：`cargo check --offline --workspace` 通过，engine-core **3779/0/1**、game-server **115/0/0**、engine-native **1546/7/71**——7 失败（`l20_liegong_batch3` 2、`nc08_upstream_liegong_xinkuanggu` 4、`nc08_skill_mapping_inventory` 1）全部是 `unknown skill fz_liegong/fz_xinkuanggu`，根因为 B5 的未提交包数据，补入工作树版数据即 7/7 通过，与本批无关。导出相对 HEAD 只补了未跟踪数据（9 个 shahai 文件、`archive/` 软链），见证据 §6 | [B1 证据](../operations/evidence/2026-10-26-b1-vm-host-verified-module/README.md)；原扫描：干净 HEAD 导出 `cargo check --offline -p engine-core` → `vm_host.rs:699 E0308` |
| B2 / P1 | **进行中（2026-10-26 20:40，ZCode A道）**——批①已提交：12 处 PA 证据 README 的 17:30 时间更正（提交时刻/父提交/数据文件 mtime 逐条核对一致，[批①证据](../operations/evidence/2026-10-26-b2-docs-time-corrections/README.md)）；工作树脏项清点：content/original 75、engine-core 55、docs/operations 38（其中 10-24/25 的 22 个证据目录整体未入库）、rule-api 33、game-server 31、web-client 24、content-compiler 19 等，下一批 P1 四项。**大量已登记为完成的改动仍未提交**：工作树约 312 项（content 75、engine-core 55、rule-api 33、game-server 31、web-client 26、content-compiler 19 等），包含进度主稿记为已关闭的 P1 四项（如 `apps/game-server/src/room_native_actor.rs`、`apps/web-client/src/engine/networkClient.ts` 最后提交仍为 10-19）及 UP37-N01/R01/R03、I01 同步等；HEAD 不能单独复现这些结论 | 由各项原执行者（或领取者）逐批核对：工作树内容即证据所述内容 → 跑该批登记的验证命令 → 只暂存该批文件并提交，提交信息引用证据目录。不得整体 `git add -A`，不得回退无主改动；核对不上的改动在台帐登记为「无主在途」 | `git status --short`、`git log -1 -- <文件>` |
| B3 / P2 | **定向通过（2026-10-26 20:28，ZCode A道）**。原问题：**PA07 原型半提交**——已提交的 `client.ts`（`f7b8e46d`）依赖未提交的 Worker 侧 SAB 镜像（`stepTimings.ts`、`engine.native.worker.ts`、`types/engine.ts`）与 `18edc033` 推迟入库的 `nativeBackend.ts` perf=1 `stepMark` 插桨；后两者同文件混有他人身份局 `packsJson`/公开事件投影改动 | **已做**：按行级拆出 B3 四文件（+48/−3）提交，他人 hunk 原样留在工作树（归 B2）。暂存树导出的真实构建上：`pnpm test` 48/0、vue-tsc 0 错、vite 独立 outDir 构建通过；驱动冒烟 60 次真实点击——perf=1 Worker 四段分段 n=60 全非空；`--coop --latency sab` 下 `initLatencySab=true`、`sabDebug seq=63 early=60`（60/60 提前解析），两轮 console_errors 0。PA07 自身「端到端未改善待甄别」状态不变 | [B3 证据](../operations/evidence/2026-10-26-b3-pa07-worker-sab-commit/README.md)；`git show --stat f7b8e46d` |
| B4 / P1 | **已提交测试依赖未跟踪内容文件**（2026-10-26 B1 复现时发现）：`crates/engine-core/tests/original_merged_shahai/original_shahai_rewrite_batch.rs:105–115` 与 `tests/original_merged_choose.rs:26`（`d5788bb9`，10-21）`include_str!` 的 `content/original/ported/shahai/shahai_meta_{gujian,gwent,hearth,mtg,ow,swd_characters,swd_combat_and_cards,xianjian,yunchou}.skill` 9 个文件在工作树为 `??`，旧 `shahai_meta.skill` 为 `D`；干净 HEAD 自 10-21 起编不过这两个测试二进制，`cargo test -p engine-core` 在干净 HEAD 上无法整体运行 | 属 B2 家族：由 shahai_meta 拆分的原执行者核对 9 个文件内容 = 其证据所述 → 跑 `cargo test --offline -p engine-core --test original_merged_shahai --test original_merged_choose` → 连同 `shahai_meta.skill` 删除一起提交；核对不上则登记「无主在途」并改测试读取路径。完成：干净 HEAD `cargo check --offline --workspace --all-targets` 通过 | [B1 证据 §3.2](../operations/evidence/2026-10-26-b1-vm-host-verified-module/README.md)；`git status --short content/original/ported/shahai/` |
| B5 / P1 | **已提交测试依赖未提交包数据**（2026-10-26 B1 补跑时发现）：`crates/engine-native/tests/l20_liegong_batch3.rs`（`d5788bb9`，10-21）、`nc08_upstream_liegong_xinkuanggu.rs`（`6c252f02`，10-18）、`nc08_skill_mapping_inventory.rs`（`7708d9b7`，10-20，基线 94）共 7 个测试 `debug_grant("fz_liegong"/"fz_xinkuanggu")`，宿主实现已在 HEAD，但 `packages/extensions/mark-supplement-upstream-liuli/{assets/cards.json,package.npack.json}` 含这两个技能的版本为工作树 `M` 未提交（HEAD 版本 0 处）；干净 HEAD `cargo test -p engine-native` 恒有 7 失败（1546/7/71） | 属 B2 家族：由 L20 批 3 / NC08 的原执行者核对两个文件内容与证据（双哈希、`gen_cardpile_packages` 口径）→ 跑 `cargo test --offline -p engine-native --test l20_liegong_batch3 --test nc08_upstream_liegong_xinkuanggu --test nc08_skill_mapping_inventory` → 只提交这两个文件；核对不上则登记「无主在途」。完成：干净 HEAD engine-native 全量 0 失败 | [B1 证据 §6.3](../operations/evidence/2026-10-26-b1-vm-host-verified-module/README.md)（补入工作树版数据后 7/7 通过） |

## 37 个扩展来源

逐来源状态、E01–E06/G0–G5、组合/平台/恢复条件以[两仓扩展台账](../migration/upstream-extension-reconstruction.md)为准；**目前没有任何来源完成 G0–G5 与八项验收**，不能以“目录未变”或“包可加载”代替。下一步按该台账逐来源填 G2–G5 证据。

## P1 已关闭（不得重做）

1. WASM 坏存档受控拒绝、原句柄继续：P1 证据。
2. 慢 WS 不阻塞房间 actor：有界队列/writer 超时。
3. 浏览器重连 pending/原 key：同实例跨重连与跨刷新路径均已实现；UI 取消按钮已接线。
4. 服务端存档目录单写者 + 崩溃重启：锁与 SIGKILL 回归。
