# E01/G0 LX.2+LX.3 执行：龙血玄黄动态采样与清单定案（2026-10-08）

[立项](../2026-10-08-e01-longxue-survey/README.md) 的 LX.2（隔离动态
采样）与 LX.3（清单定案/接口分配）完成。方法：记录型 Proxy 桩
（lib/ui/get/ai 深层写路径记录、game 调用记录、document 桩、反篡改链
良性方法桩）在 VM 内执行 `precontent`/`content` 两次（`config={}` 与
`config={shengjian:true}`），采样脚本
[lx2_trace.js](lx2_trace.js)、原始记录
[behavior-samples.txt](behavior-samples.txt)。

## 动态采样结果（与 LX.1 静态清单交叉印证）

| 采样 | 行为 |
|---|---|
| 每次 content 必做 | 反篡改检查（lib.version.includes('诗') / lib.changeLog.some）→ **注册 `lib.skill.mbqianlong`**（乾龙：audio 6、forced/locked/forever、init、derivation 引用外部技能） |
| `config.shengjian` 开 | 额外注册 **`lib.skill._CMeffects`**（forced + charlotte；trigger global gameStart/dieAfter——特效元素清理管理技）+ CSS 注入路径 |
| 未采样到 | `lib.character`/`lib.translate`/`lib.card` **零写入**——本包不注册武将/牌（files 空数组佐证） |

## LX.1 静态清单的两处修正（交叉印证价值）

1. `mbcmqingzheng`/`mbcmjiushi`/`mbjuejin`/`mbcmfangzhu` **不是本包注册
   的技能**——它们只出现在 `hasSkill('…')` 守卫与 mbqianlong.derivation
   引用中（外部依赖：手杀曹髦本体包/其他扩展）；静态面误列为注册项。
2. `mb_caomao`/`mb_caomao2` 同为 `game.me.name` **期望检查**而非本包
   注册的武将——「不能据文件名认定只有界面功能」核查完毕：本包规则面
   仅 2 技能，主体为特效/UI。

## LX.3 定案（清单定案与接口分配）

- **规则面（若迁移，走 G1 转换包/内容宿主路径）**：
  `mbqianlong`（含 init 运行时挂接与阶段态 _1/_2/_3）、
  `_CMeffects`（门控管理技）。触发语义的逐技能对齐未做——如立项迁移
  需另按 P2/N08 逐技对照口径执行。
- **界面面（宿主界面扩展能力，不迁内核）**：专属剑特效 CSS（@keyframes
  动画）、cmarea DOM 管理、updateProgress 加载进度。
- **依赖登记**：mb_caomao/mb_caomao2 武将与四个 mbcm* 技能由外部提供
  （本包不自给）。
- 范围自此明确（此前「范围未明前不估完成率」解除）：体量=2 技能+纯
  UI；是否迁移与验收入口按台账 G1/G4 流程另行分配。
- **残余**：战斗内技能触发轨迹（需带对局的旧线采样）未采——注册期
  行为已完整；如迁移规则面再补。

## 采样边界

VM 隔离（断网语义、stub 环境、Proxy 记录）；扩展正文仅被调用一次
content×2 采样，未进入构建链。反篡改检查以良性桩应答（includes/some
返回 undefined），不影响注册行为采样。
