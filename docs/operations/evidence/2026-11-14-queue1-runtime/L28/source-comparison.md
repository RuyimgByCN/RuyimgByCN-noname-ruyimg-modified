# L28 三技能源码对照与实跑边界（2026-11-14）

本文件由采集者 `/root/queue0_independent_review` 编写，留待另一人独立复核。采集开始 HEAD 为 `605201f3f49e10cea61c31d52a61bbb66c85599d`（有主 agent 的 L29 改动），续行 HEAD 为 `5c73d1d9158145e33f98f2018013914667d13964`；[11 个输入](publication-boundary-inputs.json) 的字节均未变。规则 333／NGSA 107／schema 5／ABI 759／WS8。本批只新增证据，没有修改产品、清单或版本，不新增等价签署。

上游运行的是自有静态服务中的原版 liuli `1.10.1.0.1`，不是现代核心适配。只安装原始技能属性及实际调用的 UI helper 属性；[切片行号与哈希](upstream/source-slices.json) 可与 [原始 extension.js 快照](inputs/upstream/archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js) 重算。没有执行整个扩展的 content/precontent，没有读取或加载官方／上游打包的 Spine 运行库，没有增加假卡。太平要术的真实卡牌属性来自原文件 8968–9016 行，其 onLose 本轮未执行。

## 补叙 `th_buxu`

| 原始规则位置与条件 | 实际原生落点 | 本轮核验 |
| --- | --- | --- |
| extension.js:6372；phaseUse；`getStat('skill').th_buxu + 1` 张 he 牌，承业展开区不足 6 张 | `l28_batch_a.rs:202` 按 turn_stamp 递增成本；`:558` 用六经 mask 与 he 数量准入 | 初始 3 手牌；第一次弃 ID0，第二次弃 ID30/45；空手与 6 经均无可用技能。实跑只覆盖同一 phaseUse 内递增，额外出牌阶段的统计重置未实跑。 |
| extension.js:6382 起；选弃 he 牌可在确认前取消；支付后进入自定义六经按钮 | `l28_batch_a_asks.rs:169` 先问弃牌，`:175` 再问缺失经类，后者不允许取消 | 上游经真实 phaseUse、技能处理器和选牌处理器支付；经类使用 CDP 鼠标点击实际 `.th-mamidibu`。初次 6 个按钮，第二次不含已补的基本牌类。原生 Tag6 被拒绝，DOM 中也无该按钮。取消成本时双方无弃牌、无六经。 |
| extension.js:6186–6201 定义六类：伤害锦囊、基本、无懈、无中、乐不思蜀、装备；:6431 选择 `get.cardPile(filter)` | `l28_batch_a_asks.rs:702–751`：仅按所选 Tag 置 mask，直接 `deck.pop()` | 同一初态：基本类双方取得 ID1 杀；装备类上游 ID79 绝影、原生 ID1 杀；无懈类上游 ID70 无懈、原生 ID1 杀。再次发动的装备选择也留下不同展开区与牌堆。 |
| extension.js:6435、:6479 起：搜不到指定牌时减回使用统计；超时选择随机缺经 | 原生选定后直接增加使用统计和 mask，不按类别搜索，无搜不到的统计回退 | 搜不到与超时本轮未等待实跑，保留为源码差异／未验分支，不能把正常基本类样本外推到此。 |

原生恢复覆盖每次成本、经类以及非法经类尝试；二次发动共四个挂起点。原生 `th_liujing` 与上游承业 gaintag 展开区按实际卡身份对照，未把所选经类 mask 当成真实牌类别。

## 整经 `th_zhengjing`

| 原始规则位置与条件 | 实际原生落点 | 本轮核验 |
| --- | --- | --- |
| extension.js:6018–6035；phaseUse、usable:1；首次 filter 设置 `zhengjingxiao` 后调用 `dcdAnim.loadSpine` | `l28_batch_a.rs:552` 允许激活，次数由既有宿主元数据控制；`l28_batch_a_asks.rs:585` 开数量选择 | 四个独立 fresh 页面都由真实 phaseUse 的 chooseToUse 检查进入原始 filter，首次即 `ReferenceError: dcdAnim is not defined`。每页仅一次入口；没有利用 flag 已置 true 的副作用重试，没有伪造小游戏结果。未授技能案正常无可用技能。 |
| extension.js:6050–6077：牌堆取不同名字且非毒的牌；身份模式通常随机 3–5 张；之后雷霆切果小游戏 | `l28_batch_a_asks.rs:589–614`：人工选 0–5，正数直接取连续顶部 N 张 | 原生 count3 实际取 ID0/1/2，全部是杀；上游源码明确不同牌名，不能产生这组整理出的经典。小游戏未实跑，不把人工数量选择称为对应 DOM 输入。 |
| extension.js:6096–6108：小游戏无成功结果则结束；有结果才入处理区、选强制目标 | 原生 count0 分支设置 boom、置 1 张经并摸 1 张牌；取消数量由共同取消路径结束 | count0 原生实际把 ID0 置经并摸 ID1；数量取消无牌变动。原始小游戏的无结果分支不能由本轮浏览器完成；仅源码对照，未称跨引擎复现。原始 `thunderFruit` 的 result 生成在 extension.js:494–539。 |
| extension.js:6110–6152：强制目标；可把任意非空经典子集置于目标，其余自己获得；仅一张时走单张分支 | `l28_batch_a_asks.rs:615–626`：固定第一张置经，其余全部归发动者，无分配问题 | 原生 one-self、three-target 两分支及目标挂起恢复完成；上游受前述入口阻塞，目标与分配 DOM 未执行。固定“一张”不能表达原规则全部非空子集。 |
| extension.js:6162–6181：目标准备阶段取回所有经，跳判定／摸牌并去技能 | `l28_batch_a_asks.rs:629–644`：收回 th_jing，设置跳阶段标志并移除技能 | 本轮没有推进至后续准备阶段；这部分只读源码，不宣称运行通过。 |

此环境未加载 Spine 运行库，`dcdAnim` 不存在是隔离采集的实际阻塞，不能推论原版完整安装的雷霆扩展也坏。后续 `player.thunderFruit` helper 与资源初始化亦未安装或执行。依用户限制，本轮在此停止浏览器入口，改用逐行源码对照；没有 stub、提前置旗或替换小游戏。

## 业炎 `th_yeyan`

| 原始规则位置与条件 | 实际原生落点 | 本轮核验 |
| --- | --- | --- |
| extension.js:8636 起：phaseUse、limited；原始 DOM 有三团火；同一目标超过一团须四花色手牌 | `l28_batch_b_asks.rs:209–239`：模式 0/1、目标选择、四牌选择 | 四花色为 ID0/7/21/24；缺四花色时模式 1 被拒绝，真实 DOM 第二团不能加到同一目标；已消耗限定技双方均不可用。全体目标都为真实玩家区域，包括自己，但本轮未拖到自己。 |
| extension.js:8672–8813：真实 mousedown/move/up 分配火焰；头像监听器逐团撤销；chooseCard 再确认 | `l28_batch_b_asks.rs:337–381`：通过既有宿主选择挂起结算 | 小一目标、小三目标、大一目标、大双目标完成；保留拖拽坐标、每步 DOM 火数、自动选四牌、消耗状态。成本自动选满时上游只有 OK；撤回三团再取消有效。原生三处取消均不消耗限定技，未因 step 函数内空列表分支猜测取消失败。 |
| extension.js:8888–8899：存在目标且确认成功才弃四牌、条件性 loseHp(3)、awaken；否则结束 | `l28_batch_b_asks.rs:383–418`：大模式必 emit LoseHp(3)，再置使用标志与逐目标 Damage | 原版 game.js:249–268 的真实 `getStrength(true,false)`：localStorage `strengthen_skill=null` 返回 false，`forbid` 返回 true。默认配置的大业炎两案上游发动者保留 8HP，原生变 5HP；双方均弃同四牌且消耗限定技。没有改写该 helper。 |
| extension.js:8889、:8902：先支付体力，再按座次对每目标按实际火数伤害 | `l28_batch_b_asks.rs:405–418` 按座次 emit，但实际日志顺序与声明顺序不同 | `forbid` 配置大双目标：上游 globalHistory/changeHp 为 owner loseHp3→目标1伤1→目标3伤2；原生全日志为目标3伤2→目标1伤1→owner HpLost3。小三目标亦为上游 1/2/3 与原生 3/2/1。伤害 source 均实际为 owner0，nature 均 Fire；不是仅比较 HP。 |
| extension.js:8902：使用的火团数可以只为 2，单一目标伤害 2 | 原生大模式单目标固定 3，大模式双目标固定 2+1，模式 0 每人 1 | `yeyan-great-two` 真实只拖两团并确认，确实伤目标1两点并付四牌／体力。原生没有对应火数选项；表中配最近“大模式单目标”只用于展示不可表达，明确不计同输入通过。 |

大业炎支付序列与目标分配本轮通过初始私有手牌 DOM 排序对齐：这是游戏输入前的 fixture 设置，避免原始 directgain 排序使自动选牌顺序不同。完整初态区域／牌堆顺序、名称、花色、点数、性质进入对照；无修改结算中的卡或火数。原生与上游的事件流结构和 RNG 未作整体等价比较。

## 测试与未验范围

读取了真实既有测试 `l28_thunder_eight_match.rs:578–661` 与 `l28_thunder_eight_batch_b.rs:482–533`。前者检查整经固定一张、补叙 mask／成本与恢复，未断言补到的牌类别；后者检查小／大业炎最终 HP、四花色门禁、限定次数与取消，未断言原版 strengthen 配置、实际伤害顺序或先失体力。本批没有重新执行这些 Cargo 测试；执行的是自有正式锁包 probe。

原生最终 22 案、39 次挂起恢复；每次编码保存 NGSA 原档，经正式相同锁包解码恢复，比较完整快照（仅 auth_generation 归一）、状态哈希、相同回答结果及新增日志。所有命令／编译输出、原始全日志、逐次回答后状态和原档均保留。本批不覆盖所有互相触发技能、死亡／连环／伤害替换、额外阶段、计时超时、AI 隐藏信息、全扩展配置和包安装卸载，也不把有界 ChoiceSpec 的存在当作规则等价。
