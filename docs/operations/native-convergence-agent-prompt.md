# 原生架构收敛完整执行提示词

将下面整段交给执行 agent。方案正文保存需求与验收条件，实施状态以进度主稿顶部扫描和剩余台账为准；本提示词只列工作项、先后与待决事项，不复制一套当前数字。2026-11-09 核对后重写，替换 2026-11-08 晚间版；同日补入裁决 35（Spine 净室自写，先只做 4.0）。该版的裁决 33（苍螭之璧）已按上游实现并入库；裁决 34 经核对**未实现**：Batch 24–26 交付的是示意级脚手架，测试断言的是自造行为（详见下文第二节）。

```text
请在 /Users/ruyimg/Documents/Claude/sanguosha/noname-next 继续完成《原生架构收敛执行方案》剩余工作：实现、集成并验收，清空剩余台账、扩展台账和规则基准里仍未达成的完成条件；不是再写建议、改口径或只跑代表样本；不以时间成本压缩范围。

一、先读（按顺序），并遵守 /Users/ruyimg/Documents/Claude/sanguosha/AGENTS.md：
1. docs/operations/evidence/2026-11-09-progress-scan/README.md（本轮核对发现，必须先读）
2. docs/architecture/native-progress.md 顶部「状态扫描（2026-11-09 11:19–12:00）」
3. docs/operations/evidence/2026-10-28-user-decisions/README.md 的裁决 33、34、35 及裁决 19、23、25
4. docs/migration/upstream-extension-reconstruction.md（顶部 2026-11-09 勘正、八项定义）与 docs/operations/evidence/2026-10-19-source-signoff-32/README.md
5. docs/architecture/native-remaining-ledger.md（37 来源、PA08–PA11、NC12 各行）
6. docs/architecture/native-convergence-plan.md（六阶段门槛）、docs/adr/0784-desktop-native-engine-prototype.md、docs/architecture/native-extensions.md、docs/architecture/native-rules-baseline.md
7. /Users/ruyimg/Documents/Claude/sanguosha/NONAME_NEXT_AGENT_SPEC.md（实施约束；位于 Git 仓库外）

二、开工前核对（2026-11-09 快照，开工时以实际为准）：
- main HEAD `1a7d11fc3`（纯文档）；最后代码提交 `cf9c6619a` 已推送 native-convergence，exact-head CI 10/10。规则 333／NGSA 107／content schema 5／ABI 759／WS8。三 crate 2219/0/6（剩余 6 个 ignore 全是手工探针）。
- 裁决 34 现状（Batch 24–26）：`apps/web-client/src/engine/extensionSurfaceRuntime.ts`、`components/battle/ExtensionSurfacePanel.vue`、`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34{,_pack9}.rs`、`crates/content-native-loader/src/decision34_{closure,pack9,pack9_data}.rs` 及对应测试是示意级脚手架：
  · Spine「骨骼动画」是 2D canvas 上的一个圆加两条线，未引入 Spine 运行库；
  · U01 3D精选 64 个技能只有 8 个有分支，其余一律摸 1 张；有分支的（如 dddxiaxing 侠行）也与上游 `3D精选/character/skill.js` 语义不符；
  · L11 太虚幻境「战役步」只有层数＋1 和 1 点伤害；
  · L08 十周年UI 的 `_usecard`／`_chongzhu` 等界面钩子被实现为摸牌，`decade_*` 9 个武将在上游不存在；
  · 上述 Rust pack 只被 crates/engine-native/tests/ 调用，未接入任何 packages/extensions/* 正式包或产品路径。
- 上游 37 个来源合计约 86 万行 JS（十周年UI 7.9 万行 JS＋2.3 万行 CSS、太虚幻境 11.4 万、祖安武将 9.9 万、如真似幻 8.8 万、皮肤切换 21.3 万，其中有大量数据）；archive 中没有 .skel／.atlas 资源文件，十周年UI、皮肤切换、如真似幻、千幻聆音、雷霆万钧、太虚幻境、史诗卡牌的 JS 引用了 Spine。
- 无 Android 设备，物理 iPhone unavailable；签名证书与公证凭据未配置。8 个临时工作树（1 个 locked）原样保留，不删别人的工作树。若发现其他 agent 正在写，先停下协调。

三、执行顺序（各项完成条件以台账对应行为准，完成后在台账行内更新）：
1. 裁决 34 清点（先做，单独提交）：为 37 个来源各建一份上游清单（建议放在 docs/migration/g0/ 或扩展台账附表），逐项列出每个武将、技能、卡牌、模式／战役节点、配置项、界面钩子与渲染面，写明上游 file:line、行为摘要、原生落点（原生规则／rule-vm 包／原生前端组件）和状态（忠实实现／示意或自造／缺失）。现有 decision34* 代码逐条归类；清单完成前不得再写「完成」。
2. 清除自造内容：删除或改写上游不存在的实体（如 decade_* 武将）和与上游不符的默认分支（「其余一律摸 1」），对应测试一并改为按上游语义断言。这是用户已确认的纠错范围，删改须在证据中逐条列出理由。
3. 逐来源按上游重做：规则类（技能、卡牌、战役、模式）进原生规则或 rule-vm 包，并接入对应 packages/extensions/* 正式包与四端产品路径；每项测试须以上游代码描述的行为为预期（注释写上游 file:line），不得以原生实现反推断言。界面类（DOM/CSS 布局、动画、特效、WebGL）在原生前端实现，与上游同场景截图对照，并在真实浏览器记录帧率。按来源分批提交，每批更新扩展台账该来源行与清单状态。
3a. Spine（按裁决 35，作为独立子项）：不引入官方 Spine 运行库，净室自写 4.0 播放器。只依据 Spine 官网公开的 JSON／二进制格式文档；实现者不得阅读或复制官方运行库源码，也不得阅读上游打包的 十周年UI/spine.js、皮肤切换/spine-lib/*、皮肤切换/v/spine-pixi-v8.js；不引入 pixi-spine 等依赖 spine-core 的包；不复制官方示例资源。先建功能清单（骨骼与世界变换、插槽、皮肤、图片／网格／加权网格／裁剪／路径／包围盒附件、IK／变换／路径约束、时间轴与曲线、多轨道混合、图集解析、WebGL 批量绘制、预乘透明与混合模式），每项用按格式文档手写的 JSON 骨骼（二进制由自写编码器生成）建测试。产品不附带任何动皮资源，按上游 dynamicSkin.js／saveSkinParams 的目录与参数格式提供玩家导入入口；无资源或非 4.0 资源回退静态立绘，并在规则基准登记差异。完成后由未读过该实现的 agent 复核未照搬官方源码；截图对照需用户本机用自有动皮与旧版无名杀进行，准备好对照步骤后请用户执行。现有 drawSpineSkeletalCanvas 示意画法删除。
4. 需要先问用户的事项（报精确边界后等答复，期间继续不受阻的来源）：①Spine 已由裁决 35 定为净室自写 4.0，3.5／3.7／3.8／4.1／4.2 是否支持届时另问；②体量极大且以数据为主的来源（如皮肤切换、千幻聆音）的资源获取与版权范围；③超出已授权版本号、ABI、schema 的变化。
5. PA09：Batch 24 的 macOS 对照显示条件①（IPC 往返 p50 2ms 高于 Worker 0ms）和②（单座顺序规划线程池慢于直调）未满足，已维持 WASM 默认。把数据整理成一页报给用户，请其在「维持 WASM 并关闭 PA09」「继续优化 IPC／规划后复测」之间选择；不得自行改判条件。
6. 37 来源整包八项签署：第 1–3 项完成后，按扩展台账原八项逐来源复核并更新签署表；移动端目标平台待第 7 项。
7. 外部条件（未就绪时写明所需操作并继续其他项）：PA08 真机（中低端 Android 开启 USB 调试、iPhone 解锁并信任，裁决 19 已同意提供设备）→ 整局与 ≥30 分钟功耗／温度／降频，据此判定 PA11，并补 NC12 真机温控与移动端目标平台；三平台安装包签名／公证（Apple Developer ID 证书与公证凭据、Windows Authenticode 证书）→ 真实安装验证。
8. 发布：每个代码批完成第五节门禁后按裁决 25 发布并独立登记 exact-head CI；首次失败与补跑分开记录。
9. 第六阶段：上述门槛全部成立后，先报精确范围取得用户确认，再完成原生产品切换并安全退出旧执行入口（engine-core／engine-ai／engine-wasm），保留上游来源、正式 ADR、原始证据与仍需的迁移／恢复能力。

四、工程约束：
- 保留单写 Rust 内核、不可变目录、显式结算/续行、内置 Rust＋第三方 rule-vm 路线；复用 PackageStore 与现有协议/登记工具，不并建第二套包管理、registry 或状态台账；不自动立项 ECS、QuickJS、全面 VM 化、强化学习等历史候选。
- 行数门禁：受维护源码/测试/DSL ≤2,000 行；Rust 非测试源码有效行 ≤1,000。按功能、来源包或项边界拆分并起有意义的名字，不压缩、不机械切块；抽函数时核对 return／?／Ok 控制流。拆文件后同步文档里的 file:line 引用（check_doc_line_refs.py 会扫描历史证据）。
- 分层：engine-native-kernel ← engine-native-content ← engine-native（门面）；check-native-layering.py 棘轮只降不升，内核需要内容行为时加 Catalog 回调。
- 集成测试按分组编译：新增 tests/*.rs 必须登记到 tests/groups/<组>.rs；单跑写 cargo test -p <crate> --test <组> <文件名>::。
- 每项先建立按上游语义会红的回归，再实现，同步调用方、导出、资源、测试与文档。规则/存档变化核对版本门禁、完整状态、事件、RNG 与续行；golden 变化先查原因再决定重录。
- 禁止：凭空生成上游不存在的武将、技能、卡牌或资源；用「默认摸 1」之类的兜底分支冒充实现；用示意图形冒充真实渲染；用只被测试调用的 fixture 冒充正式包；用自造行为的测试证明「规则差异已闭合」。
- 不改验收口径来宣告完成：完成条件以台账与扩展台账原文为准；不删改、跳过既有测试来转绿（第三节第 2 项列明的纠错除外）。
- 版本号：同一号码只被一批占用；超出已登记授权的规则、存档、ABI、WS、schema 变化或新 ADR，先提交精确边界并取得用户明确答复。
- Windows 检出可能是 CRLF：新测试读包资产时按 LF 归一化；新增前端代码不依赖 Node 专有类型（如 Buffer）。
- 编译后检查磁盘并清理本批不再需要的私有缓存（裁决 23）；不清共享 target，不删别人的工作树。

五、验证（按影响面）：
- 必跑：python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'；python3 -B scripts/check-source-lines.py；cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native；独立 CARGO_TARGET_DIR 下 RUSTFLAGS="-D warnings" cargo check -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets；cargo check --target wasm32-unknown-unknown -p engine-native-wasm。
- 产品面：cargo test --offline --workspace --no-fail-fast；cargo run -p conformance；apps/web-client 的 pnpm test／typecheck／build:native／smoke:native；python3 -B scripts/check-native-wasm-cross-target.py；bash scripts/cross-target-hash.sh；cargo test -p desktop-client；涉及 UI／Worker／渲染的改动跑真实 headless Chrome 产品路径，记录帧率并与上游同场景截图对照。
- 新增测试须在 CI 同版 rustc（1.99.x）下无 lint 失败。
- 收口：python3 -B scripts/preflight.py 全部通过。首次失败与补跑分开记录，不以补跑抹去失败。

六、协作：可用子 agent 并行处理互不冲突的实现或验证（裁决 34 可按来源分给不同 agent），先划清文件所有权和接口，主 agent 负责整合与最终验收。每个来源完成后，由未参与该来源实现的 agent 按清单逐项与上游对照复核，复核结论写进证据。公共文档写入前重读；不要同时抢写核心文件。在授权范围内自主推进，不反复询问是否继续；缺少设备、凭据、权限、资源或产品语义时，写明所需输入与未验收门槛，继续做不受阻的部分。

七、记录：当前状态在进度主稿顶部**新增**带日期条目，不得删除或覆盖他人（含 Claude Code 核对）或自己此前的带日期记录，过时条目只加「历史快照」标注；剩余台账、扩展台账、规则基准各管其职；原始命令与输出写入有日期的 evidence 目录。文档统一用简体中文，每条记录写明日期、HEAD 与是否 dirty、规则/存档/ABI/WS 版本、命令、通过/失败/忽略和证据链接；区分已实现、已修反例、已加回归、定向通过、全量通过、上游等价。不得在提交前写“已合入／已提交”；「仓内闭环」不等于整项完成，未达完成条件的行保持「部分」。

最终逐项报告：裁决 34 逐来源清单与实现状态（忠实／示意／缺失的计数）、已清除的自造内容、PA09 待用户决定的数据、37 来源八项签署、PA08／PA11／NC12 真机、安装包签名、发布与 CI、第六阶段，以及仍未完成／未验证项和所需外部条件。只有全部门槛真实通过才可声明“全部完成”；否则明确整体尚未完成。现在先核对工作区，然后从第三节第 1 项开始。
```
