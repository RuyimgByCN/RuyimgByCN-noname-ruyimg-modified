# C331 C9f：原子规则修复、链式出生校验与剩余阻断

核验日期：2026-10-31。生产代码基线仍为34e／rules330／NGSA105／ABI758／WS6；其后9f4b7423为纯文档提交。本目录仅保留私有331/106分支的源码、测试及复审证据，没有将其导入Main，也没有重录golden。

[原始实现报告](implementation.raw.txt)和[独立复审](independent-review-BLOCK.raw.txt)的整体结论均为 **BLOCK**。复审在所检范围确认链式出生反例及11组原子问题修复正确，F14仅部分修复，五组真实调用／选择／判定／牌后续行仍必做；全部12项provider／生命周期／分组／异步未知也仍保留。

## 来源与执行记录

- [19文件增量](original/final-freeze2/source-manifest.json)：`f19c2f6209f1eb9719cb13884e601b8a680a0bc2d66dd4e4715bdf73d4cca7da`。
- [225文件组合](original/full-source/source-manifest.json)：`ab88c526c5cb4e807a29c5bdaaa70393b4515375303980d153403047f479be96`。该目录是源码overlay；未列文件使用实际不可变5fc Git对象，不将“overlay缺文件”误当新文件。
- [最终结果](original/results-bound.json)、[七命令及完整225源码绑定](original/command-source-bindings.json)、[原始输出](original/checks-final2/)：定向87项通过，其中24项新增；202组JSON／完整snapshot／archive捕获；全量kernel/content/facade **1845通过／1失败／75忽略、exit101**，唯一失败为golden组。独立目标 `RUSTFLAGS=-Dwarnings cargo check --offline -p engine-native --all-targets` 通过，source9036、门禁单测2、layering0。
- [父独立核验](parent/noname-parent-c9f-final-audit.json)检查4,167份本轮产物、225/19源码与tar／patch／live字节、各命令的源及日志绑定，同时再次核对旧C9e全部7,333产物未变。原始checkpoint18的基线核对曾把overlay当完整仓库、把新增文件的None误当空文件哈希；[第一次父诊断](parent/noname-parent-c9f-checkpoint-audit.attempt1.json)保留，后以225前身219 overlay加不可变5fc真实对象完成核验，不作产品失败。

链式子事件在Start拒绝仍位于父remaining队列中的目标，并保留合法出生正控及typed/byte坏输入原子拒绝。F01/F05/F17修复实际自来源分支；F02沿真实相邻传播边解析effective Sha/use_as，Zuijiu仍只允许直接Sha父；F03在Begin1取消自身伤害；F04共用已准入资格并排除已命中；F06全宽255/256/257扣减及真实溢出命令回滚；F08真实he随机弃牌及装备撤销／loss／discard子树；F10保留terra、随机反击及既有全局回合边界到期；F11消耗火／雷随机流并使用真实受害技能owner；F15检查其他实际启用Duqu定义。它们的测试通过不扩大到未测分支。

F14已实现Boss／双队Versus／Doudizhu／标准Identity的来源规则及实际主槽、存活、具名Mushengoumang抽牌资格。固定上游无side的标准Identity把其他玩家视为enemy，不能替换为隐藏身份AI阵营判断。原 `boss_xiangliu` 非golden断言错误地期待额外Shedu授予，固定源码只授Zhongdu并执行关联抽牌；原红和修正原因保留。

## 仍必修的行为和证据缺口

[全部17组与12未证项处置](original/all17-findings-and12-unproved-dispositions.json)没有移出任何范围：

- **F14 P1**：不同已明置魏／蜀、unknown／ye以及 `yexinjia_friend` 的真实身份／存储规则仍缺失。`isEnemyOf == !isFriendOf` 的固定来源并不支持对这些已知敌对情形直接返回None；不得以全局阵营取反、座位猜测或“可接受差异”关闭。
- **F07/F09/F12/F13/F16**：Qinglong真实卡牌选择／取消；Zhangwu可选变长he成本及等待loss；Yuance两选项／取消与普通判定改判；Jiding等待完整虚拟杀及闪响应后移除实际被准入定义并保留分组恢复来源；Jidian真实目标／判定与自然嵌套触发，不能保留假递归静音旗标。
- 复审要求补充合法后续改量chain恢复、完整首／次／第三次Zlfanshi与expiry/renew、明确断言Abi火与雷结果、混合he Yinzei及撤销／loss顺序、启停Duqu别名、死亡／副槽Mushengoumang负控，以及虚拟Sha Shuchui首次和第二次资格。测试源码中计算 `_nature` 而未断言，不能写成两分支均已证明。
- 真实SkillInvocation Before／Begin／body／End／After及logSkill子事件、普通Judge/Tiandao、共享provider生命周期、真正ChangeHp和DeathFlow仍须实现。续行方案已限定复审，见[下一批调用契约](../2026-10-31-c331-invocation-plan/README.md)，不能用方案替代运行验收。

## 新增P2与golden边界

[原始golden处置](original/golden-business-and-outcome-dispositions.json)及比较器保留复制来的 `final-freeze4/source-manifest.json` 引用，而本轮真实冻结目录是 `final-freeze2`。独立复审当轮将其登记为P2，要求另立索引追加正确的225/19关联并修正下一比较器模板，**不改写本轮封存文件**。后续C9g以[独立索引的追加更正](../2026-10-31-c331-phase9g-invocations/original/golden-dispositions-c9f-reference-correction.json)仅替换12处路径；父审计及[C9g源码复审](../2026-10-31-c331-phase9g-invocations/independent-review-BLOCK.raw.txt)接纳此引用P2关闭。原e3703abf…文件、比较器、索引及业务捕获保持，引用更正不关闭其余源码P1或golden失败。

原C8正向生产者没有重跑。362组配对仍为11场六个已观察业务面相同、chain场先记录伤害再执行自身解链的真实顺序差异；187边界无返回、139 Accepted的争议32字节哈希单列、32 Pending完整载荷与4 Rejected完整原因相同。没有未知归一化，没有任意内部fuel／上游全规则／隐私正证据结论，也未授权重录12个golden期望。

后续[C9g239／46封存及原样验证](../2026-10-31-c331-phase9g-invocations/README.md)已完成，最终源码可编译但全原生1848/9/76失败，复审仍BLOCK；原始当轮未编译状态、所有编译红和旧期望保持。当前同一写入者继续C9h修复并补齐实际控制。262/290、剩28及全部37来源范围不因本轮绿测增加；整体仍未完成，见[进度主稿](../../../architecture/native-progress.md)。本目录最初归档复制映射保存在[发布后映射](../2026-10-31-post-c9e-doc-publication/post-release-archive-map.json)，后续README更新以新增带时间戳工作区收据界定。
