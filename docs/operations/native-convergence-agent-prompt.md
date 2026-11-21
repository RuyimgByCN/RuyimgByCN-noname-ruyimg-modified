# 原生架构收敛完整执行提示词

执行 agent 读本文件，把下面代码块里的内容当作本轮完整指令执行。方案正文保存需求与验收条件，实施状态以进度主稿顶部记录和剩余台账为准。2026-11-21 核对后重写，替换 2026-11-18 版。上一版的队列第 0 项已完成，第 1 项做完 Batch 1.1–1.5 五批（U06+L06、L28、U08、U07、L29），已复核等价 625 → 656，证据体量已受控。本版有三处主要调整：要求连续做完全部剩余工作，并写明完成标准与停止条件；大来源必须用并行子 agent 推进；第二节换成 2026-11-21 的现状快照。

```text
请在 /Users/ruyimg/Documents/Claude/sanguosha/noname-next 把《原生架构收敛执行方案》剩余的全部工作做完。本轮目标是完成，不是推进一部分：按第四节队列一批接一批连续做，做完一批立即开始下一批，不要停下来汇报，也不要询问是否继续。只有第三节的“完成标准”全部达到，或者剩余每一项都卡在第三节列出的用户/外部输入上，才可以结束本轮。上一轮 Batch 1.1–1.5 的做法是对的，请保持：先写按上游语义会失败的测试，再改实现，由另一个 agent 复核，证据只留摘要。差别在于规模：剩余约 2,800 个规则条目与约 1,000 个界面/配置条目，必须用并行的子 agent 推进（见第八节）。

一、先读（按顺序），并遵守 /Users/ruyimg/Documents/Claude/sanguosha/AGENTS.md：
1. docs/architecture/native-progress.md 顶部 2026-11-19～11-21 的 Batch 1.1–1.5 五条记录
2. docs/operations/evidence/2026-11-21-batch1-5-l29/README.md（最近一批的格式范例）
3. docs/operations/evidence/2026-11-13-queue1-review/known-automatic-choice-gaps.json、docs/operations/evidence/2026-11-14-queue1-runtime/known-automatic-choice-gaps-01.json 与 remaining-scope.md（已发现差异清单）
4. docs/operations/evidence/2026-11-12-step1-choice-channel-prototype/README.md（交互通道与迁移五步配方）
5. docs/operations/evidence/2026-10-28-user-decisions/README.md 的裁决 19、23、25、34、35、36、37、38、40
6. docs/migration/g0/inventory_status.json、docs/migration/upstream-extension-reconstruction.md、docs/architecture/native-rules-baseline.md 顶部、docs/architecture/native-remaining-ledger.md 顶部、docs/architecture/native-convergence-plan.md
7. /Users/ruyimg/Documents/Claude/sanguosha/NONAME_NEXT_AGENT_SPEC.md（位于 Git 仓库外）

二、现状（2026-11-21 14:00 快照，开工时以实际为准）：
- main HEAD `af6ef4899`（Batch 1.4 U07），已推送 native-convergence，exact-head CI 44785367168 首跑 10/10 success。版本：规则 333／NGSA 107／content schema 5／ABI 759／WS8。
- 工作区有未提交的 Batch 1.5（L29）：41 个已跟踪文件改动、新测试 crates/engine-native/tests/l29_longxue_fixes.rs（已登记 tests/groups/l_rounds.rs）、新证据 docs/operations/evidence/2026-11-21-batch1-5-l29/。核对时实跑：行数门禁 9,385 文件通过、分层 0、三 crate 2518/0/6；workspace（自报 7914/0/8）、web-client、preflight 只有自报，未复跑。写这一批的会话已停止，本机没有其他 agent 在写；开工前仍要看 git status 与文件 mtime 再确认一次。
- 本提示词文件（docs/operations/native-convergence-agent-prompt.md）的改写也未提交：与 Batch 1.5 分开，单独做一个纯文档提交。
- 清单：已复核等价 656／简化近似 3,768／缺失 0（共 4,424）。按类别：skills 363/1,545、generals 159/675、cards 17/360、modes 0/215、configs 88/346、ui_hooks 29/478、surfaces 0/149。
- 按来源：U06、U07、U08 已 100%；L28 54/2、L19 60/9、L27 41/13、L25 193/64、L20 55/40、L22 6/2、U01 33/62、L10 8/9、L29 5/4、L06 2/1；L11 2/934、U05 0/1,621；其余 22 个来源 0 等价。整包八项签署 0/37。
- 队列：第 0 项完成；第 1 项已完成 U06+L06、L28、U08、U07、L29 五批；其余未开始。
- 工作树：.wt-l10（分支 swarm/author-l10，基于 352236724）有 2026-11-18/19 未提交的 L10 草稿（4 文件约 +756 行，涉及 rangjie 与 skills_cards.rs），作者不明，只作参考，不要直接合入或删除。.wt-l28／.wt-l29／.wt-u07-l10／.wt-u08-l28 干净，其分支已按批 squash 合入 main（不是祖先，不要再 merge）。/private/var/folders 下 8 个临时工作树（1 个 locked）与 stash@{0} worker_m2_wip 原样保留。
- 证据体量已受控（每批 12–16 KB）。2026-11-14 以前已入库的约 560 MB 证据是否改写历史仍等用户决定：不改写历史、不强推。
- 磁盘：数据卷已用 94%、约 62 GB 可用。并行编译会很快吃满磁盘：每个子 agent 的独立 CARGO_TARGET_DIR 用完即清（裁决 23），不清共享 target；可用空间低于 20 GB 时先清理再继续。
- 无 Android 设备，物理 iPhone unavailable，签名/公证凭据未配置。

三、完成标准与停止条件：
完成标准（全部达到才算本轮完成）：
1. 规则条目：generals／skills／cards／modes 共 3,334 项全部为“已复核等价”。唯一例外是第 3 条所列的待用户裁决项；每个例外都在 native-rules-baseline.md 有具体差异登记，并已报用户。
2. 界面与配置条目：configs／ui_hooks／surfaces 共 1,090 项全部为“已复核等价”，每项有同场景并排截图或实际消费链核对。裁决 38 范围内不内置第三方资源的项，按裁决保留参数结构与本地导入入口，登记差异即可。
3. 只有以下情况可以停下来等用户，而且要一次性列清：①需要升规则/存档/ABI/WS/schema 版本才能表达的交互，逐技能列出精确边界；②超出裁决 38 的资源取得方式；③证据历史是否改写；④真机、签名/公证凭据、用户本机 Spine 4.0 动皮对照；⑤第六阶段切换范围确认。提问后不要等待，继续做不受阻的部分。
4. Spine 4.0：每个特性一条测试，净室复核记录完成，并已向用户发出本机对照请求和操作指引。
5. 37 来源整包八项签署：仓内可做的维度全部完成并上调；只缺真机或签名的维度写明所需操作，台账保持“部分”。
6. 每个代码批都已提交、按裁决 25 发布、exact-head CI 通过并登记；工作区不留本轮未提交的改动。
7. 第六阶段：第 1–6 条成立后，向用户提交精确切换范围；得到确认后执行原生产品切换并退出旧执行入口。
停止条件：只有上述完成标准全部达到，或剩余每一项都卡在第 3 条的输入上，才可以结束。卡住时报告每项的具体输入需求，以及给了输入之后还要做什么。上下文变长不是停止理由：每批结束都把状态写进进度主稿，然后继续。

四、执行队列（按顺序；共用文件冲突之外的批次按第八节并行。每完成一批就提交、更新清单与台账，按裁决 25 发布并在 docs/operations/evidence/ci-runs.md 登记一行）：
0. 收口 Batch 1.5（L29）：重读工作区改动与证据 README，确认与记录一致；第七节门禁全部重跑，不沿用自报；只 stage 本批路径（不带 57 个历史未跟踪路径、.pi-lens.json 与本提示词文件）；提交，用 scripts/publish_decision25.py 发布，登记 exact-head CI。首跑失败另起一行，不改阈值。本提示词文件另做一个纯文档提交。
1. 队列第 1 项剩余：
   a. L10 史诗卡牌：reguhuo 质疑与响应、rangjie、k_danggu，以及其授予的 scsniqu／scstaoluan／scschiyan（现为进入阶段即自动结算；上游分别是 phaseUse 主动限次选目标、主动转化与实际使用、useCardToPlayered 杀且目标 he 非空）。
   b. U01 3D精选：62 项技能，按上游文件或武将分批，包括 juxian J01–J05、dddyeshen、dddguiying、dddfushi、dddlangzhi、dddfuzong，及 2026-11-13 重审登记的其余差异。
   c. L28 th_zhengjing：按上游实现小游戏结果与非空子集分配；若必须升号，按第三节第 3 条报用户。
   d. L06 wuxingpan 卡牌：S2 固定 1/3 与默认 0.3 的差异，按规则基准登记的配置边界实现可配置项并对齐默认值。
2. 队列第 2 项：L27（bolan、twzhouzu、reqimou、new_meibu 等剩余 6 项技能）、L19（_huzhu、ganran2、wujinmoshiRE1 等 9 项）、L20（kuanshi、tspowei、wengua）、L25（wumou、new_guixin、gongxin 等 11 项及 1 张卡）、L22 两项自动钩子。每项在上游实际运行比对，差异当批修复。
3. 大来源规则改写：
   · L11 太虚幻境（920 项规则条目：320 武将、372 技能、32 卡、206 模式节点），按上游文件或战役/模式分批；
   · U05 杀海拾遗（1,614 项规则条目：332 武将、1,007 技能、275 卡），按 9 个子包分批；
   · 每批以一个上游子包或 50–150 个条目为宜，每批独立提交、复核、发布。
4. 其余含规则项的来源：L01 AI优化（69 技能、1 卡）→ U02／L03 boss（23 武将、2 模式）→ U03／L04 cardpile（50 卡、1 模式）→ L15 手杀全扩（3 模式）→ L21 武皇争霸（1 模式）→ U04（2 模式）。
5. 界面与配置（configs／ui_hooks／surfaces 共 1,090 项）：
   · 先做 L08 十周年UI（44 配置、319 钩子、43 显示面）的牌桌布局：环绕式座位、身份标记、中央提示区与按钮、手牌区。在真实牌桌组件上实现，用 apps/web-client/scripts/tabletop-comparison-cdp.mjs 出并排截图，直到布局一致。
   · 再做 L14 手杀ui、L20 标记补充（34 钩子）、L09 千幻聆音、L24 皮肤切换、L25／L27／L29／L10 剩余显示面与配置、L01 的 63 项配置，以及其余各来源的配置与显示面。
   · 配置项核对其实际消费链与效果；截图原图放 docs/operations/evidence-raw/，入库只放缩略对照图。
6. Spine 4.0（裁决 35、37、38）：每个特性一条测试；由未读过实现的 agent 做净室复核并留记录；写好用户本机对照指引并发出请求。
7. 37 来源整包八项签署：按扩展台账原八项逐来源复核。第①项以已复核等价占比为准，第⑧项列出该来源全部剩余差异。规则项已 100% 的来源（U06、U07、U08 起）随做随签。门禁为红或未发布时不上调。
8. 外部条件：PA08 真机 → PA11 判定、NC12 真机温控与移动端目标平台、三平台安装包签名/公证。把设备接入、证书配置所需的具体操作写成清单报给用户；在此之前把这些项在仓内能做的部分全部做完，例如脚本、CI 配置、模拟器验证。
9. 第六阶段：按第三节第 7 条执行。PA09 已由裁决 36 关闭。

五、每个条目的完成做法与复核（第四节第 0–5 项通用）：
- 规则实现：逐步对照上游 trigger／filter／content，时机、条件、次数、限定技、判定、随机（走引擎 RNG）全部保留。上游有 chooseTarget／chooseCard／chooseControl／chooseBool／chooseButton 或 direct／可选发动的，经 HostBehavior＋HostStage＋ChoiceKind 通道向玩家提问；AI 座位只用公开信息，按上游 ai 函数的取向作答。一个技能一段实现，只有上游同一实现的别名才可共用并注释；跨包技能绑定引擎里已有的等价实现，没有才新写。改名、内联、换函数名、加注释不算改写。
- 武将条目：静态字段（性别、势力、体力、技能表）与上游机器核对一致，且其全部技能已复核等价后，才可升为已复核等价。
- 模式/节点条目：流程、胜负判定、关卡奖励与上游一致，经真实对局跑通，并测存档恢复。
- 测试：先写按上游语义会失败的测试，经真实对局事件路径触发，注释上游 file:line；至少一条正例、一条“条件不满足不发动”的反例；有选择的测“选 A 与选 B 结果不同”“取消则不发动”及挂起时的存档恢复。
- 复核：每批由未参与实现的 agent 复核，固定种子抽样不少于 10% 且不少于 10 项（小来源全查），每行写明核对了哪些条件与分支，不用模板句；至少 3 个交互样本在上游实际触发（tabletop-comparison-cdp.mjs 或 headless Chrome）；一项不一致整批退回修复后重审。
- 只有复核通过的项才在 inventory_status.json 标“已复核等价”，复核人只填复核 agent；改完用 scripts/recount_g0.py 校验计数。

六、工程约束：
- 保留单写 Rust 内核、不可变目录、显式结算/续行、内置 Rust＋第三方 rule-vm 路线；复用 PackageStore 与现有协议/登记工具，不另建包管理、registry 或状态台账。
- 新增或改动 HostBehavior 时，词表、tag、键名、触发元数据、选序（source_order）、KNOWN_PROMPTS 一并登记。
- 行数门禁：受维护源码/测试/DSL ≤2,000 行；Rust 非测试源码有效行 ≤1,000。按功能、来源包或项边界拆分并起有意义的名字，不压缩、不机械切块；拆文件后同步文档里的 file:line 引用。
- 分层：engine-native-kernel ← engine-native-content ← engine-native（门面）；check-native-layering.py 棘轮只降不升。
- 集成测试按分组编译：新增 tests/*.rs 必须登记到 tests/groups/<组>.rs。
- 规则/存档变化时核对版本门禁、完整状态、事件、RNG 与续行；golden 变化先查原因与语义，再决定是否重录。
- 版本号：超出已登记授权的规则、存档、ABI、WS、schema 变化或新 ADR，先提交精确边界并取得用户明确答复。
- 证据：原始日志、.ngsa、浏览器会话、CI 包、截图原图只放 docs/operations/evidence-raw/（已忽略）；入库每批一个 README 加必要的小 JSON，原件只记路径与 sha256；单批 ≤2 MB、单文件 ≤200 KB，evidence 下不放 .rs／.py／.js。同一事实只写一处，别处链接。
- Spine 净室边界（裁决 35）不变：不读、不抄官方运行库源码与上游打包的 spine 文件，不引入依赖 spine-core 的包，产品不附带动皮资源。
- Windows 检出可能是 CRLF：新测试读包资产时按 LF 归一化；新增前端代码不依赖 Node 专有类型。
- 禁止：为了让统计或关键字搜索好看而改名、内联、换写法；在注释、文档或证据里写没做的事（包括列一条命令实际跑另一条）；用断言文档内容或清单计数的测试充数；把上游 AI 取舍写成规则；让自动结算或 AI 读取未公开信息；凭空生成上游不存在的实体；省掉上游的条件、随机、判定；为了赶进度跳过复核或降低抽样；改台账状态栏口径来关闭条目；删除或就地改写此前带日期的记录与表格；改写 Git 历史或强推。
- 状态用语固定为四种：“已有原生落点”“简化近似”“已复核等价”“缺失”；只有“已复核等价”计入完成。
- 运行会改写已跟踪文件的脚本后，检查 git status 并还原无关改动；不删别人的工作树。

七、验证（每个代码批提交前都要跑，命令不加过滤参数；证据里贴的命令必须是实际执行的命令）：
- 必跑：python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'；python3 -B scripts/check-source-lines.py；python3 -B scripts/check-native-layering.py；cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native；独立 CARGO_TARGET_DIR 下 RUSTFLAGS="-D warnings" cargo check -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets；cargo check --target wasm32-unknown-unknown -p engine-native-wasm。
- 产品面：cargo test --offline --workspace --no-fail-fast；cargo run -p conformance；apps/web-client 的 pnpm test／typecheck／build:native／smoke:native；python3 -B scripts/check-native-wasm-cross-target.py；bash scripts/cross-target-hash.sh；cargo test -p desktop-client；涉及 UI／Worker／渲染的改动跑真实 headless Chrome 产品路径。
- 新增测试须在 CI 同版 rustc 下无 lint 失败。
- 收口：python3 -B scripts/preflight.py 全部通过（含证据体量检查）。首次失败与补跑分开记录；门禁为红时先修红，不提交下一批。

八、并行与协作：
- 主 agent 负责排期、合入、门禁、发布与记录。实现由子 agent 按来源、子包或上游文件并行，同时保持 3–4 条实现线。例如 U01、L11、U05 子包、L19/L20/L25/L27 各占一条，界面一条。每条线在独立 git worktree 与独立 CARGO_TARGET_DIR 中工作，开工前划清文件所有权。
- 共用文件由主 agent 统一合入，各线只交补丁或清单，避免冲突。共用文件包括：HostBehavior 词表、tag 号段、选序登记、KNOWN_PROMPTS、inventory_status.json、tests/groups、进度主稿与台账。
- 实现与复核必须是不同的 agent；复核者只看上游与原生的实际行为和源码，不读实现者的说明。复核和下一批实现可以同时进行。
- 发布按批串行：一批门禁全绿、发布、CI 登记之后，再合入下一批。
- 在授权范围内自主推进，不反复询问是否继续。遇到第三节第 3 条的事项，把问题一次写清（各选项的实际后果），然后继续其他线。

九、记录：
- 在进度主稿顶部新增带日期条目，不删除、不覆盖此前任何带日期记录；某批发布后另起一条带日期的发布记录，不改旧条目的“待提交／发布”标题。
- 每条不超过 15 行，写明：日期、HEAD 与是否 dirty、版本、实际命令、通过/失败/忽略、证据链接、队列做到第几项、本批修复的差异数、各来源“已复核等价 N／简化近似 N”的变化。
- 文档用简体中文。第三节完成标准全部达到之前，不写“闭环”“全部交付”“100% 对齐”这类总括语。未提交、未发布的工作不写成已完成。

最终报告（仅在满足停止条件时写）：完成标准逐条达成情况；本轮各批修复的差异清单与发布/CI 记录；37 来源计数前后对照与抽样复核表；仍待用户裁决或外部输入的事项，每项写明所需输入和给了之后的剩余步骤；界面并排截图；Spine 4.0 测试与用户验证状态；37 来源八项签署；真机与签名状态；第六阶段状态。现在先核对工作区，然后从第四节第 0 项开始。
```
