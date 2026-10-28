# ADR 编号索引（INDEX）

- 正式 ADR：`docs/adr/NNNN-主题.md`，编号递增，保留历史决策；后续核验以注明日期的补注更正范围和状态，不把旧结论当作当前完成度。
- 已被正式 ADR 覆盖的三份调研底稿已删除，见 [清理记录](../architecture/legacy-reference.md)。
- 当前目标与完成度分别见 [性能架构](../../noname-performance-architecture.md) 和 [重写进度](../architecture/native-progress.md)；旧 ADR 的阶段结论只代表当时状态。
- ABI 编号领取/落地的权威记录：根目录 `ABI-COORDINATION.md`。

> 当前核验状态、测试数与待办只写在[重写进度](../architecture/native-progress.md)，本索引不复制。2026-09-23 [第 16 轮](../operations/evidence/2026-09-23-native-upstream-review-16/README.md)（规则版本 28、238 项）为历史快照（2026-09-24 补注）。ADR 0774/0775 的全部关闭声明已补注更正。扩展 E01–E06 依用户要求保留为必做，当前缺口及修复验收统一见进度主稿。

## 编号冲突消歧表（2026-08-21 登记；2026-10-18 NC11 重算并机器校验）

并发批次领取编号时未查重，以下编号被多个文件共用。引用这些编号时请以
**文件全名**（或 ABI 号/包名上下文）消歧；本表为唯一映射依据。

- **重算（2026-10-18，`python3 -B scripts/check-adr-index.py --json`）**：编号文件 **731**、唯一编号 **702**、重复编号 **27 组 / 56 个文件**、标题错号 **2**（已修，见下）。2026-10-18 审查快照的「27 组重复编号」与本次实测一致，但审查数字只作历史快照，**执行/CI 一律以脚本重算为准**。
- **唯一标识**：本表不重命名历史文件（避免断链）。后续维护需要唯一标识时，用 **文件全名**，或新分配编号时在 `ABI-COORDINATION.md`/本表登记；同一编号不得再新增第三个文件。
- **阻止新增冲突**：`scripts/check-adr-index.py` 在 CI 中检查「磁盘重复编号 ⊆ 本表」且「本表分组 = 磁盘分组」，未登记的新冲突、过时登记、分组不符、索引坏链、标题错号均以 exit 1 失败。
- **标题错号勘正（2026-10-18）**：`0446-clan-batch1.md` 原标题误写 `0444`、`0447-extra-finalize-stats-fix.md` 原标题误写 `0445`（两号均已被其他文件占用，且 `ABI-COORDINATION.md` 引用的是 0447）——按文件名更正标题编号，正文与决策未改。

| 编号 | 共用文件（引用须用全名） |
|---|---|
| 0398 | `0398-abi526-shenhua-feijun-binglve.md` / `0398-shiji-mb-shen-machao.md` |
| 0422 | `0422-shiji-db-wenyang.md` / `0422-xianding-batch13.md` |
| 0431 | `0431-abi549-551-shenhua-zhangjiao-releiji-guidao-huangtian.md` / `0431-shiji-wujing.md` |
| 0442 | `0442-standard-m1-repack-extensions.md` / `0442-xianding-batch19.md` |
| 0443 | `0443-newjiang-batch1.md` / `0443-xianding-batch20.md` |
| 0445 | `0445-abi559-shenhua-zhanghe-qiaobian.md` / `0445-shiji-xinyangjie-qingyu-mifuren.md` |
| 0446 | `0446-abi560-shenhua-liushan-xiangle-fangquan-ruoyu.md` / `0446-clan-batch1.md` / `0446-shiji-zhengsu-zhujun-lvfan.md` |
| 0453 | `0453-abi563-shenhua-zhugezhan-zuilun-fuyin.md` / `0453-key-batch1.md` |
| 0454 | `0454-abi571-shenhua-yanyan-juzhan.md` / `0454-yijiang-batch4-gaps.md` |
| 0455 | `0455-abi565-skill-pile-count.md` / `0455-shiji-zhouchu-zhanhai-zhangming.md` |
| 0456 | `0456-xianding-batch21.md` / `0456-yingbian-xiongshu.md` |
| 0457 | `0457-abi566-owner-skill-pile-count.md` / `0457-xianding-batch22.md` / `0457-yingbian-jiachong-jianhui.md` |
| 0458 | `0458-abi564-bingshi-pot-lvyi.md` / `0458-shiji-spqiai.md` |
| 0462 | `0462-abi567-choose-other-hand-store.md` / `0462-shiji-wangcan-shanxi-suoming.md` |
| 0463 | `0463-abi569-gain-all-skill-pile.md` / `0463-shiji-tongduo.md` |
| 0465 | `0465-abi573-shenhua-wangji-qizhi-jinqu.md` / `0465-shiji-liuba-duanbi.md` |
| 0466 | `0466-abi574-shenhua-kuailiangkuaiyue-jianxiang-shenshi.md` / `0466-shiji-sunyi-zaoli.md` |
| 0467 | `0467-abi570-bingshi-pot-yuji.md` / `0467-shiji-guying.md` |
| 0547 | `0547-abi624-ddd-caomao.md` / `0547-xianding-batch28.md` |
| 0550 | `0550-abi626-ddd-xianglang.md` / `0550-xianding-batch28-ref-fix.md` |
| 0552 | `0552-abi627-ddd-yujin.md` / `0552-xianding-batch30.md` |
| 0556 | `0556-abi629-ddd-baosanniang.md` / `0556-xianding-batch32.md` |
| 0561 | `0561-xianding-batch33-ref-fix.md` / `0561-xianding-batch34.md` |
| 0571 | `0571-xianding-batch36-ref-fix.md` / `0571-xianding-batch39.md` |
| 0580 | `0580-abi640-ddd-liangxi.md` / `0580-xianding-batch41.md` |
| 0587 | `0587-xianding-batch43-ref-fix.md` / `0587-xianding-batch45.md` |
| 0590 | `0590-xianding-batch45-ref-fix.md` / `0590-xianding-batch48.md` |

已知高歧义引用点（引用时需按上下文消歧）：

- 架构计划「已收尾（ADR 0442）」指 `0442-standard-m1-repack-extensions.md`；
  `ABI-COORDINATION.md` 领取表的 0442 行另指 `0442-xianding-batch19.md`。
- 架构计划 M4/M5 段的 ADR 0547/0550/0552/0556/0580 均指 `abi*` 系 ddd 批次文件
  （ABI 624/626/627/629/640），非同名 xianding 批次文件。
- `ABI-COORDINATION.md` 中「落地见 ADR 0398」指 `0398-abi526-shenhua-feijun-binglve.md`。

> 维护约定：新 ADR 落盘前先 `ls docs/adr/` 查重编号；发现新冲突请补登本表，
> 不回改历史文件名（避免断链）。
> 机器校验：`python3 -B scripts/check-adr-index.py`（单测
> `python3 -B -m unittest discover -s scripts -p 'test_check_adr_index.py'`）；
> 新增冲突未登记、登记过期、标题错号或索引坏链都会使检查失败。

- [0645 剩余深水区机制章程](0645-remaining-deep-water-mechanisms.md) —— 散技浅面全域挖尽后的阻塞项清单/原语设计草案/缺口台账
- [0646 jsrg 城池/访客/狩猎状态系统切片章程](0646-jsrg-cheng-ye-lie-state-systems.md) —— 蓄谋族闭环加注；猎标记族(最浅,含额外回合缺口)/城扩张堆族/访客武将牌族(永久高温债)优先序

## 原生线 ADR 0703–0767（历史决策）

2026-09-22 整理：去重并按编号排序；当前状态以 [进度主稿](../architecture/native-progress.md) 为准。

- [0703 原生内核第一批落地——只读规则目录 + 紧凑对局状态 + 原生技能暂停恢复](0703-native-kernel-first-batch.md)
- [0704 挂起机器规范快照与命令信任边界——engine-native 摘要可区分与保存恢复](0704-native-snapshot-command-boundary.md)
- [0705 语义差异收敛第一批——AOE 逐目标无懈、判定窗口与身份局胜负切片](0705-native-semantics-nullify-identity.md)
- [0706 牌区修复、快照结构检查与首轮暂停点恢复矩阵](0706-native-zone-conservation-restore-matrix.md)
- [0707 结构反例修复、九案例恢复矩阵与宿主时间片原型](0707-native-structure-slice-matrix.md)
- [0708 旧线规则对照与每段预算修复](0708-native-rule-source-alignment.md)
- [0709 段检查点中止、规则版本 3 与纯命令身份局](0709-native-segment-abort-versioning.md)
- [0710 N07-b 恢复后中止——段检查点随快照延续与严格失败契约](0710-native-restore-checkpoint-abort.md)
- [0711 判定队首消费与装备惩罚测试（第十四批历史记录）](0711-native-judge-order-alignment.md)
- [0712 鬼道/天妒首组技能切片（规则验收未完成）](0712-native-first-official-skills.md)
- [0713 判定稳定分组与版本 4（N06-a 原反例关闭）](0713-native-judge-grouping-version4.md)
- [0714 空改判队列拒绝、天妒范围与鬼道旧牌归属修复](0714-native-guidao-tiandu-semantics-n04d.md)
- [0715 选择承接关系逐字段校验（N04-e 关闭）——由机器状态推导待答询问](0715-native-choice-binding-validation.md)
- [0716 判定/获得事件三件套与 N05/N08 回归补全](0716-native-judgement-events-regressions.md)
- [0717 N09 首批——新线无界面 runner 与部分基准记录（phase19）](0717-native-runner-baseline.md)
- [0718 N06 内奸纯命令终局与守恒检查边界](0718-native-neijiang-pure-command.md)
- [0719 检查点再嵌套拒绝与装备改判切片（验收未完成）](0719-native-checkpoint-hierarchy-guidao-equip.md)
- [0720 判定事件全序落地、鬼道装备资格与规则版本 6](0720-native-judgement-order-equip-qualification-v6.md)
- [0721 N09 runner 判定负载跑通与计量口径修正](0721-native-runner-judgement-load-exit0.md)
- [0722 N05 主动判定恢复矩阵、逐提交守恒与洗牌组合回归](0722-native-n05-active-judgement-matrix-reshuffle.md)
- [0723 N06 距离/传导参照差分与奖惩先后时序断言](0723-native-n06-distance-differential-reward-ordering.md)
- [0724 N08 表内技能授予、动态资格与卸装效果](0724-native-n08-content-expansion.md) —— 表内组合与资格回归；完整整将缺项见复核
- [0725 N04 检查点与当前段关系完整验收](0725-native-n04-checkpoint-segment-association.md)
- [0726 N05 濒死询问轮次语义与负体力多桃连续救援](0726-native-n05-negative-hp-rescue-rounds.md)
- [0727 N06 重放/恢复逐命令事件对照](0727-native-n06-replay-restore-per-command-parity.md) —— Prototype 60 命令、半程恢复；不外推任意保存点或所有模式
- [0728 N05/N02/N09 小批闭环——矩阵扩面、死亡清理终态与峰值内存](0728-native-n05-n02-n09-small-batch.md)
- [0729 N08 八卦阵语义与武将表扩面、N09 关闭计数器对照](0729-native-n08-armor-bagua-generals-n09-counter-compare.md)
- [0730 N10 权威状态与可见视图分离（信息隔离第一批）](0730-native-n10-player-view-isolation.md)
- [0731 N10 第二批——开放选择与表现事件的可见性契约](0731-native-n10-choice-event-visibility.md) —— 选择/摸牌投影已有；私有应答日志泄露待修
- [0732 N10 AI 只读视角会话与命令配额原型](0732-native-n10-ai-session-bounded-pool.md) —— 命令配额原型；AI 有界工作池尚未实现
- [0733 N11 权威服务第一批——认证 actor 与版本化字节存档](0733-native-n11-auth-bytes-archive.md) —— 字节/会话方法已有；令牌复用与解析边界待修
- [0734 N11 服务层续批——重连/乱序/去重会话层回归](0734-native-n11-session-layer-reconnect-dedup.md)
- [0735 N12 跨进程验收——存档落盘、跨进程恢复与同哈希续跑](0735-native-n12-cross-process-archive.md) —— 本轮同宿主 H25/H35 比较通过；未跨平台
- [0736 官方武将迁移——司马懿（鬼才 + 反馈）](0736-native-simayi-guicai-fankui.md)
- [0737 DamageData 携牌字段与 N12 旧线对照可行性评估](0737-native-damage-card-field-oldline-feasibility.md)
- [0738 官方武将迁移续批——曹操（奸雄）与郭嘉（遗计）](0738-native-caocao-jianxiong-guojia-jiji.md)
- [0739 官方武将迁移续批——貂蝉（闭月）与甄姬（洛神）](0739-native-diaochan-biyue-zhenji-luoshen.md)
- [0740 官方武将迁移续批——关羽（武圣）与赵云（龙胆）view-as 转化语义](0740-native-wusheng-longdan-viewas.md)
- [0741 N10 自动化文本宿主示例与 submit 延迟](0741-native-n10-native-table-latency.md) —— 仅 submit 耗时；文本/自动化示例
- [0742 TRAITOR 终局锚点重选与旧线安装/构建打通](0742-native-traitor-anchor-oldline-install.md)
- [0743 N12 旧线无头驱动器——环境尝试与边界记录](0743-native-n12-headless-driver-attempt.md) —— 历史环境判断已由 0765 更正为回环可达
- [0744 官方武将迁移续批——吕布（无双）与黄月英（集智）](0744-native-lvbu-wushuang-huangyueying-jizhi.md)
- [0745 N08 官方迁移补全——吕布无双的决斗部分](0745-native-wushuang-duel-second-sha.md)
- [0746 N11 主动技能命令面——GameAction::ActivateSkill 与黄盖苦肉](0746-native-n11-active-skill-command.md)
- [0747 N11 内容续批——孙权「制衡」参数化主动技](0747-native-sunquan-zhiheng-parameterized-active.md)
- [0748 官方武将迁移续批——周瑜（英姿）](0748-native-zhouyu-yingzi.md)
- [0749 官方武将迁移续批——孙尚香（枭姬）与华佗（青囊）](0749-native-xiaoji-qingnang.md)
- [0750 官方武将迁移续批——吕蒙（克己）](0750-native-lvmeng-keji.md)
- [0751 官方武将迁移——大乔（国色）与吕蒙（克己）](0751-native-daqiao-guose-declared-kind.md)
- [0752 官方武将迁移续批——甘宁（奇袭）与张飞（咆哮）](0752-native-qixi-paoxiao.md)
- [0753 N11 大模式设计——国战与 KOF 的引擎级实现路径](0753-native-n11-guozhan-kof-mode-design.md) —— 模式目标与里程碑；KOF 淘汰/会话持久化仍缺
- [0754 N11 大模式实现第一批——KOF M1 与国战 M1 数据模型](0754-native-kof-m1-guozhan-data-model.md)
- [0755 N11 国战 M2 势力模型与 KOF M2 会话层](0755-native-n11-faction-kof-m2.md) —— 势力映射及独立两人局计分；完整会话未验收
- [0756 国战 M3 势力胜负判定](0756-native-guozhan-m3-faction-victory.md)
- [0757 国战明置门控与 M3 回归矩阵](0757-native-guozhan-reveal-gating.md)
- [0758 N08 官方迁移第五十九批——刚烈/突袭/裸衣](0758-native-n08-ganglie-tuxi-luoyi.md)
- [0759 N08 官方迁移第六十批——诸葛亮/陆逊](0759-native-n08-zhugeliang-luxun.md)
- [0760 N08 官方迁移第六十一批——刘备/马超（标准包收尾）](0760-native-n08-liubei-machao.md)
- [0761 N10 表现事件层与渲染投影](0761-native-n10-presentation-render.md)
- [0762 国战补偿明置时机](0762-native-guozhan-compensatory-reveal.md)
- [0763 N08 挂账简化项收尾——顺手牵羊/连营弃牌路径/激将直结算](0763-native-n08-snatch-lianying-jijiang.md) —— 三项切片交付；完整标准技能集与简化仍有缺项
- [0764 阶段 2 宿主装配器与渲染器的引擎侧原型](0764-native-stage2-host-prototypes.md) —— 将槽元数据与文本渲染原型；未装配实际技能/体力
- [0765 N12 旧线无头驱动器——回环贯通与服务端要件更正](0765-native-n12-headless-driver-loopback-breakthrough.md)
- [0768 P1 缺口收口：N10-a/N11-a/N11-b/N04-f 修复与 N08 技能登记](0768-native-p1-gap-closeout.md) —— 隐私日志/令牌/头部长度原反例已修；缺牌快照、prompt 总量和技能规则仍待验收
- [0769 P1/N11 将领装配与 KOF 会话](0769-native-n11-general-assembly-kof.md) —— 明置装配、KOF 将池/轮换已实现；六技能漏接和空池解析旧反例已修，超时判负与实际续跑待修
- [0770 P1/N08 六技能——跨玩家响应机构建完成](0770-native-n08-six-skills.md) —— 六技能机器已注册并接武将表；护驾权限局部修复，队列/多闪与规则差异仍开放
- [0771 P2/N05、N06 复合状态验收扩展](0771-native-p2-n05-n06-composite-acceptance.md) —— 局部恢复/身份轨迹测试通过，流离内存/字节恢复旧反例已修；完整事件与实际中间恢复对照待补
- [0772 P2/N10 策略类型隔离、全程计时与有界任务调度](0772-native-p2-n10-policy-planner.md) —— PolicyInput 与同步预算规划已实现；提交序号旧反例已修，隐藏信息、模拟序号和候选反例开放，CPU 计时不含帧提交
- [0773 P2/N09、N12 同负载证据与对照边界](0773-native-p2-n09-n12-load-evidence.md) —— 新线日志已存；本轮补缺失索引，同负载/跨目标未验，全部完成结论更正
- [0774 P1 缺陷表收口——护驾隐私/规划信息集/流离分歧/KOF 解析/无洞校验/恢复映射](0774-native-p1-defect-table-closeout.md) —— 六技能接线/流离内存与字节/KOF 空池、提交高水位等局部修复确认；余下无懈/选择恢复、缺牌、AI 与护驾组合反例开放
- [0775 性别门控与事件摘要补强](0775-native-p2-n08-n05n06-closeout.md) —— 性别元数据/过滤已落地，恢复派生未同步；六技能/中间续跑测试未齐；E01–E06 不按 v3.2 剥离
- [0776 E01–E06 扩展运行时基础与验收](0776-native-e01-e06-extension-runtime.md) —— 内存注册、临时卡牌列表、基础名冲突 panic 和元数据检查；真实对局/保存恢复及路线图 E04–E06 仍待接入
- [0777 C0 生产侧装载器择址 content-native-loader](0777-native-c0-native-loader.md) —— 桥接实现从 8 份测试 bridge 收敛为宿主层 crate；名称走符号表 intern（内存界验收）；择址与依赖方向决策。`PackSpec` 输入与名单一次性分配已由 0778 取代
- [0778 原生扩展包数据键表与自描述装载（C0 续）](0778-native-c0-pack-data-keys.md) —— 引擎提供牌种/宿主行为/开局钩子数据键与牌种伴随技能表（A0 测试锁定）；cards.json 自带包元数据、射程、装备技，装载器无逐包参数；严格校验；武将技能名单走符号表。其余 6 份 bridge 迁移待做
- [0779 名称符号表总量上限与拒装策略（不回收）](0779-native-symbol-table-limit.md) —— `SYMBOL_TABLE_LIMIT=1e6`，装载器三入口超限拒装（`SymbolTableLimit`）；回收因 `'static` 贯穿目录/存档而不取；单包内逼近上限登记为已知边界
- [0766 N12 旧线驱动器收尾批——两次进局与旧线自身缺陷定界](0766-native-n12-driver-closeout-oldline-defects.md)
- [0767 N12 旧线浏览器单局完局记录与统计边界](0767-native-n12-driver-complete.md) —— 浏览器完局历史记录；UI 操作/终态数量不等于 n09 同口径统计
- [0702 M7 移动端阻塞说明与替代方案](0702-m7-mobile-blocking.md) —— 历史 ADR：原 M7 移动壳决策已被性能架构原生平台路线取代；配套 phase28 快照已清理（2026-10-11 补录索引，T16⑤ 附带）
- [0780 卡牌实例受控合成原语（CardMod 伴随表）](0780-card-instance-mod-primitive.md) —— 机关术批：card_mods 伴随表建模合成（目录不可变），名称合成/数值升级登记差异；存档 v45
- [0781 可变座位环原语（swapSeat）](0781-mutable-seat-ring.md) —— GameStateCore.seat_ring 显式座位环，座位语义全经环访问器；存档 v46；长蛇阵换座/破阵诀随机重排落地
- [0782 扩展执行后端与旧 JS 兼容范围（D2/T8 定案）](0782-extension-backend-and-legacy-js-scope.md) —— 旧 JS 不承诺零改动运行；内置 HostBehavior + 扩展包 rule-vm 分层；QuickJS 兼容层不立项、WASM 暂缓；草稿未证明项转 T8 集成验收
- [0783 Boss 关卡流程子系统（D4）](0783-boss-stage-flow-subsystem.md) —— 十殿还人/隐藏链八技/翼轮次/bianshen 换 Boss 进原生流程面；与 D5 龙血·ylyg*、映种、庞统外部技划界；实现分 B0–B4
- [0784 桌面端原生执行引擎原型（PA09 草案）](0784-desktop-native-engine-prototype.md) —— 草案：引擎在 Tauri Rust 进程原生运行、沿用 Worker 协议语义，以 PA08 数据为对照组；原型未落地，是否切换默认由用户拍板。
- [0785 原生 GPU 牌桌原型（PA11 草案）](0785-wgpu-table-prototype.md) —— 草案：PA08 实测 DOM 余量充足、性能动机不成立，原型若做只做单场景 wgpu 对照；投入由用户拍板，拍板前不立项。
- [0786 VM 效果级选择窗挂起与主动技桥（PA12 §2 J + 批⑥ 草案）](0786-vm-effect-choice-window-and-active-bridge.md) —— 草案：提交期挂起（commit_effect 开窗 + MachineSnapshot::Rvm 追加提交期字段 + 复用 Answer 通道）与 active_index→ActivateSkill 桥；J1/J2 分批建议。VM 提交协议、选择/命令协议与任何升号实施前须分别请示（裁决⑤）。
