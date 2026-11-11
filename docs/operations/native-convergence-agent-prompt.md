# 原生架构收敛完整执行提示词

将下面整段交给执行 agent。方案正文保存需求与验收条件，实施状态以进度主稿顶部扫描和剩余台账为准；本提示词只列工作项、先后与待决事项，不复制一套当前数字。2026-11-11 核对后重写，替换 2026-11-09 版。上一版的清点、清除自造内容、接入正式包、净室 Spine 4.0 播放器已有交付；「按上游重做」经抽查只达到简化近似，清单里的「忠实实现 4,424／0／0」是生成脚本写死的，不是比对结果（详见下文第二节）。

```text
请在 /Users/ruyimg/Documents/Claude/sanguosha/noname-next 继续完成《原生架构收敛执行方案》剩余工作：实现、集成并验收，清空剩余台账、扩展台账和规则基准里仍未达成的完成条件；不是再写建议、改口径或只跑代表样本；不以时间成本压缩范围。

一、先读（按顺序），并遵守 /Users/ruyimg/Documents/Claude/sanguosha/AGENTS.md：
1. docs/operations/evidence/2026-11-11-progress-scan/README.md（本轮核对发现与 9 个抽查对照，必须先读）
2. docs/architecture/native-progress.md 顶部「状态扫描（2026-11-11 14:05–15:10）」
3. docs/operations/evidence/2026-10-28-user-decisions/README.md 的裁决 33–40（重点 34、35、40）及裁决 19、23、25
4. docs/migration/upstream-extension-reconstruction.md（顶部 2026-11-11 勘正、八项定义）、docs/operations/evidence/2026-10-19-source-signoff-32/README.md、docs/migration/g0/README.md
5. docs/architecture/native-remaining-ledger.md（37 来源、PA08、PA11、NC12 各行）与 docs/architecture/native-rules-baseline.md 顶部
6. docs/architecture/native-convergence-plan.md（六阶段门槛）、docs/architecture/native-extensions.md
7. /Users/ruyimg/Documents/Claude/sanguosha/NONAME_NEXT_AGENT_SPEC.md（实施约束；位于 Git 仓库外）

二、开工前核对（2026-11-11 快照，开工时以实际为准）：
- main HEAD 为 2026-11-11 扫描提交（纯文档）；最后代码提交 `99d9091bb` 已在远端 native-convergence。规则 333／NGSA 107／content schema 5／ABI 759／WS8。三 crate 2229/0/6（6 个 ignore 全是手工探针）。
- 已有且可沿用：docs/migration/g0/ 的 37 份逐来源清单（4,424 项，带上游 file:line）；各来源经 attach_formal_package_upstream_rules 接入 packages/extensions/* 正式包；apps/web-client/src/engine/spine40/ 净室 Spine 4.0 播放器；上游不存在的实体已清除。
- 未达成（抽查结论）：
  · crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34*.rs 的技能体多为简化近似：玩家选择被换成 first_other_alive（座次上第一个存活的其他角色，185 处）或第一张手牌，向玩家发起选择 0 处；条件、随机、判定、可选发动丢失；多个技能共用一个分支；U05 有 807 处单行 u05_exec 模板（141 处只是摸 N 张）。例：雷击变成「每用一张牌就对下家造成 2 点雷电伤害」，不屈不查「创」点数重复，猎奇、科考变成摸 1 张。
  · 测试用 debug_commit_effect 直接分派并断言上述原生行为，不走真实触发路径，也不是上游语义。
  · scripts/legacy-harness/decision34_inventory_builder.py 把「忠实实现」写死，重新生成恒为 4,424／0／0。
  · 界面：上游参考截图只是启动欢迎页；原生面是文字状态面板与按钮，不是十周年UI／手杀ui 等的牌桌布局。
  · Spine 4.0：二进制 .skel 只做过自编自解往返，未用真实动皮验证；逐特性测试、净室复核、用户本机截图对照未完成。
- 内核已有不升版本号的玩家选择通道：原生 HostBehavior ＋ HostStage::TargetChoice／SelectCard ＋ ChoiceKind::SelectPlayers／SelectCards／Confirm／按牌名选择（crates/engine-native-content/src/content/host/asks.rs、crates/engine-native-kernel/src/choice.rs），苍螭之璧即经此实现并支持挂起恢复。
- 裁决 40（2026-11-11）已撤销裁决 39 的自动结算部分：上游有玩家选择的技能必须经上述既有通道向玩家提问并支持挂起恢复，AI 座位按上游 ai 函数的取向决策；不升版本号的部分保留。确实做不到的逐技能列出原因，不得笼统登记。
- 无 Android 设备，物理 iPhone unavailable；签名证书与公证凭据未配置。8 个临时工作树（1 个 locked）原样保留。若发现其他 agent 正在写，先停下协调。

三、执行顺序（各项完成条件以台账对应行为准，完成后在台账行内更新）：
1. 先修清单工具（单独提交）：decision34_inventory_builder.py 不得再写死状态。状态改为逐项记录并只允许四种：「已复核等价」（须填复核人、日期、所对照的上游 file:line 与测试名）、「简化近似」（须写明与上游的差别）、「缺失」、「上游未定义」。现有全部规则项先统一降为「简化近似（待复核）」，界面项降为「示意（待对照）」；复核结果存为受版本管理的数据文件，由脚本读取，不由脚本推断。手工加在 g0/README.md 顶部的 2026-11-11 勘正段在重新生成时保留。
2. 逐来源、逐技能按上游改写规则（按来源分批提交）。每个技能逐步对照上游 content／filter／trigger：
   · 触发时机与过滤条件按上游（不是 7 个粗粒度窗口加「其余按阶段开始」）；
   · 上游有 chooseTarget／chooseCard／chooseControl／chooseBool 的，经既有选择通道向玩家提问，AI 座位按上游 ai 函数的取向决策；
   · 条件、判定、随机（走引擎 RNG 流）、每回合／每轮次数、限定技、可选发动逐项保留；
   · 禁止多个技能共用一个分支，除非上游就是同一实现的别名，并在注释里写明；
   · 上游来自标准包或其他已迁移包的跨包技能（如雷击、不屈），优先绑定引擎里已有的等价实现，没有才新写；
   · u05_exec 这类参数模板只可用于上游确实只有「摸／伤／回复」单步效果的技能，其余改为逐技能实现。
3. 测试按上游语义写：每个技能至少一条经真实对局事件路径触发的测试（不是 debug_commit_effect 直接分派），预期取自上游代码并在注释写 file:line，覆盖至少一个「条件不满足不发动」的反例；有选择的技能测玩家应答与挂起恢复。旧的断言原生自造行为的测试按上游语义改写，改写理由逐条写入证据。
4. 独立抽样复核：每个来源完成后，由未参与该来源实现的 agent 用固定种子从清单随机抽样（不少于该来源规则项的 10%，且不少于 10 项，小来源全查），逐项读上游与原生源码填对照表（上游行为／原生行为／是否一致），写入证据；有一项不一致则该来源退回。复核通过的项才可在清单标「已复核等价」。
5. 界面类（十周年UI、手杀ui、千幻聆音、皮肤切换、如真似幻、标记补充等的布局、标记、动画、特效）：在真实牌桌组件上实现上游的布局与交互，不是状态文字面板。对照方法：在 headless Chrome 里实际运行 archive/upstream 的无名杀并启用对应扩展，进入同一对局场景截图，与原生同场景截图并排存入证据；逐个界面钩子在清单里写明原生对应的组件与可见效果。帧率在真实牌桌场景测。
6. Spine 4.0（裁决 35、37、38）：补齐「每个特性一条测试」的清单；由未读过实现的 agent 做净室复核并留记录；准备一页用户操作说明（如何导入自有 4.0 动皮、在哪里截图、与旧版无名杀怎么对照），请用户在本机执行并回传结果。二进制解码在用户真实资源验证前标「未经真实资源验证」。
7. 需要先问用户的事项（报精确边界后等答复，期间继续不受阻的部分）：①经既有通道仍无法表达、确需新增 HostStage、ABI 操作码或存档字段的交互（裁决 40 第③点），逐技能列出原因与所需版本号变化，答复前这些技能标「简化近似」；②体量大且以数据为主的来源的资源取得方式若超出裁决 38。
8. 37 来源整包八项签署：第 1–6 项完成后，按扩展台账原八项逐来源复核并更新签署表；第①项以清单「已复核等价」占比为准，第⑧项须列出该来源全部剩余差异。移动端目标平台待第 9 项。
9. 外部条件（未就绪时写明所需操作并继续其他项）：PA08 真机（中低端 Android 开启 USB 调试、iPhone 解锁并信任）→ 整局与 ≥30 分钟功耗／温度／降频，据此判定 PA11，并补 NC12 真机温控与移动端目标平台；三平台安装包签名／公证（Apple Developer ID 证书与公证凭据、Windows Authenticode 证书）→ 真实安装验证。
10. 发布：每个代码批完成第五节门禁后按裁决 25 发布并独立登记 exact-head CI；首次失败与补跑分开记录。
11. 第六阶段：上述门槛全部成立后，先报精确范围取得用户确认，再完成原生产品切换并安全退出旧执行入口（engine-core／engine-ai／engine-wasm），保留上游来源、正式 ADR、原始证据与仍需的迁移／恢复能力。PA09 已由裁决 36 关闭：维持 WASM 默认，保留 ?desktopEngine=native 开关及其测试。

四、工程约束：
- 保留单写 Rust 内核、不可变目录、显式结算/续行、内置 Rust＋第三方 rule-vm 路线；复用 PackageStore 与现有协议/登记工具，不并建第二套包管理、registry 或状态台账；不自动立项 ECS、QuickJS、全面 VM 化、强化学习等历史候选。
- 行数门禁：受维护源码/测试/DSL ≤2,000 行；Rust 非测试源码有效行 ≤1,000。按功能、来源包或项边界拆分并起有意义的名字，不压缩、不机械切块；抽函数时核对 return／?／Ok 控制流。拆文件后同步文档里的 file:line 引用。
- 分层：engine-native-kernel ← engine-native-content ← engine-native（门面）；check-native-layering.py 棘轮只降不升，内核需要内容行为时加 Catalog 回调。
- 集成测试按分组编译：新增 tests/*.rs 必须登记到 tests/groups/<组>.rs。
- 规则/存档变化核对版本门禁、完整状态、事件、RNG 与续行；golden 变化先查原因再决定重录。
- 禁止：凭空生成上游不存在的实体；用模板、分组分支或自动选目标冒充逐技能实现；把上游的条件、随机、判定省掉；用只断言原生行为的测试证明等价；用启动页或状态面板冒充同场景对照；在脚本或文档里写死完成状态；删除或改写他人带日期的记录（上一轮 `ff82883a7` 删了签署表的勘正段，已恢复）。
- 不改验收口径来宣告完成：完成条件以台账与扩展台账原文为准。「已有原生落点」「已接入正式包」「简化近似」「已复核等价」四种状态分开写，只有最后一种可计入第①项。
- 版本号：同一号码只被一批占用；超出已登记授权的规则、存档、ABI、WS、schema 变化或新 ADR，先提交精确边界并取得用户明确答复。向用户请示时把选项的实际后果写清楚（例如「玩家将不能自选目标」），不要只写「按推荐进行」。
- Spine 净室边界（裁决 35）不变：不读、不抄官方运行库源码与上游打包的 spine 文件，不引入依赖 spine-core 的包，产品不附带动皮资源。
- Windows 检出可能是 CRLF：新测试读包资产时按 LF 归一化；新增前端代码不依赖 Node 专有类型（如 Buffer）。
- 编译后检查磁盘并清理本批不再需要的私有缓存（裁决 23）；不清共享 target，不删别人的工作树。

五、验证（按影响面）：
- 必跑：python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'；python3 -B scripts/check-source-lines.py；cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native；独立 CARGO_TARGET_DIR 下 RUSTFLAGS="-D warnings" cargo check -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets；cargo check --target wasm32-unknown-unknown -p engine-native-wasm。
- 产品面：cargo test --offline --workspace --no-fail-fast；cargo run -p conformance；apps/web-client 的 pnpm test／typecheck／build:native／smoke:native；python3 -B scripts/check-native-wasm-cross-target.py；bash scripts/cross-target-hash.sh；cargo test -p desktop-client；涉及 UI／Worker／渲染的改动跑真实 headless Chrome 产品路径。
- 新增测试须在 CI 同版 rustc（1.99.x）下无 lint 失败。
- 收口：python3 -B scripts/preflight.py 全部通过。首次失败与补跑分开记录，不以补跑抹去失败。

六、协作：可用子 agent 并行处理互不冲突的来源，先划清文件所有权和接口，主 agent 负责整合与最终验收。实现与复核必须是不同的 agent；复核者只读上游与原生源码，不读实现者的说明。公共文档写入前重读；不要同时抢写核心文件。在授权范围内自主推进，不反复询问是否继续；缺少设备、凭据、权限、资源或产品语义时，写明所需输入与未验收门槛，继续做不受阻的部分。

七、记录：当前状态在进度主稿顶部**新增**带日期条目，不得删除或覆盖他人（含 Claude Code 核对）或自己此前的带日期记录，过时条目只加「历史快照」标注；剩余台账、扩展台账、规则基准各管其职；原始命令与输出写入有日期的 evidence 目录。文档统一用简体中文，每条记录写明日期、HEAD 与是否 dirty、规则/存档/ABI/WS 版本、命令、通过/失败/忽略和证据链接。进度数字只报「已复核等价 N／简化近似 N／缺失 N」，并附该批抽样对照表链接。不得在提交前写“已合入／已提交”。

最终逐项报告：各来源「已复核等价／简化近似／缺失」计数与抽样对照表、仍为自动结算的技能清单及原因、界面同场景对照截图、Spine 4.0 逐特性测试与用户验证状态、37 来源八项签署、PA08／PA11／NC12 真机、安装包签名、发布与 CI、第六阶段，以及仍未完成／未验证项和所需外部条件。只有全部门槛真实通过才可声明“全部完成”；否则明确整体尚未完成。现在先核对工作区，然后从第三节第 1 项开始。
```
