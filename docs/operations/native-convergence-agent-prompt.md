# 原生架构收敛完整执行提示词

将下面整段交给执行 agent。方案正文保存需求与验收条件，实施状态以进度主稿顶部扫描和剩余台账为准；本提示词只列工作项、先后与待决事项，不复制一套当前数字。2026-11-13 核对后重写，替换 2026-11-11 夜间版。上一版的队列第 0、1 项和第 2 项的 13 个来源已有实质交付，做法继续沿用；当前要先修红门禁并补发布（详见下文第二节）。

```text
请在 /Users/ruyimg/Documents/Claude/sanguosha/noname-next 继续《原生架构收敛执行方案》剩余工作。剩余工作量仍然很大（L11 太虚幻境 934 项、U05 杀海拾遗 1,621 项，另有界面与真机项），不要求一轮做完：按第三节的队列顺序推进，做到哪里如实报告到哪里。只有队列清空且全部门槛通过才可以说“全部完成”。

一、先读（按顺序），并遵守 /Users/ruyimg/Documents/Claude/sanguosha/AGENTS.md：
1. docs/operations/evidence/2026-11-13-progress-scan/README.md（本轮核对发现，必须先读）
2. docs/operations/evidence/2026-11-12-step1-choice-channel-prototype/README.md（交互通道做法与迁移五步配方，后续技能照此做）
3. docs/architecture/native-progress.md 顶部「状态扫描（2026-11-13 19:59–21:10）」
4. docs/operations/evidence/2026-10-28-user-decisions/README.md 的裁决 34、35、37、38、40，及裁决 19、23、25、36
5. docs/migration/upstream-extension-reconstruction.md（顶部勘正、八项定义）、docs/operations/evidence/2026-10-19-source-signoff-32/README.md、docs/migration/g0/README.md 与 inventory_status.json
6. docs/architecture/native-remaining-ledger.md、docs/architecture/native-rules-baseline.md 顶部、docs/architecture/native-convergence-plan.md、docs/architecture/native-extensions.md
7. /Users/ruyimg/Documents/Claude/sanguosha/NONAME_NEXT_AGENT_SPEC.md（实施约束；位于 Git 仓库外）

二、开工前核对（2026-11-13 快照，开工时以实际为准）：
- main HEAD 为 2026-11-13 扫描提交（纯文档）；最后代码提交 `afa72d46b`。远端 native-convergence 仍是 `99d9091bb`：其后的 15 个代码提交（`c9cf9c7de` 至 `afa72d46b`）全部未发布、无 CI。规则 333／NGSA 107／content schema 5／ABI 759／WS8。
- 三 crate 门禁为红：2420 passed／2 failed／6 ignored。失败的是 c331_source_order_phase5::phase5_all44_source_tuples_match_the_current_native_bindings 与 c331_source33_integration::phase7_all_trigger_bearing_host_behaviors_have_bound_source_order。原因是 `87b64c732`（L20）让 HostBehavior::FzLiegong 带触发却没登记选序（source_order）。其后 U08、L25 两批是在红门禁下提交的——证据里写的是完整三 crate 命令，实际跑的是加了 --lib 的版本。
- 全工作区测试 7815 passed／3 failed／8 ignored：除上述两个外，game-server 的 native_character_only_installed_package_enters_match（apps/game-server/tests/native_product_path.rs:306）失败——U01 的 64 个技能已全部迁为原生 host 技能，测试仍断言 d3-select 包是 2 个 host 技能加 62 个 VM 技能，`5b265bbce`（U01）之后未同步。
- 已完成并可沿用：
  · 交互通道：原生 HostBehavior ＋ HostStage ＋ ChoiceKind，不升版本号即可向玩家提问并支持选择挂起中的存档恢复；
  · 13 个来源（U06、U07、L06、L22、L29、L10、U01、L28、L27、L19、L20、U08、L25）的规则项已迁到 crates/engine-native-content/src/content/host/step/ 下逐技能实现；
  · 清单为已复核等价 851／简化近似 3,573。
- 抽查结论（固定种子抽 12、细读 6）：hanbei、biaoqi、tspowei、th_guili、dinghan 与上游一致；dddanzhi 有两处偏差——触发条件多了「或弃牌堆非空」，所选目标不是自己时自己也获得免疫。
- 复核的问题：13 份来源复核全部是读源码比对，没有按要求在上游无名杀里实际触发；U01 复核表有 72 行是同一句模板话，dddanzhi 的偏差就漏在其中。
- 未做：L11、U05 及其余含规则项的来源（L01、U02／L03、U03／L04、L15、L21 等）；界面；Spine 逐特性测试、净室复核与用户本机对照；八项签署（0/37）；真机与签名；第六阶段。
- 无 Android 设备，物理 iPhone unavailable；签名证书与公证凭据未配置。8 个临时工作树（1 个 locked）原样保留。若发现其他 agent 正在写，先停下协调。

三、执行队列（按顺序；每完成一项就提交、更新清单与台账，再做下一项）：
0. 修门禁并补发布（先做，单独提交）：
   · 给 FzLiegong 补齐选序登记，使上述两个测试通过；按 U01 迁移后的实际构成更新 game-server 那条过期断言（先确认 64 个原生 host 技能、0 个 VM 技能是预期结果，而不是装载出了问题）；确认 14 场 golden 哈希与既有存档不受影响，若选序变化会改变既有对局轨迹，先报边界；
   · 完整跑第五节全部门禁（三 crate 命令不得加 --lib 或其他过滤），把完整输出落到证据目录；
   · python3 -B scripts/check_doc_line_refs.py 报告 23 处行号越界，都在 2026-11-12 的 U06、U07、L10、L27 复核证据里（所引文件后来被拆分）。把这些引用改到现在的文件与行号，不改证据的其他内容；
   · 修 dddanzhi：触发条件只看「上一张被使用或打出的牌为黑色」（含打出）；只有所选目标获得免疫；补对应反例测试；
   · 按裁决 25 把未发布的提交推到 native-convergence，登记 exact-head CI；首次失败与补跑分开记录。
1. 复核补课（针对已完成的 13 个来源，单独提交）：
   · U01 复核表中的模板行逐行重审，写出每个技能具体核对了什么（触发时机、每个条件、每个选择分支、结算步骤）；发现偏差就修并补测试；
   · 每个来源挑 3 个有交互的技能（不足 3 个则全取），用 apps/web-client/scripts/tabletop-comparison-cdp.mjs 的启动方式在 headless Chrome 的上游无名杀里实际触发，记录结果，与原生同场景结果比对，对照表入证据。上游里确实跑不起来的，写明原因并改为逐行比对；
   · 界面钩子、配置类条目若只是「无规则效果／惰性登记」，在清单的差别栏写明这一点，不要与技能的等价混为一谈。
2. 继续按来源改写规则，队列为：L11 太虚幻境 → U05 杀海拾遗 → L01 AI优化 → U02／L03 boss → U03／L04 cardpile → L15 手杀全扩 → L21 武皇争霸 → 其余含规则项的来源。L11、U05 体量大，可按上游文件或子包分批（如 U05 的 9 个子包），每批独立提交、独立复核。每个技能的完成标准不变：
   · 逐步对照上游 trigger／filter／content：触发时机、过滤条件、次数限制、限定技、判定、随机（走引擎 RNG）全部保留；
   · 上游有 chooseTarget／chooseCard／chooseControl／chooseBool／chooseButton 或 direct／可选发动的，经既有通道向玩家提问；AI 座位只用公开信息、按上游 ai 函数的取向作答；
   · 一个技能一段实现；只有上游是同一实现的别名才可共用，并在注释里写明；
   · 跨包技能绑定引擎里已有的等价实现，没有才新写；
   · 改名、内联、换函数名、加注释都不算改写。
3. 每个技能的测试：经真实对局事件路径触发，预期取自上游代码并在注释写 file:line；至少一条正例、一条「条件不满足不发动」的反例；有选择的技能测「选 A 与选 B 结果不同」「取消则不发动」和选择挂起时的存档恢复。
4. 独立复核（实现者不得参与）：每个来源或每批做完后，复核 agent 用固定种子抽样（不少于 10%，且不少于 10 项，小来源全查）。每个样本的复核行必须写明具体核对的条件与分支，不得使用通用模板句；其中至少 3 个有交互的样本按第 1 项的方法在上游实际触发比对。有一项不一致则该批退回。只有复核通过的项可在 inventory_status.json 标「已复核等价」，复核人一栏只填复核 agent。
5. 界面：先做十周年UI 的牌桌布局——按上游的环绕式座位、身份标记、中央提示区与按钮、手牌区在真实牌桌组件上实现，用现有对照脚本出并排截图，直到布局一致；再依次做手杀ui、标记补充、千幻聆音、皮肤切换、如真似幻，以及已完成来源里留作「简化近似」的 CSS／显示面。每个界面钩子在清单写明原生组件与可见效果，对照通过才从「示意（待对照）」改为「已对照」。
6. Spine 4.0（裁决 35、37、38）：补齐每个特性一条测试；由未读过实现的 agent 做净室复核并留记录；请用户按指引用自有 4.0 动皮在本机对照并回传结果。
7. 需要先问用户的事项（把选项的实际后果写清楚，等答复期间继续不受阻的部分）：①经既有通道确实无法表达、需要升版本号的交互，逐技能列出；②资源取得方式超出裁决 38 的来源；③第 0 项若发现选序修复会改变既有轨迹。
8. 37 来源整包八项签署：按扩展台账原八项逐来源复核；第①项以「已复核等价」占比为准，第⑧项列出该来源全部剩余差异。门禁为红或未发布时不上调。
9. 外部条件（未就绪时写明所需操作并继续其他项；对应台账行保持「部分」）：PA08 真机 → PA11 判定、NC12 真机温控与移动端目标平台；三平台安装包签名／公证。
10. 发布：每个代码批完成第五节门禁后按裁决 25 发布并独立登记 exact-head CI，不要攒到最后。
11. 第六阶段：上述门槛全部成立后，先报精确范围取得用户确认，再做原生产品切换并退出旧执行入口。PA09 已由裁决 36 关闭。

四、工程约束：
- 保留单写 Rust 内核、不可变目录、显式结算/续行、内置 Rust＋第三方 rule-vm 路线；复用 PackageStore 与现有协议/登记工具，不并建第二套包管理、registry 或状态台账。
- 新增 HostBehavior 时，词表、tag、键名、触发元数据、选序（source_order）、KNOWN_PROMPTS 一并登记；改动既有 HostBehavior 的触发元数据同样要同步选序。
- 行数门禁：受维护源码/测试/DSL ≤2,000 行；Rust 非测试源码有效行 ≤1,000。按功能、来源包或项边界拆分并起有意义的名字，不压缩、不机械切块。拆文件后同步文档里的 file:line 引用。
- 分层：engine-native-kernel ← engine-native-content ← engine-native（门面）；check-native-layering.py 棘轮只降不升。
- 集成测试按分组编译：新增 tests/*.rs 必须登记到 tests/groups/<组>.rs。
- 规则/存档变化核对版本门禁、完整状态、事件、RNG 与续行；golden 变化先查原因再决定重录。
- 禁止：为了让统计数字或关键字搜索变好看而改名、内联、换写法；在注释、文档或证据里写没有做的事（包括列出一条命令却实际跑了另一条）；用断言文档内容或清单计数的测试充数；把上游 AI 的取舍写成规则；让自动结算或 AI 读取未公开信息；凭空生成上游不存在的实体；省掉上游的条件、随机、判定；改台账状态栏的口径来关闭条目；删除或就地改写他人或自己此前带日期的记录与表格。
- 状态用语固定为四种并分开写：「已有原生落点」「简化近似」「已复核等价」「缺失」。只有「已复核等价」计入第①项。
- 版本号：超出已登记授权的规则、存档、ABI、WS、schema 变化或新 ADR，先提交精确边界并取得用户明确答复。
- Spine 净室边界（裁决 35）不变：不读、不抄官方运行库源码与上游打包的 spine 文件，不引入依赖 spine-core 的包，产品不附带动皮资源。
- Windows 检出可能是 CRLF：新测试读包资产时按 LF 归一化；新增前端代码不依赖 Node 专有类型。
- 编译后检查磁盘并清理本批不再需要的私有缓存（裁决 23）；不清共享 target，不删别人的工作树。运行会改写已跟踪文件的脚本后，检查 git status 并还原无关改动。

五、验证（每个代码批提交前都要跑，命令不得加过滤参数；证据里贴的命令必须就是实际执行的命令）：
- 必跑：python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'；python3 -B scripts/check-source-lines.py；cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native；独立 CARGO_TARGET_DIR 下 RUSTFLAGS="-D warnings" cargo check -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets；cargo check --target wasm32-unknown-unknown -p engine-native-wasm。
- 产品面：cargo test --offline --workspace --no-fail-fast；cargo run -p conformance；apps/web-client 的 pnpm test／typecheck／build:native／smoke:native；python3 -B scripts/check-native-wasm-cross-target.py；bash scripts/cross-target-hash.sh；cargo test -p desktop-client；涉及 UI／Worker／渲染的改动跑真实 headless Chrome 产品路径。
- 新增测试须在 CI 同版 rustc（1.99.x）下无 lint 失败。
- 收口：python3 -B scripts/preflight.py 全部通过。首次失败与补跑分开记录。门禁为红时不得提交下一批。

六、协作：可用子 agent 并行处理互不冲突的来源或子包，先划清文件所有权和接口（词表、tag 号段、选序登记这类共用文件由主 agent 统一合入），主 agent 负责整合与最终验收。实现与复核必须是不同的 agent；复核者只看上游与原生的实际行为和源码，不读实现者的说明。公共文档写入前重读。在授权范围内自主推进，不反复询问是否继续；缺少设备、凭据、权限、资源或产品语义时，写明所需输入与未验收门槛，继续做不受阻的部分。

七、记录：当前状态在进度主稿顶部**新增**带日期条目，不得删除或覆盖他人（含 Claude Code 核对）或自己此前的带日期记录；表格需要更新时新增一张带日期的表，旧表标「历史快照」保留。原始命令与输出写入有日期的 evidence 目录。文档统一用简体中文，每条记录写明日期、HEAD 与是否 dirty、规则/存档/ABI/WS 版本、命令、通过/失败/忽略和证据链接。进度只报：队列做到第几项、各来源「已复核等价 N／简化近似 N／缺失 N」、本批抽样对照表链接。不写「闭环」「全部交付」「100% 对齐」这类总括语，除非第三节全部完成。未提交、未发布的工作不得写成已完成。

最终逐项报告：队列进度、门禁与发布／CI 状态、各来源计数与抽样对照表、上游实际触发比对的样本、仍为自动结算或需升号的技能清单及原因、界面并排截图、Spine 4.0 测试与用户验证状态、37 来源八项签署、真机与签名状态、第六阶段，以及未完成项和所需外部条件。现在先核对工作区，然后从第三节第 0 项开始。
```
