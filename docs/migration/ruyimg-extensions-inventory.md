# Noname-ruyimg-Modified 扩展清点（ADR 0605）

> 2026-09-23 新范围：用户指定的是 `archive/upstream/noname` 与 `archive/upstream/noname-liuli` 的全部扩展，当前 8 + 29 = 37 个来源实例。
> 本页 44 项是另一个 fork 的历史快照，不能替代当前清单；“非内容/不可迁移/全部处理完毕”不适用于本次原生目标。
> 以 [两仓逐项重构计划](upstream-extension-reconstruction.md) 的功能、依赖、组合与验收为准，以下保留旧线证据。

> 2026-09-22 更正（09-23 起由上段范围说明取代）：本页 44 项数量未重盘；“非内容/不可迁移/全部处理完毕”不能作为原生线豁免或完成依据。同名能力的适配与验收在两仓台账登记，扩展能力边界见 [原生扩展机制](../architecture/native-extensions.md)。

> 目标：`archive/Noname-ruyimg-Modified/apps/core/extension` 下扩展迁移到本项目。
> 审计快照：2026-08-05。共 **44** 个扩展目录。结论：**无新可迁移内容**——
> 5 个为上游重复（已由既有包全覆盖），39 个为非内容扩展（模式/配置/UI/皮肤/工具）。

## A. 上游重复（5）——内容已迁移，仅资产差异

与 `archive/upstream/noname/apps/core/extension` 同名扩展**代码逐字节一致**
（diff 排除 image/audio 后 0 差异），仅图片/音频资产不同（本 fork 使用替换版
立绘/卡图——表现层，按约定不进入 npack）。

| fork 扩展 | 已覆盖包 | 说明 |
|---|---|---|
| 3D精选 | `extensions/ddd` | 31 将/66 技能/31 立绘已投放（fork 立绘为替换版） |
| 杀海拾遗 | `extensions/shahai` | 332 将/1092 技能/41 装备卡已投放 |
| 欢乐卡牌 | `extensions/huanle` | 5 装备卡/5 技能已投放 |
| 玩点论杀 | `extensions/wandianlunsha` | 9 将/3 卡已投放 |
| 英雄杀 | `extensions/yxs` | 43 将/111 技能已投放 |

## B. 非内容扩展（39）——登记不可迁移

按类别（模式脚本 / 配置 / UI / 皮肤 / 工具——新引擎不执行旧 JS，模式未实现）：

| 类别 | 扩展 |
|---|---|
| 游戏模式 | boss、boss_liuli、cardpile、cardpile_liuli、coin、coin_liuli、wuxing、wuxing_liuli、武皇争霸（mightyFoe）、末日乱斗（僵尸模式，角色为 mode_extension 门控 + forbidai/unseen）、太虚幻境（侍从/收集/赛季系统，操作既有角色） |
| 配置/设置 | 祖安武将（祖安设置：补丁 key_ 包 + 元气系统配置）、AI优化、SJ Settings、电脑适配、更新保护、段位制裁、手杀全扩（改名/皮肤映射） |
| UI | 十周年UI（27k 文件）、手杀ui、斗转星移、斗转露头、点绛唇（Bp）、全能搜索、拖拽读取、史诗卡牌（cardPhantom 卡面特效 + reguhuo/rangjie 技能补丁）、龙血玄黄（混淆 JS + 专属技能区 UI）、一将成名（招募/结算/背包系统，850 文件 302M） |
| 皮肤/动画/音频 | 千幻聆音（58k 文件）、无名美化（技能动画补丁，修改既有技能如 tamo）、如真似幻、如真重置版、皮肤切换、标记补充、手杀MVP、选将美化、雷霆万钧、笨蛋插件、identity（身份图标） |

## C. fork 角色层新内容（2026-08-05：jiange 为 `ABI-COORDINATION.md` 领取表 0606，无 ADR 文件；shoulijian/gwent 见 [ADR 0607](../adr/0607-huanle-shoulijian-abi642-gwent.md)）

- **jiange（剑阁）包**（`apps/core/character/jiange`，源自 Liuli M4）：8 将/15 技能
  已迁移投放 `extensions/jiange`（jizhen 激阵、sfanshi 反噬真实运行时，其余元数据
  登记；新神速/八阵复用 shenhua，奇雾为上游引用遗留占位）。
- gujian/gwent/hearth/mtg/ddd 为上游重复（已迁移）；bingshi/mobile 等 skill.js
  差异为音频/表现层 + 小逻辑微调（不改变已迁移 DSL 语义合同，不重迁移）；card/
  新文件（thunder_card/太虚幻境/欢乐卡牌）为空代理（内容由扩展 precontent 加载，
  模式类）。

## D. 清点结论

`Noname-ruyimg-Modified/apps/core/extension` 全部 44 个扩展处理完毕：内容型 5 个
（上游重复）由既有包覆盖；非内容型 39 个按类别登记不可迁移（模式/配置/UI/皮肤/
工具，旧 JavaScript 不进入 npack 或运行时）。

## E. 静态资源迁移（2026-08-05，ADR 0609/0610/0611，C 段 omp）

用户指示静态资源（图片/语音）也须迁入本项目，反转此前「表现层豁免」约定：

- **fork 5 内容扩展资产随包挂接**（ADR 0609）：yxs 44（43 立绘 + 1 音频）、
  huanle 16（6 卡图 + 10 语音）、wandianlunsha 13（3 卡图 + 9 立绘 + 1 语音）、
  ddd 31（fork 版立绘替换）、shahai 577（573 图 + 4 语音）；角色经
  `portrait_asset` 挂接，卡图/语音入包由 UI 按命名约定引用。
- **上游立绘 2,396 张挂接**（ADR 0610）：24 ported 包 2,366 + standard 30/30
  （`std_` 前缀回退）；非 ASCII 文件名经 `url_safe_asset_id` 变换（12 处修正）；
  25 包 logic_hash 逐一 SAME-HASH 核验。
- **上游音频 7,831 份拷贝**（ADR 0611）：skill 6,343 + die 1,486 + card 2，
  按技能名/武将名/卡名归属入 29 包 `assets/audio/`；voice/effect/background
  与基础牌语音（引擎域）登记不迁移。
- 全部 31 资产包重打包部署；32 包空 store 冷导入
  （content_lock=ee263cfb…）+ `real_extensions_lock` 门禁 1/1 + conformance 8/8
  通过。

