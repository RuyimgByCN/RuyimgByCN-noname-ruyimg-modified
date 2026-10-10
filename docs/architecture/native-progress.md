# 原生重写进度与下一步

本页是当前状态主稿，按 **2026-10-10** [进度核对](../operations/evidence/2026-10-10-progress-check/README.md)更新（源码为本地 git 基线 `72583b2b`，规则 v193、NGSA 存档格式 v41）；同日按可领取的粒度重排了待做表（[待办整理核对](../operations/evidence/2026-10-10-todo-handoff/README.md)，只读源码，未重跑全量），并清理了过期文档（[清理核对](../operations/evidence/2026-10-10-doc-cleanup/README.md)）。目标见[性能架构 v3.3](../../noname-performance-architecture.md)；逐来源扩展状态见[两仓台账](../migration/upstream-extension-reconstruction.md)；逐版本规则语义见[规则基准](native-rules-baseline.md)；按批追加的历史流水与 10-10 事故恢复逐轮记录移至[批次流水存档](native-progress-history.md)，实现与决策过程见[证据目录](../operations/evidence/)和[ADR 索引](../adr/INDEX.md)。

## 当前核验快照

> **2026-10-10 事故与恢复**：v192（机关桶）批开发中 `crates/engine-native/src/content/host.rs` 被误截断为空。经会话记录回放与证据语义重建，九轮修复后全量转绿，14 golden 补齐 v191/v192 两批重录（[恢复证据](../operations/evidence/2026-10-10-replay-golden-v192-recovery/README.md)，逐轮数字见[流水存档](native-progress-history.md#2026-10-10-hostrs-截断事故恢复记录)）。当日 `noname-next` 建立仅本地使用的 git 仓库（基线提交 `72583b2b`，pre-push 钩子拦截推送），此后以提交为回退点。恢复遗留项见下方“后续待做”T2–T3（T1 已于同日完成）。

| 口径 | 实际结果（2026-10-10，git `72583b2b`） | 边界 |
|---|---|---|
| 当前版本 | 规则 **v193**（`STANDARD_RULESET_VERSION`）、NGSA 存档格式 **v41**（`ARCHIVE_FORMAT_VERSION`）；逐版本语义见[规则基准](native-rules-baseline.md)，v193 为 T10 机关桶收尾所升 | 版本号只说明编码与语义已递增，不等于上游等价 |
| 原生全量 | `cargo test --offline -p engine-native --no-fail-fast`：**527 通过、0 失败、1 ignore**（2026-10-10 T10 后复跑，24 个测试目标，含 14 个 golden 场景（v193 重录）、15 个包真实宿主生命周期测试、存档加固负例）——[日志](../operations/evidence/2026-10-10-t10-jiguantong-v193/engine-native-tests.log) | 全绿限于现有测试；host.rs 14 个宿主行为分支已于 10-10 按上游逐项核对并改写为有据的 no-op（T1 完成），效果仍靠内核钩子承担 |
| 装载与包管理 | `content-runtime`+`engine-ai`+`content-native-loader` 合计 **63/0/0**（16 个目标）——[日志](../operations/evidence/2026-10-10-progress-check/deps-tests.log) | 正式客户端/服务端尚未接入原生装载与规则执行 |
| CI 门禁 1b | `RUSTFLAGS="-D warnings" cargo check -p engine-native --lib`：**通过（exit 0）**（2026-10-10，独立 target 目录）——[日志](../operations/evidence/2026-10-10-t2-t3-residual-cleanup/gate1b-check.log) | lib 12 条警告已于同日清理（T3 完成）；测试与示例目标的约 576 条警告不在本门禁范围（D3 未决） |
| 上游来源 | 清点脚本重跑与 9-23 基线**不一致**：noname HEAD 快进到 `df2682e6`（8 个扩展目录未改）；Liuli 10 个来源实例内容变化（10-08 stash/reset 后本地修改 50→7 个文件；同日复算其中 9 个只是换行差异，见[复算](../operations/evidence/2026-10-10-doc-cleanup/liuli-crlf-check.txt)）；仍为 37 项，基线未重录——[清点复核](../operations/evidence/2026-10-10-progress-check/inventory-check.txt) | 变化来源的功能清单与切片影响待复核（T4） |

全工作区 `cargo test --workspace` 等 CI 门禁已于 2026-10-10（git `e1ffc3fd`）实跑并留存日志：**workspace 5098/0/2、conformance ALL PASS、cross-target 全过、sync-verify 通过、web-client 构建成功**——[T5 证据](../operations/evidence/2026-10-10-t5-workspace-verification/README.md)；旧线 `engine-core` 单独全量最近一次是 10-07 [维护批](../operations/evidence/2026-10-07-maintenance-batch/README.md)的 3777/0（本批 5098 为两线合计）。

## 已完成并复验的切片

| 项目 | 已交付与本次确认 | 验收边界 |
|---|---|---|
| P0 回放与目录 | 拆牌循环重复入牌已修（9-24）；标准目录现为 124 张声明表（v74 核定，`standard_catalog_inventory` 守护）；14 个 golden 场景随版本逐批重录，最近一次为 2026-10-10 v191/v192 补录，本日通过 | 重录依据见[9 月 24 日证据](../operations/evidence/2026-09-24-replay-golden-v60/README.md)与[v192 恢复证据](../operations/evidence/2026-10-10-replay-golden-v192-recovery/README.md)；124 张含原生扩充段，与上游 108 张的差异见[规则基准](native-rules-baseline.md#上游标准牌表对照) |
| P0 N11 机器 tag 与解码加固 | XiaoJi=43、Dismantle=29；tag 唯一/往返、枭姬挂起和错 tag/短输入负例通过；未知牌种/尾部载荷/势力编码加固通过 | 加固时格式为 v2，现为 v41；逐次格式变化见[规则基准](native-rules-baseline.md) |
| 非法命令与注册表 | 主动使用无懈返回 WrongPhase；`deny(unreachable_patterns)`、数据键表及牌种伴随技能表检查已落地 | 只证明已登记非法命令和表项，不能替代所有规则分支验收 |
| C0 / E02 装载器 | `content-native-loader` 数据桥接、严格错误检查、**八份测试桥接全部迁移**（古剑/玩点论杀/欢乐/炉石/gwent/运筹/3D精选/五行）；同名名称/技能名单重载复用已有验收；standard 46 技能编号声明表锁定 | 正式客户端/服务端尚未调用装载器（T6③）；cardpile 的 `pile.json` 仍在测试内解析（T6①）；全局符号表仍永久保留首次出现的唯一名称/名单（T6②）；新规则仍须改引擎重编译（T8） |
| P0 事故恢复 T1 | `content/host.rs` 14 个 `TODO(事故恢复)` 空分支已按上游逐项核对并改写为有据的 no-op（4 项上游仅 mod/group；10 项效果在 kernel/engine/state 内联），并补元宝肉“每回合仅一张额外杀”上限负例、启用 `def_ybr` 断言 | 只改注释与测试，未动行为；engine-native **525/0/1**、14 golden 不变，见[证据](../operations/evidence/2026-10-10-t1-host-recovery-stubs/README.md) |

## 完成状态

| 领域 | 已核实交付 | 仍需处理 |
|---|---|---|
| 原生内核与恢复 | 顺序事件执行、类型化暂停、内容锁建局/恢复、NGSA 存档（现 v41）、所列选择与持牌恢复回归 | 其余存档异常输入（未知牌种与载荷读尽已于 10-07 加固）、全规则/模式、长局和跨目标恢复验收 |
| AI 与信息边界 | 原有单选合法回退等反例已修；同步 BoundedPlanner 与部分座位视图用例通过 | 工作池、运行中取消、候选质量及全部模式/扩展的信息隔离 |
| 扩展基础与切片 | 15 个转换包（数据型卡牌/武将包），全部有真实宿主全生命周期测试（10-10 通过；ow/xianjian 同日补齐，T7）；部分宿主行为与内容锁恢复；具体包和张数见[台账](../migration/upstream-extension-reconstruction.md) | 两仓 37 来源整包原生验收仍为 **0/37**（生命周期只是整包条件之一）；开放第三方执行/SDK、模式/AI/UI/联机和已登记规则差异 |
| 标准玩法 | 上游 standard 33 将全对齐（另原生新增曹仁）；身份/国战原型与 versus/斗地主/Boss/KOF 模式基线（v80–v84）；逐技对照五批与逐人询问、仁德累计等局部修复 | [规则差异表](native-rules-baseline.md#本地上游对齐的新增核查)的已登记差异与逐技能语义；各模式变体与 KOF 多局连续会话（宿主层） |
| 产品与性能 | 原生 runner、同宿主跨进程样例保留历史证据 | 客户端/服务端仍用旧线；原生产品接入、目标设备与同负载收益未验收 |

**原有扩展机制必须保留。** 玩家应能独立安装、启停、升级、卸载；作者应能发布武将、技能、卡牌、模式、规则、AI、资源、配置和界面扩展。用户指定的两仓全部 37 来源均在范围内；旧 JS 零改动范围未决，不影响保留这些能力。详见[宿主契约](native-extensions.md)与[两仓台账](../migration/upstream-extension-reconstruction.md)。

## 后续待做（按阻塞关系）

2026-10-10 第二次整理：本节只列未完成项，写到可以直接领取的程度，包括位置、做法和完成条件。逐处源码定位、警告逐条结论和上游对照见[待办整理核对](../operations/evidence/2026-10-10-todo-handoff/README.md)。整理前的原表（含已完成项的完整叙述）原样存于[批次流水存档](native-progress-history.md#2026-10-10-整理前的待做表原文)。性能项 O1–O5 见下一节。

**接手方式**

- **领取与回填**：
  - 开工前用 `git status` 确认工作区干净并记下 HEAD，把下表该行“状态”改为“进行中（日期，执行者）”。
  - 多个 agent 并行时，写入本页前先重读。
  - 完成后按 [AGENTS.md](../../../AGENTS.md) 第 2–3 条回填结论、命令、结果和证据目录（`docs/operations/evidence/<日期>-<主题>/`），并同步规则基准、台账或覆盖清单里的对应条目。
- **回退点**：`noname-next` 的 git 只在本地用，不 push、不加 remote（pre-push 钩子会拦截）。每批开工前和通过后各提交一次。`crates/engine-native/src/content/host.rs` 约 1.4 万行，10-10 曾被截断为空：只做定点编辑，提交前用 `git diff --stat` 核对改动规模。
- **构建目录**：共享的 `target/` 可能有其他会话在用，不要执行 `cargo clean`。Gate 1b 这类要改 `RUSTFLAGS` 的检查放到独立的 `CARGO_TARGET_DIR`，用完删除。
- **验证口径**：

  | 改动 | 要跑的验证 |
  |---|---|
  | 原生内核 | `cargo test --offline -p engine-native --no-fail-fast`（10-10 为 525/0/1） |
  | 装载或包管理 | 再加跑 `cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast`（63/0/0） |
  | 规则语义 | 递增 `STANDARD_RULESET_VERSION`；14 个 golden 先按 AGENTS.md 第 4 条查明原因，再重录 |
  | 存档编码 | 递增 `ARCHIVE_FORMAT_VERSION` |
  | 覆盖清单 | `cargo test --offline -p engine-native --test replay_scenarios`（清单联动守护） |

| 编号 | 优先级 / 领域 | 状态 | 任务、位置与做法 | 完成条件 |
|---|---|---|---|---|
| T2 | P0 / 事故恢复 | 已完成（2026-10-10，ZCode） | 删除 `host.rs.pre_assembly`、`host.rs.recovered-v192` 两个备份与 4 处按环境变量开关的 eprintln（resolution.rs `push_frame`、`plan_triggers`×2，machine.rs `expire_temporary_skills`）；`xly_state_growth_probe` 的 ignore 说明同步去掉 PT_DBG 字样 | `rg 'env::var' crates/engine-native/src` 与 PT_DBG/XLY_DBG 均 0 命中；全量 **525/0/1** 与改前一致——[证据](../operations/evidence/2026-10-10-t2-t3-residual-cleanup/README.md) |
| T3 | P3 / 维护 | 已完成（2026-10-10，ZCode） | 逐条清理 lib 12 条警告（host.rs 7 条、kernel.rs 2 条、engine.rs/presentation.rs/snap.rs 各 1 条）：冗余绑定改判空/删行、`self.card = self.card` 删除、空 if-let 块删除（保留"伤害牌从 cards[0] 取"注释）、多余 `mut` 与未用导入删除；删除只打印无断言的 `tests/dbg4.rs`（等价断言覆盖已存在于 `savage_nullify_after_deck_drain_conserves` 与 boundary_snapshot 暂停往返） | `RUSTFLAGS="-D warnings" cargo check -p engine-native --lib` 独立 target 目录 **exit 0**；全量 **524/0/1**（较改前少 1 为 dbg4 删除，0 失败），golden 不变——[证据](../operations/evidence/2026-10-10-t2-t3-residual-cleanup/README.md)。测试与示例目标的约 576 条警告仍留 D3 |
| T4 | P1 / E01 | 待领取 | 复核上游变化。noname 10-08 快进了 42 个提交（`5c02a745..df2682e6`）。扩展目录和 `card/standard.js` 没改。变化集中在三处：`mode/guozhan`（33 个文件）；`card/{guozhan,huodong,jiubian}.js`；`library/element/{content.ts,player.js,character.js}` 与 `library/skill.js`。另有 `mode/{chess,single,tafang}.js` 有改动。标准武将只有 `standard/skill.js` 里洛神判定回调的签名变了。Liuli 有 10 个来源实例的字节变化（名单见台账）：其中 9 个只是换行（CRLF→LF）差异，内容与 9-23 一致；史诗卡牌另有 3 个文件共 148 字节无法由换行解释（[复算](../operations/evidence/2026-10-10-doc-cleanup/liuli-crlf-check.txt)）。已转换的 wuxing、cardpile 不在其中 | 判断国战原型和已登记的标准结算语义是否受影响，必要时补登规则差异。史诗卡牌按 D1 的结论复核 E01 功能清单，其余 9 个 Liuli 来源只有换行差异，无需复核。以当前工作区重录 `inventory.json` 基线，或在台账里写明保留 9-23 基线的理由 |
| T5 | P1 / 验证 | 已完成（2026-10-10，ZCode） | 五项门禁实跑（git `e1ffc3fd`，含同日全部改动）：`cargo test --workspace --no-fail-fast` **5098/0/2**；`cargo run -p conformance` **ALL PASS**；`bash scripts/cross-target-hash.sh` **全过**（wasm32）；`node scripts/sync-original-content.mjs --verify-only` **通过**（366 复制+2703 链接）；`pnpm --dir apps/web-client build` **成功**。均 exit 0，完整日志入证据目录 | 五项全绿，失败项为零无需另立任务；快照与“全工作区”段已回填——[证据](../operations/evidence/2026-10-10-t5-workspace-verification/README.md)。旧线单独口径见 10-07 维护批 3777/0 |
| T6 | P1 / C0、E02 | ① 已完成（2026-10-10，ZCode）；②③ 待领取 | 装载器收尾。① `content-native-loader` 新增 `bridge_pile`（pile.json 严格解析，counts 键 BTreeMap 排序保证生成卡牌序仅由数据决定；错误文案 cards.json→包数据），`g1_cardpile.rs` 删除内嵌 `parse_spec` 改调装载器（3 处调用点），验收测试补正反 2 例 | ① g1_cardpile 5/5 经装载器通过（113 张包数据不变）、loader 验收 3/3——[证据](../operations/evidence/2026-10-10-t6-loader-pile/README.md)。② 全局符号表永久保留首次出现的唯一名称和名单，要定总量上限或回收策略：策略写成 ADR，有重复装载和上限测试。③ 正式客户端/服务端接入“安装包 → 装载 → 原生建局”，和 T13 衔接：产品路径有实测证据 |
| T7 | P1 / E02、C3 | 已完成（2026-10-10，ZCode） | `g1_wudian.rs` 新增 `ow_real_host_full_lifecycle`、`xianjian_real_host_full_lifecycle`，与 wudian 同款五步：独立安装+激活 → 安装包→原生建局（ow 24 将、xianjian 36 将，均 0 卡；目录断言士兵76=突击/目镜、赵灵儿=回盟/天蛇/天蛇2）→ 对局中卸载 → 在局锁定副本续玩 → 撤销后重装/重新 import 被拒 | 两个包测试通过，`*_real_host_full_lifecycle` 达 **15/15**；全量 **526/0/1**；台账 E02 与复核段同步——[证据](../operations/evidence/2026-10-10-t2-t3-residual-cleanup/README.md) |
| T8 | P1 / E03、G1 | 可做原型，定案待 D2 | 开放规则。包里的技能现在只能映射到编译期的 `HostBehavior` 枚举（`host:*` 键），新规则必须改引擎再重编译，达不到[扩展契约](native-extensions.md)里“新包无需重编译客户端”的要求。按[演进提案 C1](architecture-evolution-plan.md#c1-扩展执行后端选择)，用同一个真实扩展比较各候选后端，再写 ADR | 不重编译就能安装含新技能或新牌的包；选择挂起/恢复、随机、内容锁与存档、超限和故障回滚都有实测 |
| T9 | P1 / E01、G0 | 部分完成 | 37/37 来源的功能面基线已完成（10-07/08，含龙血玄黄 LX.1–LX.3）。剩余：每个来源的旧行为动态样本、菜单与规则分支的语义清单、目标接口和验证入口登记；龙血玄黄战斗内的触发轨迹（迁移前置项）；改进工具，让它能统计壳扩展的 `const card = {}` 段。逐来源状态见[台账](../migration/upstream-extension-reconstruction.md) | 台账里每个来源都有行为样本证据、目标接口和验证入口 |
| T10 | P2 / N08 | 已完成（2026-10-10，ZCode） | 机关桶补齐两支：kernel 机关牌工厂臂把目标表反转为结算序（上游 `reverseOrder`）；host Act 完成点按日志实伤判定（无以本牌归因的 `DamageDealt`）补 contentAfter 使用者摸 1，随后本体入弃堆。回归补「有人受伤不摸」断言，新增逆序+全员弃牌摸一测试。规则升 **v193**；存档格式不变（v41）；14 golden 重录（版本回退探针证明位移唯一来源是版本字段，场景命令与断言不变）；规则基准补 v193 条目撤销 v192 两条登记差异 | 机关桶回归 2 支通过；`replay_scenarios` 2/2（重录后含联动守护）；全量 **527/0/1**——[证据](../operations/evidence/2026-10-10-t10-jiguantong-v193/README.md) |
| T11 | P2 / N08、N11 | 长期 | 其余规则对齐：<br>• 规则基准已登记的逐技能差异。<br>• 模式变体：versus 替补和 4v4、斗地主特殊开局、Boss 专属技能包，以及在宿主层的 KOF 多局连续会话。<br>• 同一命令下的事件、随机流和恢复轨迹对照。<br>• 10-10 新发现一处未登记差异：掠影在 v154 记为“登记差异：无”，但上游要求另有其他有牌角色才能发动，而且是可选发动；原生 `kernel.rs:3087` 只要目标有牌就自动获得一张 | 逐项修复或登记；语义有变化就递增规则版本 |
| T12 | P2 / E04–E06、G2–G5 | 整包 0/37 | 逐来源推进见台账。下一批切片已经明确：<br>• swd 机关术×2、机关妖术×2。<br>• mtg 其余的地。<br>• zhenfa 的 5 张 deferred。<br>• 玩点论杀 2 条未映射技能（duoqi/youzhan，暂缓）。<br>之后是模式、AI、配置、界面、资源、战役档案和联机 | 每个来源完成功能、组合、保存恢复、安装升级和目标平台验收后，才上调整包数 |
| T13 | P2 / N10、N12 | 待领取 | 两部分：<br>• AI：工作池、运行中取消、合法座位视图、策略质量。<br>• 产品：正式客户端/服务端接入原生（含 T6③）；跨进程与目标平台运行；帧率和延迟实测 | 各项都有实测证据；旧线的测试和性能数据单列 |
| T14 | P3 / 文档 | 已完成（2026-10-10，ZCode） | ① 覆盖清单 `kof_session` 行按 `kof_session_consistency_and_midrestore` 实测覆盖改写：状态“近似”→“已验证（会话行与单局恢复路径）”，测试列表补齐，边界改述为“语义矛盾行（字段间不一致）仍接受；多局连续会话循环与示例策略超时判负在宿主/示例层”。② 原生注释与 legacy-reference 的“莫守”→“墨守”、“闪弹”→“霰弹”（engine/catalog/standard/host/kernel/snap/g1_wudian 共 24 处；规则基准与进度流水经 10-10 文档清理已用对名称；带日期证据文档保留原文） | `replay_scenarios` 联动守护 2/2 通过；注释改动不涉行为——[证据](../operations/evidence/2026-10-10-t14-doc-naming-kof-session/README.md) |
| T15 | P1 / 旧线 | ① 已完成（2026-10-10，ZCode）；②③ 待领取 | **①核查结论：735–753 批已完成，仅宿主声明滞后**（领取表 735–753 全登记；content-compiler additive 常量 726…753 缺 743 与领取表一致；`ResolveXiangshu` 等宿主实现在位；源 npack 745/753/751 为编译器计算）。已按流程收口：`RULE_VM_ABI_VERSION` 与 ABI-MAX 734→**753**（compat 对 host 单调，31 个已部署包零拒装）；`rebuild-pack.sh` 重打包 sp/sp2/xianding（与源模块逐字段一致）并部署双副本（714/713/706→745/753/751）；连带把 cardpile 双转换包按生成器重生成至新默认 753（各 1 行 diff，logic/asset 哈希不变）<br>• ②**哈希缺口**：`hand_limit_bonus` 不在旧线 canonical hash（`crates/engine-core/src/hash.rs`）内，此前只登记在 [Arc 化记录](../migration/content-registry-arc-split.md)——待补编码（旧线 golden 按第 4 条处理）或写明有意排除。<br>• ③**移植余项**：10-08 同步后 catalog 有 19 将 39 技 `source_synced`（未移植），分布见 [delta 页 10-08 节](../migration/upstream-1.11.5-delta.md)——待排期或注明暂缓 | ① `cargo test -p engine-api` 4/4（含两枚账面守护）；全工作区升级后全绿（见[证据](../operations/evidence/2026-10-10-t15-abi-reconcile/README.md)补录）；旧线结果与本页快照分开记 |
| T16 | P3 / 文档 | 待领取 | 完成 10-10 文档清理中未审计的部分（[清理核对](../operations/evidence/2026-10-10-doc-cleanup/README.md) §5）：<br>• `architecture-evolution-plan.md`：S6 的 catalog 计数，S11/S13、A5、C0 落地段中“当前未修/待迁移”的陈述（10-07 已完成）。<br>• 规则基准的非历史段落：页首、“当前声明与验证边界”、“内容差异与后续验收”、“本地上游对齐的新增核查”表（如“80 张自造测试牌”“五谷、桃园、无中、借刀等牌种仍缺”）。v61 起的版本条目是历史记录，不改。<br>• 覆盖清单其余行（`kof_session` 行归 T14）。<br>• `docs/operations/` 下 18 份旧线阶段文档（phase10–30、governance-ops）：核对路径与“当前”类陈述，缺历史参考横幅的补上。<br>• `docs/rule-semantics/`、`docs/protocol/`、`author-guide.md`、`phase6-web-client.md`、fuzz、cross-target、desktop、content README：路径已查无失效，还需核对状态性陈述。<br>• 在[旧实现参考](legacy-reference.md)补“2026-10-10 文档整理”一节 | 每份文档的结论（删除、合并、修正或不动）写进证据目录，并说明依据；改动后跑链接检查和 `replay_scenarios` 联动守护；修改覆盖清单时不能破坏 `replay:<场景名>` 与带反引号的机制键 |

**需用户决定**

- **D1 史诗卡牌 3 个文件是否追查**：10-08 的 stash/reset 后，Liuli 本地修改从 50 个文件降到 7 个。复算表明消失的 43 个文件中 40 个只是换行（CRLF→LF）差异，内容与 9-23 一致（[复算](../operations/evidence/2026-10-10-doc-cleanup/liuli-crlf-check.txt)）。剩下史诗卡牌的 `asset/lib/better-scroll.js`、`cardPhantom.js`、`emoji.js` 合计有 148 字节无法由换行解释，9-23 的原文无法从 git 取回。10-08 的 stash 仍以悬空提交 `625f8770` 留在 Liuli 对象库中，但其中不含这 3 个文件，`Noname-ruyimg-Modified` 中的副本也复原不了（[清理核对](../operations/evidence/2026-10-10-doc-cleanup/README.md) §3）。需要追查吗（例如你手头有 9-23 前的副本），还是以当前工作区为准？T4 中史诗卡牌一项依赖这个决定。
- **D2 旧 JS 扩展兼容范围**：旧 JS 扩展零改动兼容到什么程度，还没有定（见[扩展契约](native-extensions.md)）。agent 可以按演进提案 C1 做对照原型、起草 ADR，但不要把 WASM、JS 或动态库预设为定案。T8 依赖这个决定。
- **D3 测试代码警告**：测试与示例目标里约 576 条编译警告，要不要纳入清理？Gate 1b 只检查 lib。

## 性能优化核验后的待办（2026-09-24）

O1 已于 2026-10-07 完成；截至 2026-10-10，证据目录中没有 O2–O5 的新测量或验收记录。

[优化假设复核](../operations/phase17-test-perf-infrastructure.md#2026-09-24-优化假设复核)确认若干旧线代码路径、文件数量和依赖配置问题，但原报告的 P0/P1、5–10 倍、80% 等收益没有可复查基准。`engine-core`/`engine-ai`/`engine-wasm` 是当前产品线，`engine-native` 是重构线；两线分别设指标，不把旧线提速记为原生架构验收。以下按“先测量、再局部改动、最后等价验收”执行。

| 编号 / 状态 | 处理计划 | 完成条件 |
|---|---|---|
| O1 / 已完成（2026-10-07） | `tempfile` 已移至开发依赖（src/ 零使用，仅 tests 经 content_runtime 别名）；dev-dependencies 中重复的 `content-compiler`/`content-schema` 已删除 | `cargo tree` 验证 engine-core 与 engine-wasm 正常依赖 0 命中 tempfile；engine-core 全量 3777/0、锁文件随检查更新；当时本机未装 wasm32 目标，真实 wasm32 构建未做（[维护批证据](../operations/evidence/2026-10-07-maintenance-batch/README.md)）；2026-10-10 起本机已有 `wasm32-unknown-unknown` 与 wasm-pack，随 T5 跨目标哈希一并验证 |
| O2 / 待测，构建效率 | 先确认空闲磁盘，在独立 `CARGO_TARGET_DIR` 固定工具链、机器和起始状态，测 `engine-core` 干净/热态 `cargo test --no-run` 的耗时、链接目标数、磁盘；不清理并行代理共享的 `target`。若确为主要瓶颈，挑 20–50 个文件试合并 | 试点前后测试名/数量/过滤行为相同、全套通过；比较干净与增量构建/磁盘后决定是否扩大，不预设 1,078→20 或 90% 收益 |
| O3 / 待测，旧线运行时 | 复用 `benchmark-runner` 记录 `query` 命中率、分配/耗时、`project_observer` 扇出、AI 节点成本；只对已证明的热点局部优化查询 key/来源、同状态哈希复用或 Store 遍历。优先保留已有命令提交与不变量校验 | 同负载前后分位数与分配数可复查；命令结果、规则状态哈希、事件、随机流、回放和存档恢复保持一致。没有证据前不增加跳过校验的 `submit_sim` |
| O4 / 待测，WASM 交付 | 固定 `wasm-pack`/`wasm-opt` 版本，对现有构建和体积优化配置做可重复实验；比较原始、gzip/Brotli 体积与加载耗时 | 浏览器功能回归通过，记录体积/启动数据后选择配置；不以原文“7.75 MB→1.5 MB”作验收值 |
| O5 / 条件立项，格式与结构 | JSON 快照/回放、动态派发、大文件拆分、Blake3 工具合并和目录重命名均暂不作为性能任务；只有实际剖析或维护缺陷指向它们时再建最小改动 | 若改存档/回放，版本化并保留旧 JSON 读取及哈希链校验；若拆文件或换派发，保持 API、测试与构建/运行指标不退化 |

上述优化不改变 E01–E06 和上游规则待办的完成条件；测量结果及每项是否立项应回填本页或对应证据，不把代码存在、测试通过和性能收益混为一项。

## 证据与维护

- 最新实跑是[2026-10-10 进度核对](../operations/evidence/2026-10-10-progress-check/README.md)（git `72583b2b`，规则 v192）；同日[待办整理核对](../operations/evidence/2026-10-10-todo-handoff/README.md)只读源码定位 T1–T14，除覆盖清单联动守护外未重跑测试；上一轮为[2026-10-07 进度核对](../operations/evidence/2026-10-07-progress-check/README.md)（v60），[2026-10-06 进度复核](../operations/evidence/2026-10-06-native-progress-audit/README.md)另含存档缺陷探针与来源清点。10-07 至 10-10 各批实现见对应日期的证据目录与[批次流水存档](native-progress-history.md)；更早[第 48 轮](../operations/evidence/2026-09-23-native-review-48/README.md)的全工作区结果只适用于当时 v59。历史数值只描述对应源码。
- `noname-next` 自 2026-10-10 起用本地 git 管理（不推送远端，`.git/hooks/pre-push` 拦截）；核验记录写明对应提交，回退以提交为准。
- [规则基准](native-rules-baseline.md)记语义和版本，[覆盖清单](../native-coverage-manifest.md)记机制范围，[扩展台账](../migration/upstream-extension-reconstruction.md)记逐来源状态。新增实现同步源码、定向回归、全量结果和台账。
- “反例已修”“已加测试”“子包切片完成”“整包验收通过”“上游规则等价”是不同结论；性能收益必须有同负载测量。
