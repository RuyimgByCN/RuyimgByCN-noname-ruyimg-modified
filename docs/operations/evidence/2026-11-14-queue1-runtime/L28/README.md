# L28 交互采集（2026-11-14，待独立复核）

采集者 `/root/queue0_independent_review`；此次角色为采集者，不担任本批独立复核。范围为 `th_buxu`、`th_zhengjing`、`th_yeyan`。没有修改产品、共享清单、台账或版本，没有提交、发布或新增等价签署。

初始 HEAD `605201f3f49e10cea61c31d52a61bbb66c85599d` 有主 agent 的 L29 改动；续行 HEAD `5c73d1d9158145e33f98f2018013914667d13964`，本范围 11 个初始源码／包输入均未变化，见 [输入续行核对](publication-boundary-inputs.json)。规则 333／NGSA 107／schema 5／ABI 759／WS8。当前其他 agent 的文档改动原样保留。

## 原始输入

- [中立原始输入索引](raw-input-index.json)：最终 25 个 case 的原生行、浏览器行、每次 CDP 请求／回复行号、源文件快照与 SHA256、61 个 NGSA 原档、真实配置与入口。独立复核者可只读该索引与实际源码，不依赖本说明或比较器。
- [原生 probe](native/probe-02.rs)、[真实 Cargo 命令与环境](native/round2-command.json)、[stdout 全部 22 行](native/round2.jsonl)、[编译 stderr](native/round2.stderr)、[退出结果](native/round2-exit.json)、[正式冻结包](native/round2/formal-frozen-package.json)。使用实际包 validate/freeze → locked bridge → attach → 正式 start/restore，没有增加假实体。调试修改仅用于布置初始玩家／牌区／技能。
- [最终浏览器全部 25 行](browser/round8-clean/case-results.jsonl)、[逐步 CDP 行映射](browser/round8-clean/control-map.jsonl)、[真实事件／DOM 设置](browser/start-case.js)、[输入驱动](browser/run-cases.py)、[启动器](launch.mjs)、[实际服务与 Chrome 元信息](active-session-l28.json)。原始 requests/replies/HTTP 文件的精确名字和哈希在中立索引中。
- [上游原始技能切片](upstream/skills-original.js)、[原始 helper 切片](upstream/helpers-original.js)、[切片源行及哈希](upstream/source-slices.json)、[原始卡牌属性](upstream/card-original.js)。源文件／核心／CSS 的快照也列在索引中。

上游是自有静态 origin `http://127.0.0.1:52217/` 的原版 liuli `1.10.1.0.1`，8 座；不是现代核心适配，也未启用完整雷霆扩展安装流程。实际 helper 与技能属性原样执行，DOM 操作包括补叙按钮的 CDP 点击、业炎 mousedown/move/up 拖拽和头像撤回。没有 stub 或伪造小游戏结果。没有读／加载官方或上游打包的 Spine 运行库；静态服务在读文件前拦截含 spine 的路径，整个 HTTP 原档中此类请求为 0。

## 结果与边界

最终 cohort 为 round8-clean 的全部 25 行，25 个不同 case，不去重：21 行完成，4 行整经在真实 phaseUse 的首次 filter 入口被 `dcdAnim is not defined` 阻塞。完成行同时包含条件不符、取消和实际结算，不把这 21 行都称为技能成功触发。原生为 22 案无 probe_error、39 次挂起编码／解码／锁包恢复；完整快照（仅 auth_generation 归一）、hash、回答结果及新增日志逐次一致，无忽略项。

声明投影下，12 行状态／支付／HP事件顺序相同，8 行有差异，另 1 行为原生无法表达的两火分配，见 [逐场表](comparison.md) 与 [逐字段重算](comparison-results.json)。此数字只属于限定投影，不是整技能等价数。初态比较全部 8 玩家和区域中的卡牌身份、名称、花色、点数、性质及顺序；通用装备的命名映射、私有手牌 DOM 初始排序、全事件流／RNG 未对照范围明确保留。闪电的雷性质由原生 ShanDian kind 与原始 `cardnature` 显式对应，未删除性质字段。

补叙的装备／无懈选择取牌不同；业炎的默认真实 strengthen 配置下体力代价不同，而且即使配置为 `forbid`，支付体力与逐目标伤害的实际顺序也不同。整经受 Spine 边界阻塞，后续数量、去重、分配只逐行读源码。具体条件／选择／结算、既有测试真正断言和未覆盖项见 [源码逐项对照](source-comparison.md)；精确清单键与限定建议见 [findings](findings.json)。清单由主 agent 在独立复核后处理，本目录不写清单。

## 首错与方法修正

- [首次切片选择错误](upstream/extraction-first-error.txt) 与 [卡牌切片界限错误](upstream/card-first-extraction-error.txt) 均保留，修正后按原始行哈希抽取。
- [首轮业炎多按确认的错误](browser/round2/yeyan-small-one.json)：原始 direct 技能已进入后续流程，不存在额外确认；改为真实等待火焰 DOM，不归咎产品。
- [四牌选满后直接取消的原始错误](browser/round6-final/yeyan-great-discard-cancel.json) 保留。原始 UI 此时只有确认，正确取消路径经原始头像监听器撤回火焰后取消；[后续单案](browser/round7-cancel-clear/case-results.jsonl) 与最终全 cohort 均保留。
- [首比较器失败](compare-first.stderr)：遗漏闪电的内置雷性质，初态 assert 立即拦下，未产最终结果。随后加显式来源映射。未运行闪电，不把该映射称为整卡语义证明。
- [排空启动前的比较结果](comparison-second-invalid-fixture.json)、[对应输出](compare-second.stdout) 保留为无效 fixture 诊断：原版 cardsAsync/onfree 延迟在初态后追加原始牌。最终驱动调用／等待真实原始回调，之后才替换牌堆并安装外来牌 MutationObserver；25 个最终初态和所有完成终态均无外来牌。全部此前浏览器轮次都保留在索引，不把其污染终态当规则失败，也不从最终 cohort 删除反例。
- [最终重算 stdout](compare-third.stdout)／[stderr](compare-third.stderr)：退出成功。完整跨引擎事件流、RNG、互相触发技能、死亡／连环／伤害替换、超时、AI、安装卸载和完整扩展配置均未验。未跑共享完整 Cargo／产品门禁；由主 agent 单独登记。

## 回收与冻结

[回收记录](cleanup.json)：停止自有 Node 94370／Chrome 94379，删除自有 142748KiB profile 和临时 probe 目录；源码、Cargo 清单／锁文件、全输出与 NGSA 原档已保存。本批未清共享 target；共享现代服务 5173／FS51215 保持。

目录交给独立复核后冻结；若后续需要补采，须新建带轮次的证据目录，不覆盖这里的原始输入。
