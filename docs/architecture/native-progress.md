# 原生重写进度与下一步

本页是当前状态主稿，按 **2026-10-10** [进度核对](../operations/evidence/2026-10-10-progress-check/README.md)更新（源码为本地 git 基线 `72583b2b`，规则 v192、NGSA 存档格式 v41）。目标见[性能架构 v3.3](../../noname-performance-architecture.md)；逐来源扩展状态见[两仓台账](../migration/upstream-extension-reconstruction.md)；逐版本规则语义见[规则基准](native-rules-baseline.md)；按批追加的历史流水与 10-10 事故恢复逐轮记录移至[批次流水存档](native-progress-history.md)，实现与决策过程见[证据目录](../operations/evidence/)和[ADR 索引](../adr/INDEX.md)。

## 当前核验快照

> **2026-10-10 事故与恢复**：v192（机关桶）批开发中 `crates/engine-native/src/content/host.rs` 被误截断为空。经会话记录回放与证据语义重建，九轮修复后全量转绿，14 golden 补齐 v191/v192 两批重录（[恢复证据](../operations/evidence/2026-10-10-replay-golden-v192-recovery/README.md)，逐轮数字见[流水存档](native-progress-history.md#2026-10-10-hostrs-截断事故恢复记录)）。当日 `noname-next` 建立仅本地使用的 git 仓库（基线提交 `72583b2b`，pre-push 钩子拦截推送），此后以提交为回退点。恢复遗留项见下方“后续待做”P0/P3。

| 口径 | 实际结果（2026-10-10，git `72583b2b`） | 边界 |
|---|---|---|
| 当前版本 | 规则 **v192**（`STANDARD_RULESET_VERSION`）、NGSA 存档格式 **v41**（`ARCHIVE_FORMAT_VERSION`）；逐版本语义见[规则基准](native-rules-baseline.md)，v192 于本日补登 | 版本号只说明编码与语义已递增，不等于上游等价 |
| 原生全量 | `cargo test --offline -p engine-native --no-fail-fast`：**525 通过、0 失败、1 ignore**（24 个测试目标，含 14 个 golden 场景、13 个包真实宿主生命周期测试、存档加固负例）——[日志](../operations/evidence/2026-10-10-progress-check/engine-native-tests.log) | 全绿限于现有测试；host.rs 14 个宿主行为分支是空实现（P0），效果靠内核钩子承担，未逐项对照批次证据 |
| 装载与包管理 | `content-runtime`+`engine-ai`+`content-native-loader` 合计 **63/0/0**（16 个目标）——[日志](../operations/evidence/2026-10-10-progress-check/deps-tests.log) | 正式客户端/服务端尚未接入原生装载与规则执行 |
| CI 门禁 1b | `RUSTFLAGS="-D warnings" cargo check -p engine-native --lib`：**失败（exit 101）**，lib 12 条警告被拒——[日志](../operations/evidence/2026-10-10-progress-check/gate1b-check.log) | 10-07 记的“lib 零警告”已回退，P3 重开 |
| 上游来源 | 清点脚本重跑与 9-23 基线**不一致**：noname HEAD 快进到 `df2682e6`（8 个扩展目录未改）；Liuli 10 个来源实例内容变化（10-08 `reset`，本地修改 50→7 个文件）；仍为 37 项，基线未重录——[清点复核](../operations/evidence/2026-10-10-progress-check/inventory-check.txt) | 变化来源的功能清单与切片影响待复核（P1） |

全工作区 `cargo test --workspace`、conformance、跨目标哈希、前端构建、联机与目标设备本次未运行；最近一次全工作区结果仍是 9-23 第 48 轮（v59），只适用于当时源码。

## 已完成并复验的切片

| 项目 | 已交付与本次确认 | 验收边界 |
|---|---|---|
| P0 回放与目录 | 拆牌循环重复入牌已修（9-24）；标准目录现为 124 张声明表（v74 核定，`standard_catalog_inventory` 守护）；14 个 golden 场景随版本逐批重录，最近一次为 2026-10-10 v191/v192 补录，本日通过 | 重录依据见[9 月 24 日证据](../operations/evidence/2026-09-24-replay-golden-v60/README.md)与[v192 恢复证据](../operations/evidence/2026-10-10-replay-golden-v192-recovery/README.md)；124 张含原生扩充段，与上游 108 张的差异见[规则基准](native-rules-baseline.md#上游标准牌表对照) |
| P0 N11 机器 tag 与解码加固 | XiaoJi=43、Dismantle=29；tag 唯一/往返、枭姬挂起和错 tag/短输入负例通过；未知牌种/尾部载荷/势力编码加固通过 | 加固时格式为 v2，现为 v41；逐次格式变化见[规则基准](native-rules-baseline.md) |
| 非法命令与注册表 | 主动使用无懈返回 WrongPhase；`deny(unreachable_patterns)`、数据键表及牌种伴随技能表检查已落地 | 只证明已登记非法命令和表项，不能替代所有规则分支验收 |
| C0 / E02 装载器 | `content-native-loader` 数据桥接、严格错误检查、**八份测试桥接全部迁移**（古剑/玩点论杀/欢乐/炉石/gwent/运筹/3D精选/五行）；同名名称/技能名单重载复用已有验收；standard 46 技能编号声明表锁定 | 实际产品调用、独立规则载荷和生命周期未完成；全局符号表仍永久保留首次出现的唯一名称/名单 |

## 完成状态

| 领域 | 已核实交付 | 仍需处理 |
|---|---|---|
| 原生内核与恢复 | 顺序事件执行、类型化暂停、内容锁建局/恢复、NGSA 存档（现 v41）、所列选择与持牌恢复回归 | 存档异常输入、全规则/模式、长局和跨目标恢复验收 |
| AI 与信息边界 | 原有单选合法回退等反例已修；同步 BoundedPlanner 与部分座位视图用例通过 | 工作池、运行中取消、候选质量及全部模式/扩展的信息隔离 |
| 扩展基础与切片 | 15 个转换包（数据型卡牌/武将包），其中 13 个有真实宿主全生命周期测试（10-10 通过）；部分宿主行为与内容锁恢复；具体包和张数见[台账](../migration/upstream-extension-reconstruction.md) | 两仓 37 来源整包原生验收仍为 **0/37**（生命周期只是整包条件之一）；开放第三方执行/SDK、模式/AI/UI/联机和已登记规则差异 |
| 标准玩法 | 上游 standard 33 将全对齐（另原生新增曹仁）；身份/国战原型与 versus/斗地主/Boss/KOF 模式基线（v80–v84）；逐技对照五批与逐人询问、仁德累计等局部修复 | [规则差异表](native-rules-baseline.md#本地上游对齐的新增核查)的已登记差异与逐技能语义；各模式变体与 KOF 多局连续会话（宿主层） |
| 产品与性能 | 原生 runner、同宿主跨进程样例保留历史证据 | 客户端/服务端仍用旧线；原生产品接入、目标设备与同负载收益未验收 |

**原有扩展机制必须保留。** 玩家应能独立安装、启停、升级、卸载；作者应能发布武将、技能、卡牌、模式、规则、AI、资源、配置和界面扩展。用户指定的两仓全部 37 来源均在范围内；旧 JS 零改动范围未决，不影响保留这些能力。详见[宿主契约](native-extensions.md)与[两仓台账](../migration/upstream-extension-reconstruction.md)。

## 后续待做（按阻塞关系）

已完成的回放/目录、XiaoJi tag、存档解码加固和八份桥接迁移移至上表或标记完成。下列各项除注明“已完成”者外均为未完成或部分完成；按批次的历史流水见[批次流水存档](native-progress-history.md)。

| 优先级 / 编号 | 当前状态 | 下一步及完成条件 |
|---|---|---|
| P0 / 事故恢复遗留（2026-10-10） | **待处理**：全量 525/0/1 通过，但重建代码尚未逐项复核 | ① `content/host.rs` 有 14 个宿主行为分支只有 `TODO(事故恢复)` 空实现（MoshouImmunity、GujianFoodYuanbaorou、WudianCiqiu/Chezhen/Tuji/GuangshuSpade/Duwen/Feiren/Xiandan/Jijia/Longxiang/Lueying、XianjianTannang/Xuanyan）。抽查刺猬、探囊两项，效果由内核钩子承担（如 `engine.rs` 探囊距离豁免）；13 项有同名回归测试通过，元宝肉无同名测试。需对照各批证据逐项确认空臂是否本就无事可做，确认后删除 TODO 或补实现。② 恢复期备份 `host.rs.pre_assembly`、`host.rs.recovered-v192` 仍在 `src/content/`（不参与编译，git 基线已含），可删。③ 恢复期调试探针：`machine.rs` 1 处、`resolution.rs` 3 处按环境变量输出 eprintln，其中 `push_frame` 每次压帧都读一次环境变量；建议删除或改为编译期开关。清单见[源码核对](../operations/evidence/2026-10-10-progress-check/source-checks.txt) |
| P1 / E01 上游来源变化（2026-10-10） | **待复核** | noname 快进 42 个提交（扩展目录与 `card/standard.js` 未改，`character/standard/skill.js` 仅洛神判定回调签名改动）；Liuli 10 个来源实例内容变化。下一步：逐项判断是否影响 E01 功能清单与已转换切片（wuxing、cardpile 不在变化名单内），复核后重录或保留 9-23 基线并写明原因（[清点复核](../operations/evidence/2026-10-10-progress-check/inventory-check.txt)） |
| P0 / N11 存档解码加固 | **已完成（2026-10-07）** | `card_kind_from_num` 可失败化（`UnknownCardKind(u32)` 不截断），4 处调用全部传播；`dec_snapshot` 后读尽检查（`TrailingBytes`）；未知编号改 255 与尾部夹带 4 字节两负例随测试运行；`CardKind::ALL` 86 牌种编码往返 + `enc_winner` 改走显式 `enc_faction`（往返与 BadTag 拒绝）。规则/格式不递增（合法 v2 字节不变，14 golden 通过）。证据见[本批核验](../operations/evidence/2026-10-07-unfinished-batch/README.md) |
| P1 / C0、E02、G1 统一装载 | **八份测试桥接全部迁移完成（2026-10-07）；实际产品接入仍待做** | huanle/hearth/gwent/yunchou 补 `pack` 段（炉石键规范化、欢乐改 `skills[].mapping`、索链甲 `link_on_equip`）、d3 `character.json` 补 pack 段、wuxing 新增自描述 `cards.json`，六份测试改调装载器：6/6、7/7、6/6、12/12、3/3、3/3 全部保持原名/数/结果；A0 第 3 点 standard 46 技能编号声明表落地。装载器 `cards` 可选（纯武将包）、`generals` 合并、`link_on_equip` 解析。剩余：接入实际安装包→原生建局调用；新增唯一名称的上限/回收策略仍需确定。证据见[本批核验](../operations/evidence/2026-10-07-unfinished-batch/README.md) |
| P1 / E01、G0 功能清单 | **来源清点与功能面基线完成（2026-10-07，37/37）；行为样本不完整** | 功能面基线已落盘（[E01 清点](../operations/evidence/2026-10-07-e01-feature-inventory/README.md)：入口规模/段计数/配置与设置引用/子插件/资源体量/混淆标记，龙血玄黄混淆特征已确认——1 行入口 + packed 标记，行为样本须动态观测）。剩余：逐来源旧行为样本（动态）、菜单/规则分支语义清单、目标接口与验证入口逐项登记；壳扩展 `const card = {}` 段计数为工具改进项。**龙血玄黄动态采样立项定案（2026-10-08）**：静态勘察完成（434KB 单行混淆=obfuscator.io 字符串表+debugger×2；子资源手杀 UI 补丁可读；存活明串→功能假说=手杀曹髦专属扩展：剑特效+「道心值」技能系统+UI 样式补丁）；三阶段方案 LX.1 静态可读化（Node 隔离解码+AST 还原，先中和 debugger）→ LX.2 隔离旧线勾桩采样 → LX.3 清单定案与规则/界面接口分配（执行混淆代码仅限隔离环境，产物须与静态读稿交叉印证）；方案与证据见[勘察立项](../operations/evidence/2026-10-08-e01-longxue-survey/README.md)、台账 L29/G0 行已同步。**LX.1 执行完成（2026-10-08）**：混淆判定修正为 jsjiami.com.v6 多字符串表（packed 假说被 stub 运行探针推翻——正文无 eval 载荷）；VM 隔离注入 28 函数+7 表声明+全字面量扫描代换，632/637 调用点解码（余 5 处动态键登记）；功能清单初稿=武将 mb_caomao×2+技能（乾龙四段/清正/酒诗/绝进/放逐/围通）+剑特效 CSS/UI 补丁（files 全空资源内联），规则面与界面面并存。证据见[LX.1 可读版](../operations/evidence/2026-10-08-e01-longxue-lx1/README.md)。**LX.2+LX.3 执行完成（2026-10-08）**：记录型 Proxy 桩隔离采样（config={} 与 {shengjian:true} 两轮）——规则面仅 2 技能（mbqianlong 无条件；_CMeffects 门控），静态清单两处修正（mbcm* 四技能与 mb_caomao×2 为 hasSkill/game.me.name 外部依赖引用，本包零武将/牌注册），界面面=剑特效 CSS/cmarea DOM/updateProgress；LX.3 定案：范围明确（2 技能+纯 UI），规则面若迁移走 G1 转换路径+逐技对照口径、界面面走宿主界面扩展，战斗内触发轨迹登记为迁移前置残余。证据见[LX.2/LX.3 采样定案](../operations/evidence/2026-10-08-e01-longxue-lx2/README.md)。**37/37 来源功能清单齐（G0 完成条件达成，龙血玄黄为样板）** |
| P1 / E03、G1 开放规则宿主 | **E03 全链路验收完成（2026-10-07）：事件帧生命周期缺陷已定位修复，翻面/窃取/据守/面具 ignore 全部移除** | 猴子偷桃切片（2026-10-07）：使用前取消窗口全链路可用——确认窗口前移到使用提交的命令层挂起机器（`MonkeyStealMachine`，ActivateSkill 槽模式）：他人出桃 → 逐持有者确认门（挂起态可恢复，`active_ask` 派生校验一致）→ 确认：取消使用（无 CardUsed 帧）、桃收为持有者手牌、弃猴子离场撤销装备技；全部放弃：按普通桃结算（回血、入弃堆）——两种路径与恢复/回滚均有测试。`CardUseData.cancelled` 不入哈希（golden 不变）+ 格式 v3。与上游差异登记：原生取消发生在使用提交拦截（无 useCardToBegin 帧），多持有者按座位序逐个询问。证据见[E03 核验](../operations/evidence/2026-10-07-e03-monkey/README.md)。写轮眼切片（2026-10-07）：CardKind::Xuelunyang（88）+ HostBehavior::HuanleXuelunyangCard（76，多步宿主机器：确认→选目标→PickTag 选技能→`add_skill_until_turn_end` 临时授予）+ 欢乐包独立载荷；窃取流程、到期禁用、Pass 路径（OptionalGate 放弃 = 空选择取消）全部测试通过。曹仁「据守」技（真实翻面来源，2026-10-07）：结束阶段可选——翻面经 TurnOverBefore 子事件（面具可取消）+ 摸 1（登记差异：翻面被取消时摸牌仍执行）；`caoren_jushou_flips_and_mianju_cancels_flip` 通过（两次据守翻面/翻回 + 面具取消）。mianju（漩涡面具）翻面原语（2026-10-07）：PlayerState.face_down + TurnOverBefore/TurnOvered 事件对 + `debug_flip` 原语 + CardKind::Mianju（89）伴随表 + 面具取消/摘除后翻面测试；规则 v61 + 格式 v4（14 golden 重录）。**帧生命周期缺陷根因与修复（2026-10-07）**：`Ctx::emit` 曾把 TurnOverBefore 子事件以 Commit 阶段起步（Damage 之外一律 Commit），前置取消窗口从未执行——commit 以 cancelled=false 先行翻转，AfterTriggers 再按 f.kind 查候选把取消写回已提交的残留帧（旧 DBG「commit 先于写回、cur_frame 残留」与根因吻合）；修复 = emit 对 TurnOverBefore 子事件从 BeforeTriggers 起步 + TurnOverBefore 帧 after 窗口静默（目录无「翻面后」触发种类，避免取消技重复入队）+ `debug_flip` 事件驱动化（压帧→run 驱动到弹栈，非 Done 即 panic 回滚）。写轮眼状态膨胀（规模探针曲线已平坦）与面具取消失效确认同源。新增回归守卫 `flip_frame_lifecycle_no_residue`（翻面后 stack/parks 必须清空）；规模探针保留 ignore 诊断。**独立规则载荷验收完成（2026-10-07）**：上游 7 张实体牌逐张对照（list.js/card.js/skill.js，commit 5c02a745）——酒尾 ♥9 宝物补齐（`CardKind::Jiuwei` 编码 89 表尾追加 + 包声明映射既有 `host:phase_end_recover_or_draw`，真实装备驱动 + 受伤回血/满血摸牌/放弃三路径 + 恢复测试）；修正猴子花色点数（♥8→♦5）、面具花色点数与栏位（♠1→♣3、宝物栏→防具栏 equip2）；`huanle_pack_payload_matches_upstream_list` 守护 7 张清单与装备栏；苦无/写轮眼语义差异登记（见规则基准）。全量 engine-native 360/0/1、golden 不变（扩展牌种不入标准 golden）、content-runtime/loader/engine-ai 全绿、`-D warnings` 通过。证据见[帧生命周期核验](../operations/evidence/2026-10-07-e03-frame-lifecycle/README.md)与[载荷验收](../operations/evidence/2026-10-07-e03-payload-parity/README.md)。**跨牌组合矩阵完成（2026-10-07）**：`huanle_combo_*` 三测试——窃取×据守（窃得据守对窃得者真实触发翻面+摸 1，复制非转移）、窃取×据守×面具（前置窗口对窃得技同样生效、摸牌仍执行）、苦无×非杀伤害负例（手里剑伤害不触发苦无，两线一致）×雷杀（is_sha_like 超集行为面固化）；语义交互通道共四类（伤害来源联动/翻面前窗口/窃取/使用前取消），其余组合为无交互独立技并存（装备生命周期覆盖），台账 U06 据此关账组合项。全量 engine-native 363/0/1。剩余：mianju 验收的真实翻面来源仍归 P2/N08 曹仁条目装配（当前上游归档无标准曹仁/据守，`debug_flip` 验收驱动保留） |
| P1 / E02、C3 生命周期 | **升级/回滚/依赖图/吊销/卸载场景已补（2026-10-07）** | 已有：跨层升级场景（v1 建局保存 → 错版恢复 `RuleVersionMismatch` 拒绝 → 回滚恢复续局，`e02_c3_lifecycle`）+ 依赖图五场景（激活强校验拒绝悬空依赖、吊销级联剪枝到不动点、升级受阻、激活失败原子性、revoke 重激活拒绝，`c3_dependency_conflict`）+ **卸载进行中对局**（`e03_c3_uninstall_during_active_game_lock_pinned`：宿主注册表卸载不影响活局——引擎持 Arc 目录无注册表回写通道，扩展内容仍可查、命令照常、锁指纹不变；卸载后标准目录恢复挂起对局被拒、原包集合恢复成功（存档自包含）——宿主须保留包字节直到无对局引用（宿主策略登记，引擎不感知引用计数）；卸载后新开局不含扩展内容）。**签名信任链与多级历史补齐（2026-10-08，`signature_trust_chain`）**：签名安装（正确密钥 signed=true 激活）→篡改字节拒绝→错钥签名拒绝→verify_key 模式未签名拒绝→多级版本历史（v1/v2 同槽并存、activate_version 切换、list_installed 双版本保留）→hash 撤销 v2（重装 Revoked）→v1 回激活（历史回退）。产品化策略仍登记：密钥分发/轮换与吊销名单分发属宿主运营面（引擎/宿主只验 keyed-blake3 v1 签名）。 |
| P2 / N08、N11 规则对齐 | **局部修复；完整规则未验收**（2026-10-10 核对） | 已落地（各批语义与登记差异见[规则基准](native-rules-baseline.md)，逐批原文见[批次流水存档](native-progress-history.md#p2--n08n11-规则对齐批次流水2026-10-07--10-08)）：装配授予统一与 standard 缺失 8 将补齐；缺牌种清零与牌表逐张对照（标准目录 124 张）；九把命名武器与仁王盾（v64–v73）；花色点数对齐（v74）；逐技对照五批（v75–v79）；versus/斗地主/Boss（含 addFellow）/KOF 模式基线（v80–v84）。剩余：已登记的逐技能语义差异（含 v192 机关桶未实现「无人受伤则使用者摸一」与结算序）；各模式变体（versus 替补/4v4、斗地主特殊开局、Boss 专属技能包、KOF 多局连续会话在宿主层）；同命令、事件、随机流和恢复轨迹对照，语义变化递增规则版本 |
| P2 / E04–E06、G2–G5 整包 | **整包 0/37** | 迁移剩余内容/角色/模式，接 AI、配置、界面、资源、战役档案和联机；逐来源完成全部功能、组合、保存恢复、安装升级及目标平台验收后才上调整包状态 |
| P2 / N10、N12 AI 与产品 | **同步原型/历史样例** | AI 工作池、运行中取消、合法座位视图与策略质量；正式客户端/服务端原生接入、跨进程/目标平台和帧/延迟实测。旧线测试与性能数据单列 |
| P3 / 维护：编译警告 | **回退，重开（2026-10-10）**：10-07 曾清零 lib 警告并在 CI 开启 Gate 1b（`RUSTFLAGS=-D warnings`），本日实测 lib 12 条、门禁失败 | lib 12 条：host.rs 未用导入/变量 5 条与字段自赋值 1 条（`self.card = self.card`）、`snap.rs` 流离 `liuli_card` 未用、`kernel.rs`/`engine.rs` Boss 字段未用、`presentation.rs` 未用变量等（[日志](../operations/evidence/2026-10-10-progress-check/gate1b-check.log)）。修复后重跑 Gate 1b 与 engine-native 全量。测试与示例另有约 576 条警告（24 个目标合计 588 条减去 lib 12 条），处理与否另议 |

## 性能优化核验后的待办（2026-09-24）

O1 已于 2026-10-07 完成；截至 2026-10-10，证据目录中没有 O2–O5 的新测量或验收记录。

[优化假设复核](../operations/phase17-test-perf-infrastructure.md#2026-09-24-优化假设复核)确认若干旧线代码路径、文件数量和依赖配置问题，但原报告的 P0/P1、5–10 倍、80% 等收益没有可复查基准。`engine-core`/`engine-ai`/`engine-wasm` 是当前产品线，`engine-native` 是重构线；两线分别设指标，不把旧线提速记为原生架构验收。以下按“先测量、再局部改动、最后等价验收”执行。

| 编号 / 状态 | 处理计划 | 完成条件 |
|---|---|---|
| O1 / 已完成（2026-10-07） | `tempfile` 已移至开发依赖（src/ 零使用，仅 tests 经 content_runtime 别名）；dev-dependencies 中重复的 `content-compiler`/`content-schema` 已删除 | `cargo tree` 验证 engine-core 与 engine-wasm 正常依赖 0 命中 tempfile；engine-core 全量 3777/0、锁文件随检查更新；wasm32 目标本机未装，真实 wasm32 构建留待目标环境（[维护批证据](../operations/evidence/2026-10-07-maintenance-batch/README.md)） |
| O2 / 待测，构建效率 | 先确认空闲磁盘，在独立 `CARGO_TARGET_DIR` 固定工具链、机器和起始状态，测 `engine-core` 干净/热态 `cargo test --no-run` 的耗时、链接目标数、磁盘；不清理并行代理共享的 `target`。若确为主要瓶颈，挑 20–50 个文件试合并 | 试点前后测试名/数量/过滤行为相同、全套通过；比较干净与增量构建/磁盘后决定是否扩大，不预设 1,078→20 或 90% 收益 |
| O3 / 待测，旧线运行时 | 复用 `benchmark-runner` 记录 `query` 命中率、分配/耗时、`project_observer` 扇出、AI 节点成本；只对已证明的热点局部优化查询 key/来源、同状态哈希复用或 Store 遍历。优先保留已有命令提交与不变量校验 | 同负载前后分位数与分配数可复查；命令结果、规则状态哈希、事件、随机流、回放和存档恢复保持一致。没有证据前不增加跳过校验的 `submit_sim` |
| O4 / 待测，WASM 交付 | 固定 `wasm-pack`/`wasm-opt` 版本，对现有构建和体积优化配置做可重复实验；比较原始、gzip/Brotli 体积与加载耗时 | 浏览器功能回归通过，记录体积/启动数据后选择配置；不以原文“7.75 MB→1.5 MB”作验收值 |
| O5 / 条件立项，格式与结构 | JSON 快照/回放、动态派发、大文件拆分、Blake3 工具合并和目录重命名均暂不作为性能任务；只有实际剖析或维护缺陷指向它们时再建最小改动 | 若改存档/回放，版本化并保留旧 JSON 读取及哈希链校验；若拆文件或换派发，保持 API、测试与构建/运行指标不退化 |

上述优化不改变 E01–E06 和上游规则待办的完成条件；测量结果及每项是否立项应回填本页或对应证据，不把代码存在、测试通过和性能收益混为一项。

## 证据与维护

- 最新实跑是[2026-10-10 进度核对](../operations/evidence/2026-10-10-progress-check/README.md)（git `72583b2b`，规则 v192）；上一轮为[2026-10-07 进度核对](../operations/evidence/2026-10-07-progress-check/README.md)（v60），[2026-10-06 进度复核](../operations/evidence/2026-10-06-native-progress-audit/README.md)另含存档缺陷探针与来源清点。10-07 至 10-10 各批实现见对应日期的证据目录与[批次流水存档](native-progress-history.md)；更早[第 48 轮](../operations/evidence/2026-09-23-native-review-48/README.md)的全工作区结果只适用于当时 v59。历史数值只描述对应源码。
- `noname-next` 自 2026-10-10 起用本地 git 管理（不推送远端，`.git/hooks/pre-push` 拦截）；核验记录写明对应提交，回退以提交为准。
- [规则基准](native-rules-baseline.md)记语义和版本，[覆盖清单](../native-coverage-manifest.md)记机制范围，[扩展台账](../migration/upstream-extension-reconstruction.md)记逐来源状态。新增实现同步源码、定向回归、全量结果和台账。
- “反例已修”“已加测试”“子包切片完成”“整包验收通过”“上游规则等价”是不同结论；性能收益必须有同负载测量。
