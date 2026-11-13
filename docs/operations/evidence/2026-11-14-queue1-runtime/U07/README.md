# U07 三技能真实运行样本（2026-11-14）

最终输入为 native09 与 browser twelfth：24行对应24个唯一case，原生和上游各实际运行24案，43个原生待答窗口均保存原始NGSA、编码/解码恢复，并核对同答后的state_hash和新增日志一致。24案的所选初态投影通过；终态投影12案相同、12案不同。本轮不签新增规则等价。

HEAD为 `605201f3f49e10cea61c31d52a61bbb66c85599d`，dirty；规则333／NGSA107／schema5／ABI759／WS8。执行期间主agent有酒诗恢复定义、L29测试及公共文档/构建产物变动，完整dirty输出保留在输入快照；所捕获478个原生/loader/上游输入文件在最终原生运行前后、浏览器运行前后均无哈希变化。原生编译时第479项probe二进制变化符合预期，浏览器期间该二进制也未变。

[逐case对照表](comparison-formal.md)、[逐字段比较](comparison-formal.json)、[精确原始输入索引](final-input-index.json)是最终入口。索引逐案列出native/browser JSONL行号、原始CDP夹具请求及结算请求/回复行号、源文件和日志哈希，没有将重复尝试静默去重。[最终范围](final-scope.json)保留全部浏览器轮次的成功/失败记录数。

| 技能 | 实际覆盖 | 当前实际差别 |
| --- | --- | --- |
| 筹度 | 目标1/2、手牌/已装备成本、首张/异面展示牌、放回牌堆顶、选成本/选目标取消、无he牌 | 上游询问全体有手牌角色并允许选入展示牌、最后放回牌堆顶；原生只问所选角色换出牌，固定拿展示首牌，余牌全弃。全部后续都取消时可得到相同终态投影，选择空间仍不同。 |
| 复仇/复仇2 | 交手牌/已装备牌、取消交牌、无he牌；phaseAfter初手无杀及初手有备用杀的实际拒绝/使用 | 交牌取消原杀后，上游原杀弃置，原生额外返还原杀。上游回合末需实际出杀或自身失HP；原生自动虚拟杀。无装备的末尾隔离案中，已有备用杀的拒绝/使用都实际出现chooseToUse。 |
| 诱战 | 手牌装备/已装备成本、取消、距离2、无装备、反击杀命中、反击杀被闪后再拒绝 | 支付成本后上游取消原杀；原生仍让原目标额外承受原杀伤害。反击被闪后，上游再次询问出杀，原生没有相应循环。 |

正式包没有调兵遣将实体牌：[无占位目录输出](native/formal-catalog-without-placeholder.json)记录catalog共138张、匹配实体ID为空、未增加任何卡。上游[原始牌表](../../../../../../archive/upstream/noname/apps/core/extension/玩点论杀/card/list.js#L25)确有红桃4/方块4两张。native09的筹度宿主路径能执行，不证明该实体已注册或可真实使用；因为没有合法实体ID，未捏造一次PlayCard尝试。native01–08手工加占位的轮次原样保留，不作正式包产品可用性的证明。

比较范围是玩家HP/maxHP、存活/横置/翻面/护甲、区域牌ID集、规范化技能名及复仇标记，另比较牌堆ID顺序和弃牌/处理中牌多重集；初态还逐ID核对映射后的name/suit/rank。没有比较完整初态、身份/武将/AI值、跨引擎完整事件/RNG/续行，也没有比较整副牌的性质与语义。原生6张性质杀（87/88雷、94–97火）被浏览器映射为sha而未携带nature；已核对6张都未进入本次实际路径。后缀通用装备/坐骑的映射仅为夹具载入；armor→bagua只作为手牌装备成本，没有防具效果背书。

上游共享服务只读，使用独立Chrome157 profile/CDP/control。每案刷新，等8玩家、sha定义及节点就绪，等待官方onfree/cardsAsync完成后重建夹具；逐个移除虚拟装备/判定，再核对实际区域、技能、状态与牌堆ID。记录中24案都无额外无ID牌追加。实际调用原始技能函数和原始选择事件，只包装choose方法切换手动控制座位，决策通过原UI点击。启动可停在选将，因此[末案截图](browser/final-table.png)仍有原选将对话框及旧UI牌数提示；截图不证明正常产品UI已验收，实际区域/牌堆以原始快照为准。

原生命令和源码：[run09命令](native/run-09-command.json)、[probe09](native/probe-09.rs)、[Cargo清单](native/Cargo.toml)、[锁文件](native/Cargo.lock)、[stdout24行](native/09-formal.log)、[stderr](native/09-formal.stderr)、[43份NGSA及SHA-256](native/archive-manifest-formal.json)。浏览器实际命令为 `U07_NATIVE_LOG=09-formal.log U07_RUN_TAG=twelfth U07_CONTINUE=1 python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/U07/browser/run-cases.py`；[24行原始结果](browser/case-results-twelfth.jsonl)、[运行stdout](browser/twelfth-run.log)、[运行stderr](browser/twelfth-run.stderr)。源码落点与哈希见[最终源码索引](source-map-final.json)。

首次失败和更正均保留。早期native有目录名不匹配、丢失牌ID导致NGSA拒绝、空答不等于Pass等夹具错误；早期browser有序列化循环对象、将动画暂停误当选择、未提供模块全局绑定等失败。第一份汇总把onfree追加尾牌算进牌堆差别，并按case名字误称执行出杀选择；第二份汇总的cancel后缀误取消初次交牌，且末尾装备造成额外效果。两份中间表均不是最终依据，修正见[启动/调度更正](correction-07-initial-review.json)及[分支/目录更正](correction-08-final-fixture.json)。native01–04没有逐次二进制指纹，不作最终版本重放证明；最终native09与browser twelfth有前后源码与二进制指纹。

未跑产品全量门禁、本轮未改产品/清单/公共文档/Git index/refs。未覆盖多复仇标记、来源死亡、筹度同回合二次发动/其他cardEnabled条件、诱战全部距离和其他转化杀等分支。全局门禁及发布由主agent负责；本目录仍属未提交证据。

已停止自己的Chrome并清理约543MB profile及24KB外部probe crate；未创建私有Cargo target、未清共享target，Vite/FS和U06会话未动，详见[清理记录](cleanup.json)。原始最终输入已发独立复核agent，复核结果由其独立记录。

本目录核验（2026-11-14）：执行 `python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/U07/validate-evidence.py`，Markdown本地链接及行号、12条源码引用、最终输入哈希、24+24唯一case与43份NGSA均通过，见[原始输出](doc-evidence-check.log)。未运行全局preflight。
