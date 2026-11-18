# 原生架构收敛完整执行提示词

将下面整段交给执行 agent。方案正文保存需求与验收条件，实施状态以进度主稿顶部扫描和剩余台账为准；本提示词只列工作项、先后与待决事项，不复制一套当前数字。2026-11-18 核对后重写，替换 2026-11-13 版。上一版的队列第 0 项已完成，第 1 项部分完成；实际运行比对把已复核等价从 851 撤回到 625，这是对的。本版有两处主要调整：先修已发现的规则差异，再跑新的比对；对证据的入库体量设上限。

```text
请在 /Users/ruyimg/Documents/Claude/sanguosha/noname-next 继续《原生架构收敛执行方案》剩余工作。剩余工作量仍然很大，不要求一轮做完：按第三节的队列顺序推进，做到哪里如实报告到哪里。只有队列清空且全部门槛通过才可以说“全部完成”。本版的重点是把时间花在修规则上，而不是写证据上。

一、先读（按顺序），并遵守 /Users/ruyimg/Documents/Claude/sanguosha/AGENTS.md：
1. docs/operations/evidence/2026-11-18-progress-scan/README.md（本轮核对发现，必须先读）
2. docs/operations/evidence/2026-11-14-queue1-runtime/remaining-scope.md 与 known-automatic-choice-gaps-01.json（上一轮实际运行发现的差异清单，本轮第 1 项的工作来源）
3. docs/operations/evidence/2026-11-12-step1-choice-channel-prototype/README.md（交互通道做法与迁移五步配方）
4. docs/architecture/native-progress.md 顶部「状态扫描（2026-11-18 12:31–13:40）」
5. docs/operations/evidence/2026-10-28-user-decisions/README.md 的裁决 34、35、37、38、40，及裁决 19、23、25、36
6. docs/migration/g0/inventory_status.json、docs/migration/upstream-extension-reconstruction.md、docs/operations/evidence/2026-10-19-source-signoff-32/README.md
7. docs/architecture/native-remaining-ledger.md、docs/architecture/native-rules-baseline.md 顶部、docs/architecture/native-convergence-plan.md
8. /Users/ruyimg/Documents/Claude/sanguosha/NONAME_NEXT_AGENT_SPEC.md（实施约束；位于 Git 仓库外）

二、开工前核对（2026-11-18 快照，开工时以实际为准）：
- main HEAD 为 2026-11-18 扫描提交（纯文档）；最后代码提交 `5c73d1d91`，已发布，CI 首跑 10/10。规则 333／NGSA 107／content schema 5／ABI 759／WS8。三 crate 2447/0/6，门禁为绿。
- 清单：已复核等价 625／简化近似 3,799。各来源计数以 inventory_status.json 为准。
- 已完成：队列第 0 项；第 1 项中 U06、L29、U07、L10、U08、L28 的实际运行比对与复核，`juxian`、`jiushi` 已修。
- 未完成：
  · 实际运行发现的规则差异只登记、未修，包括 U01 聚贤 J01–J05 与已重审差异、U06 猴子消费与写轮眼取消、L29 清正／酒诗／放逐剩余语义、U07 筹度／复仇清算／诱战及实体诱敌、L10 蛊惑质疑与响应／让节／党锢与常侍阶段自动触发、U08 慧眼主技与两个辅助、L28 补叙／整经／业炎，以及 known-automatic-choice-gaps-01.json 中的自动／固定选择（写轮眼第二步取消、五行盘强制选择、蛊惑质疑等）；
  · L27、L19、L20、L25 各 3 个候选（共 12 个）未实际运行，L22 自动钩子未处理；
  · 队列第 2 项起（L11 太虚幻境 934 项、U05 杀海拾遗 1,621 项、其余来源）、界面、Spine 验证、八项签署（0/37）、真机与签名、第六阶段未开始。
- 证据体量失控：上一轮 6 个提交约 1,800 行代码、约 1,480 万行证据。docs/operations/evidence/2026-11-14-queue1-runtime/ 约 560 MB、4,584 个文件（1,973 个 .ngsa、单个 26 MB 的浏览器会话日志），已推送远端；本地 .git 1.5 GB。已入库的文件是否改写历史清除，等用户决定，在此之前不要改写历史、不要强推，也不要再往这些目录里加文件。
- 工作区里可能还有上一轮留下的未提交记录（2026-11-14 04:05 的发布终态与续做记录）和约 113 MB 未跟踪文件（含 docs/operations/evidence/2026-11-14-queue1-next-sources/）。先看清楚再决定；不要把大文件提交进去。
- 无 Android 设备，物理 iPhone unavailable；签名证书与公证凭据未配置。8 个临时工作树（1 个 locked）原样保留。若发现其他 agent 正在写，先停下协调。

三、执行队列（按顺序；每完成一项就提交、更新清单与台账、发布，再做下一项）：
0. 证据存放改造（先做，单独提交，只改规则和工具，不删已入库文件）：
   · 在仓库里新建一个被 .gitignore 忽略的原始证据目录（如 docs/operations/evidence-raw/），以后的原始日志、.ngsa 存档、浏览器会话、CI 下载包、截图原图一律放这里，不入库；
   · 入库的证据只放摘要：每批一个 README（结论、实际命令、通过／失败数、对照表）和必要的小 JSON 计数表，原始文件只记路径与 sha256；
   · 设上限并加进 preflight 检查：每批新增入库证据合计不超过 2 MB，单个文件不超过 200 KB；evidence 目录下不得出现 .rs／.py／.js 等源码文件（脚本放 scripts/，测试放 tests/）；
   · CI 登记每次一行：提交 SHA、run id、结果，首跑失败另起一行；不再为每次发布写单独的证据目录。
1. 修复已发现的规则差异（第二节列出的全部条目，按来源分批）：
   · 每个差异：先写一条按上游语义会失败的测试（经真实对局事件路径，注释写上游 file:line），再改实现使其通过；
   · 自动／固定选择的条目，经既有 HostBehavior ＋ HostStage ＋ ChoiceKind 通道改为向玩家提问；确实无法表达、需要升版本号的，逐技能列出报用户；
   · 一个来源的差异全部修完后，由未参与实现的 agent 复核这些修复（读上游与原生源码，必要时用 tabletop-comparison-cdp.mjs 在上游实际触发），通过后才把对应条目在清单里改为「已复核等价」。
2. 完成队列 1 剩余：L27（bolan、reqimou、new_meibu）、L19（_huzhu、ganran2、wujinmoshiRE1）、L20（kuanshi、tspowei、wengua）、L25（wumou、new_guixin、gongxin）在上游实际运行比对；L22 自动钩子按上游实现。发现差异照第 1 项的做法修，而不是只登记。
3. 继续按来源改写规则，队列为：L11 太虚幻境 → U05 杀海拾遗 → L01 AI优化 → U02／L03 boss → U03／L04 cardpile → L15 手杀全扩 → L21 武皇争霸 → 其余含规则项的来源。L11、U05 体量大，按上游文件或子包分批（如 U05 的 9 个子包），每批独立提交、复核、发布。每个技能的完成标准：
   · 逐步对照上游 trigger／filter／content：触发时机、过滤条件、次数限制、限定技、判定、随机（走引擎 RNG）全部保留；
   · 上游有 chooseTarget／chooseCard／chooseControl／chooseBool／chooseButton 或 direct／可选发动的，经既有通道向玩家提问；AI 座位只用公开信息、按上游 ai 函数的取向作答；
   · 一个技能一段实现；只有上游是同一实现的别名才可共用，并在注释里写明；
   · 跨包技能绑定引擎里已有的等价实现，没有才新写；
   · 改名、内联、换函数名、加注释都不算改写。
4. 测试与复核（适用于第 1–3 项）：
   · 每个技能经真实对局事件路径触发，预期取自上游代码并注释 file:line；至少一条正例、一条「条件不满足不发动」的反例；有选择的技能测「选 A 与选 B 结果不同」「取消则不发动」和选择挂起时的存档恢复；
   · 每批由未参与实现的 agent 按固定种子抽样复核（不少于 10%，且不少于 10 项，小来源全查）；每行写明具体核对了哪些条件与分支，不得用通用模板句；其中至少 3 个有交互的样本在上游实际触发比对；有一项不一致则该批退回；
   · 只有复核通过的项可在 inventory_status.json 标「已复核等价」，复核人只填复核 agent。
5. 界面：先做十周年UI 的牌桌布局——按上游的环绕式座位、身份标记、中央提示区与按钮、手牌区在真实牌桌组件上实现，用现有对照脚本出并排截图，直到布局一致；再依次做手杀ui、标记补充、千幻聆音、皮肤切换、如真似幻，以及已完成来源里留作「简化近似」的 CSS／显示面。截图原图放原始证据目录，入库的只放缩略对照图。
6. Spine 4.0（裁决 35、37、38）：补齐每个特性一条测试；由未读过实现的 agent 做净室复核并留记录；请用户按指引用自有 4.0 动皮在本机对照并回传结果。
7. 需要先问用户的事项（把选项的实际后果写清楚，等答复期间继续不受阻的部分）：①需要升版本号才能表达的交互，逐技能列出；②资源取得方式超出裁决 38 的来源。
8. 37 来源整包八项签署：按扩展台账原八项逐来源复核；第①项以「已复核等价」占比为准，第⑧项列出该来源全部剩余差异。门禁为红或未发布时不上调。
9. 外部条件（未就绪时写明所需操作并继续其他项；对应台账行保持「部分」）：PA08 真机 → PA11 判定、NC12 真机温控与移动端目标平台；三平台安装包签名／公证。
10. 发布：每个代码批完成第五节门禁后按裁决 25 发布并登记 exact-head CI（按第 0 项的一行格式），不要攒到最后。
11. 第六阶段：上述门槛全部成立后，先报精确范围取得用户确认，再做原生产品切换并退出旧执行入口。PA09 已由裁决 36 关闭。

四、工程约束：
- 保留单写 Rust 内核、不可变目录、显式结算/续行、内置 Rust＋第三方 rule-vm 路线；复用 PackageStore 与现有协议/登记工具，不并建第二套包管理、registry 或状态台账。
- 新增或改动 HostBehavior 时，词表、tag、键名、触发元数据、选序（source_order）、KNOWN_PROMPTS 一并登记。
- 行数门禁：受维护源码/测试/DSL ≤2,000 行；Rust 非测试源码有效行 ≤1,000。按功能、来源包或项边界拆分并起有意义的名字，不压缩、不机械切块。拆文件后同步文档里的 file:line 引用。
- 分层：engine-native-kernel ← engine-native-content ← engine-native（门面）；check-native-layering.py 棘轮只降不升。
- 集成测试按分组编译：新增 tests/*.rs 必须登记到 tests/groups/<组>.rs。
- 规则/存档变化核对版本门禁、完整状态、事件、RNG 与续行；golden 变化先查原因再决定重录。
- 证据：遵守第三节第 0 项的存放规则与体量上限。证据的作用是让别人能复查结论，不是记录每一步过程；同一事实只写一处，别处链接。
- 禁止：为了让统计数字或关键字搜索变好看而改名、内联、换写法；在注释、文档或证据里写没有做的事（包括列出一条命令却实际跑了另一条）；用断言文档内容或清单计数的测试充数；把上游 AI 的取舍写成规则；让自动结算或 AI 读取未公开信息；凭空生成上游不存在的实体；省掉上游的条件、随机、判定；改台账状态栏的口径来关闭条目；删除或就地改写他人或自己此前带日期的记录与表格；改写 Git 历史或强推（除非用户另有决定）。
- 状态用语固定为四种并分开写：「已有原生落点」「简化近似」「已复核等价」「缺失」。只有「已复核等价」计入第①项。
- 版本号：超出已登记授权的规则、存档、ABI、WS、schema 变化或新 ADR，先提交精确边界并取得用户明确答复。
- Spine 净室边界（裁决 35）不变：不读、不抄官方运行库源码与上游打包的 spine 文件，不引入依赖 spine-core 的包，产品不附带动皮资源。
- Windows 检出可能是 CRLF：新测试读包资产时按 LF 归一化；新增前端代码不依赖 Node 专有类型。
- 编译后检查磁盘并清理本批不再需要的私有缓存（裁决 23）；不清共享 target，不删别人的工作树。运行会改写已跟踪文件的脚本后，检查 git status 并还原无关改动。

五、验证（每个代码批提交前都要跑，命令不得加过滤参数；证据里贴的命令必须就是实际执行的命令）：
- 必跑：python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'；python3 -B scripts/check-source-lines.py；cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native；独立 CARGO_TARGET_DIR 下 RUSTFLAGS="-D warnings" cargo check -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets；cargo check --target wasm32-unknown-unknown -p engine-native-wasm。
- 产品面：cargo test --offline --workspace --no-fail-fast；cargo run -p conformance；apps/web-client 的 pnpm test／typecheck／build:native／smoke:native；python3 -B scripts/check-native-wasm-cross-target.py；bash scripts/cross-target-hash.sh；cargo test -p desktop-client；涉及 UI／Worker／渲染的改动跑真实 headless Chrome 产品路径。
- 新增测试须在 CI 同版 rustc 下无 lint 失败。
- 收口：python3 -B scripts/preflight.py 全部通过（含第 0 项新增的证据体量检查）。首次失败与补跑分开记录。门禁为红时不得提交下一批。

六、协作：可用子 agent 并行处理互不冲突的来源或子包，先划清文件所有权和接口（词表、tag 号段、选序登记这类共用文件由主 agent 统一合入），主 agent 负责整合与最终验收。实现与复核必须是不同的 agent；复核者只看上游与原生的实际行为和源码，不读实现者的说明。公共文档写入前重读。在授权范围内自主推进，不反复询问是否继续；缺少设备、凭据、权限、资源或产品语义时，写明所需输入与未验收门槛，继续做不受阻的部分。

七、记录：当前状态在进度主稿顶部**新增**带日期条目，不得删除或覆盖他人（含 Claude Code 核对）或自己此前的带日期记录；表格需要更新时新增一张带日期的表，旧表标「历史快照」保留。每条进度记录不超过 15 行，写明日期、HEAD 与是否 dirty、规则/存档/ABI/WS 版本、实际命令、通过/失败/忽略、证据链接，以及：队列做到第几项、本批修复的差异数、各来源「已复核等价 N／简化近似 N」的变化。文档统一用简体中文。不写「闭环」「全部交付」「100% 对齐」这类总括语，除非第三节全部完成。未提交、未发布的工作不得写成已完成；一轮结束前把自己的记录提交，不要留在工作区。

最终逐项报告：队列进度、本轮修复的差异清单、门禁与发布／CI 状态、各来源计数与抽样对照表、仍为自动结算或需升号的技能清单及原因、界面并排截图、Spine 4.0 测试与用户验证状态、37 来源八项签署、真机与签名状态、第六阶段，以及未完成项和所需外部条件。现在先核对工作区，然后从第三节第 0 项开始。
```
