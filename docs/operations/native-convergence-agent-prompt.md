# 原生架构收敛完整执行提示词

将下面整段交给执行 agent。方案正文保存需求与验收条件，实施状态以进度主稿顶部扫描和剩余台账为准；本提示词只列工作项、先后与待决事项，不复制一套当前数字。2026-11-11 夜间核对后重写，替换同日下午版。上一版交付了数据驱动的清单、如实的签署表和真实的同场景截图对照；但逐技能改写只做了改名和模板内联，裁决 40 的真实玩家选择没有实现（详见下文第二节）。

```text
请在 /Users/ruyimg/Documents/Claude/sanguosha/noname-next 继续《原生架构收敛执行方案》剩余工作。剩余工作量很大（约 1,900 个扩展技能要按上游逐个改写，另有界面与真机项），不要求一轮做完：按第三节的队列顺序推进，做到哪里如实报告到哪里。只有队列清空且全部门槛通过才可以说“全部完成”。

一、先读（按顺序），并遵守 /Users/ruyimg/Documents/Claude/sanguosha/AGENTS.md：
1. docs/operations/evidence/2026-11-11-progress-scan-night/README.md（本轮核对发现，必须先读）
2. docs/operations/evidence/2026-11-11-progress-scan/README.md（9 个技能的上游／原生对照表）
3. docs/architecture/native-progress.md 顶部「状态扫描（2026-11-11 22:53–23:50）」
4. docs/operations/evidence/2026-10-28-user-decisions/README.md 的裁决 34、35、37、38、40，及裁决 19、23、25、36
5. docs/migration/upstream-extension-reconstruction.md（顶部勘正、八项定义）、docs/operations/evidence/2026-10-19-source-signoff-32/README.md、docs/migration/g0/README.md 与 inventory_status.json
6. docs/architecture/native-remaining-ledger.md、docs/architecture/native-rules-baseline.md 顶部、docs/architecture/native-convergence-plan.md、docs/architecture/native-extensions.md
7. /Users/ruyimg/Documents/Claude/sanguosha/NONAME_NEXT_AGENT_SPEC.md（实施约束；位于 Git 仓库外）

二、开工前核对（2026-11-11 夜间快照，开工时以实际为准）：
- main HEAD 为夜间扫描提交之前的 `057ba1789`；其后的「Milestone 1–4」工作（94 个已跟踪文件改动加 tests/e2e/ 等未跟踪文件）可能仍未提交、未发布、无 CI。远端 native-convergence 为 `99d9091bb`。规则 333／NGSA 107／content schema 5／ABI 759／WS8。该工作区三 crate 2524/0/6。
- 可沿用：docs/migration/g0/inventory_status.json 与数据驱动的清单生成；apps/web-client/scripts/tabletop-comparison-cdp.mjs（能在 headless Chrome 里实际运行 archive/upstream 的无名杀并进入对局）；l11_taixu_boss_events.rs 的真实事件路径测试写法；Spine 4.0 播放器与用户指引。
- 未达成：
  · 逐技能改写基本没做。上一轮把 first_other_alive 改名为 hostile_target／friendly_target／filtered_target（314 处），把 807 处 u05_exec 模板原样内联（例：lieqi、kekao 仍是「摸 1 张」），把 debug_commit_effect 改名为 debug_execute_effect。行为没有变。清单自己的计数是已复核等价 4／简化近似 4,420。
  · 裁决 40 未实现：rvm_ops／rvm_bridge 中没有任何 open_choice 调用；choice_routing.rs 只读 ctx.res.answer（无人提问即无应答），然后按态度分自动挑人、按固定优先级自动挑牌，并直接读取身份局的真实身份。可选发动仍是必发。
  · 标为「已复核等价」的 4 项（dddxiaxing、boss_chuanyun、boss_lingfeng、boss_tianyun）同样缺玩家选择，复核人含实现者，不成立。
  · tests/e2e/ 的 290 条测试多为断言清单计数、文档字符串与通用存读档，不证明规则等价。
  · 同场景对照图显示：上游是环绕式牌桌，原生是一排横向座位卡片，布局不同。
  · Spine 二进制解码未经真实动皮验证。
- 根本障碍：decision34*.rs 的技能经 rule-vm StorageSet 分派到内核里同步执行，一个窗口内跑完，中途没有挂起点，所以问不了玩家。能问玩家的是原生 HostBehavior ＋ HostStage 通道（crates/engine-native-content/src/content/host/asks.rs、crates/engine-native-kernel/src/choice.rs），苍螭之璧即走这条路。有交互的技能需要迁到这条通道上，而不是在同步分派里换一个挑人函数。
- 无 Android 设备，物理 iPhone unavailable；签名证书与公证凭据未配置。8 个临时工作树（1 个 locked）原样保留。若发现其他 agent 正在写，先停下协调。

三、执行队列（按顺序；每完成一项就提交、更新清单与台账，再做下一项）：
0. 收拾工作区（单独提交）：
   · 逐文件看过未提交改动再提交，不整包盲提；
   · inventory_status.json 中那 4 项降回「简化近似」，差别栏写明「缺玩家选择」等；
   · tests/e2e/ 中断言清单计数或文档字符串的测试删除或改为不绑定具体数字（清单如实更新不应导致测试失败）；测试名与内容不符的（如名为 choice_suspend 实为通用存读档）改名；
   · apps/web-client/dist/ 下被构建改动的已跟踪文件：恢复，或正式移出版本管理，二选一并写明；
   · docs/operations/evidence/ 根目录的 tabletop-*.png、tabletop-fps-report.json、spine40-user-guide.md 移入带日期的证据目录并修正链接，删除 apps/web-client/evidence/ 下的重复件；
   · choice_routing.rs 的注释改为与代码一致；get_upstream_attitude 不得读取未公开的身份。
1. 交互通道打样（单独提交，先做）：选 3 个有代表性的技能——一个选目标（如 boss_chuanyun）、一个选牌（如 dddxiaxing）、一个可选发动加选目标（如 boss_tianyun）——迁到 HostBehavior ＋ HostStage 通道，做到：人类座位收到选择请求并可取消；AI 座位由规划器按上游 ai 函数的取向作答；在选择挂起时存档、恢复后继续，状态哈希一致。记录这条路是否需要改动存档格式、ABI 或 WS。需要则停下，把精确边界和涉及的技能数报给用户；不需要则把做法写成一页说明，后续技能照此迁移。
2. 按来源逐个改写规则，队列为：U06 欢乐卡牌 → U07 玩点论杀 → L06 wuxing → L22 段位制裁 → L29 龙血玄黄 → L10 史诗卡牌 → U01 3D精选 → L28 雷霆万钧 → L27 选将美化 → L19 末日乱斗 → L20 标记补充 → U08 英雄杀 → L25 祖安武将 → L11 太虚幻境 → U05 杀海拾遗 → 其余含规则项的来源。一个来源做完并通过第 3、4 项才进入下一个。每个技能的完成标准：
   · 逐步对照上游 trigger／filter／content：触发时机、过滤条件、次数限制、限定技、判定、随机（走引擎 RNG）全部保留；
   · 上游有 chooseTarget／chooseCard／chooseControl／chooseBool／chooseButton 或 direct／可选发动的，按第 1 项的做法向玩家提问；
   · 一个技能一段实现；只有上游是同一实现的别名才可共用，并在注释里写明；
   · 跨包技能（如雷击、不屈）绑定引擎里已有的等价实现，没有才新写；
   · 改名、内联、换函数名、加注释都不算改写。
3. 每个技能的测试：经真实对局事件路径触发（不是 debug_execute_effect），预期取自上游代码并在注释写 file:line；至少包含一条正例、一条「条件不满足不发动」的反例；有选择的技能测「玩家选 A 与选 B 结果不同」「玩家取消则不发动」和选择挂起时的存档恢复。
4. 独立复核（实现者不得参与）：每个来源做完后，复核 agent 用固定种子从该来源抽样（不少于 10%，且不少于 10 项，小来源全查）。对每个样本，用 tabletop-comparison-cdp.mjs 的启动方式在 headless Chrome 的上游无名杀里实际触发该技能并记录结果，再在原生同场景下触发，逐项比对；上游里确实跑不起来的样本，改为逐行读上游源码比对并写明原因。对照表（上游结果／原生结果／是否一致）写入证据。有一项不一致则该来源退回。只有复核通过的项可在 inventory_status.json 标「已复核等价」，复核人一栏只填复核 agent。
5. 界面：先做十周年UI 的牌桌布局——按上游的环绕式座位、身份标记、中央提示区与按钮、手牌区在真实牌桌组件上实现，用现有对照脚本出并排截图，直到布局一致；再依次做手杀ui、标记补充、千幻聆音、皮肤切换、如真似幻。每个界面钩子在清单写明原生组件与可见效果，对照通过才从「示意（待对照）」改为「已对照」。
6. Spine 4.0（裁决 35、37、38）：补齐每个特性一条测试；由未读过实现的 agent 做净室复核并留记录；请用户按指引用自有 4.0 动皮在本机对照并回传结果。
7. 需要先问用户的事项（把选项的实际后果写清楚，等答复期间继续不受阻的部分）：①第 1 项发现需要升版本号时的精确边界；②经既有通道确实无法表达的交互，逐技能列出；③资源取得方式超出裁决 38 的来源。
8. 37 来源整包八项签署：按扩展台账原八项逐来源复核；第①项以「已复核等价」占比为准，第⑧项列出该来源全部剩余差异。
9. 外部条件（未就绪时写明所需操作并继续其他项；「解耦」「建档」不等于完成，对应台账行保持「部分」）：PA08 真机 → PA11 判定、NC12 真机温控与移动端目标平台；三平台安装包签名／公证。
10. 发布：每个代码批完成第五节门禁后按裁决 25 发布并独立登记 exact-head CI；首次失败与补跑分开记录。
11. 第六阶段：上述门槛全部成立后，先报精确范围取得用户确认，再做原生产品切换并退出旧执行入口。给 URL 参数加别名之类的改动不是第六阶段。PA09 已由裁决 36 关闭。

四、工程约束：
- 保留单写 Rust 内核、不可变目录、显式结算/续行、内置 Rust＋第三方 rule-vm 路线；复用 PackageStore 与现有协议/登记工具，不并建第二套包管理、registry 或状态台账。
- 行数门禁：受维护源码/测试/DSL ≤2,000 行；Rust 非测试源码有效行 ≤1,000。按功能、来源包或项边界拆分并起有意义的名字，不压缩、不机械切块。拆文件后同步文档里的 file:line 引用。
- 分层：engine-native-kernel ← engine-native-content ← engine-native（门面）；check-native-layering.py 棘轮只降不升。
- 集成测试按分组编译：新增 tests/*.rs 必须登记到 tests/groups/<组>.rs。
- 规则/存档变化核对版本门禁、完整状态、事件、RNG 与续行；golden 变化先查原因再决定重录。
- 禁止：为了让某个统计数字或关键字搜索变好看而改名、内联、换写法；在注释或文档里写代码没有做的事；用断言文档内容或清单计数的测试充数；把上游 AI 的取舍写成规则（如「体力不大于 1 就不发动」）；让自动结算读取未公开信息；凭空生成上游不存在的实体；省掉上游的条件、随机、判定；改台账状态栏的口径来关闭条目；删除或就地改写他人或自己此前带日期的记录与表格。
- 状态用语固定为四种并分开写：「已有原生落点」「简化近似」「已复核等价」「缺失」。只有「已复核等价」计入第①项。
- 版本号：超出已登记授权的规则、存档、ABI、WS、schema 变化或新 ADR，先提交精确边界并取得用户明确答复。
- Spine 净室边界（裁决 35）不变：不读、不抄官方运行库源码与上游打包的 spine 文件，不引入依赖 spine-core 的包，产品不附带动皮资源。
- Windows 检出可能是 CRLF：新测试读包资产时按 LF 归一化；新增前端代码不依赖 Node 专有类型。
- 编译后检查磁盘并清理本批不再需要的私有缓存（裁决 23）；不清共享 target，不删别人的工作树。运行会改写已跟踪文件的脚本（如 vite build）后，检查 git status 并还原无关改动。

五、验证（按影响面）：
- 必跑：python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'；python3 -B scripts/check-source-lines.py；cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native；独立 CARGO_TARGET_DIR 下 RUSTFLAGS="-D warnings" cargo check -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets；cargo check --target wasm32-unknown-unknown -p engine-native-wasm。
- 产品面：cargo test --offline --workspace --no-fail-fast；cargo run -p conformance；apps/web-client 的 pnpm test／typecheck／build:native／smoke:native；python3 -B scripts/check-native-wasm-cross-target.py；bash scripts/cross-target-hash.sh；cargo test -p desktop-client；涉及 UI／Worker／渲染的改动跑真实 headless Chrome 产品路径。
- 新增测试须在 CI 同版 rustc（1.99.x）下无 lint 失败。
- 收口：python3 -B scripts/preflight.py 全部通过。首次失败与补跑分开记录。

六、协作：可用子 agent 并行处理互不冲突的来源，先划清文件所有权和接口，主 agent 负责整合与最终验收。实现与复核必须是不同的 agent；复核者只看上游与原生的实际行为和源码，不读实现者的说明。公共文档写入前重读。在授权范围内自主推进，不反复询问是否继续；缺少设备、凭据、权限、资源或产品语义时，写明所需输入与未验收门槛，继续做不受阻的部分。

七、记录：当前状态在进度主稿顶部**新增**带日期条目，不得删除或覆盖他人（含 Claude Code 核对）或自己此前的带日期记录；表格需要更新时新增一张带日期的表，旧表标「历史快照」保留。原始命令与输出写入有日期的 evidence 目录。文档统一用简体中文，每条记录写明日期、HEAD 与是否 dirty、规则/存档/ABI/WS 版本、命令、通过/失败/忽略和证据链接。进度只报：队列做到第几项、各来源「已复核等价 N／简化近似 N／缺失 N」、本批抽样对照表链接。不写「闭环」「全部交付」「准出条件达成」这类总括语，除非第三节全部完成。未提交的工作不得写成已完成；提交并发布后再登记。

最终逐项报告：队列进度、交互通道打样结论与版本号边界、各来源计数与抽样对照表、仍为自动结算的技能清单及原因、界面并排截图、Spine 4.0 测试与用户验证状态、37 来源八项签署、真机与签名状态、发布与 CI、第六阶段，以及未完成项和所需外部条件。现在先核对工作区，然后从第三节第 0 项开始。
```
