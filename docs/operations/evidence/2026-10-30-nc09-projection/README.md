# NC09 裁决第15项：可见牌面、区域与武将定义投影（2026-10-30）

整合基准为 F7 已验收提交 `2b3348ef`，整合前工作树干净；[起点](integration-start.json)。本批是裁决第15项的投影部分，**不包含超时执行，也不表示 NC09 整项完成**。规则328、存档105、原生ABI753、WS5保持不变；没有新增消息、选择类型或协议字段。

## 代码与信任边界

- 服务端从经过权限过滤的原生 `PlayerView` 输出本人手牌、全场装备/判定/弃牌/处理区，以及这些可见牌的真实 suit/rank/def/kind；归属座位明确从原生0基转换为协议1基。牌种沿用现有六类和Other，不另造分类映射。
- 仅服务端赋予的 Player 身份可以选取对应私有座位。Spectator 即使附有效座位仍取公共视图；0、257、u32::MAX不会经整数截断进入其他玩家座位。武将定义跟随既有暗将可见性进入 GeneralView，再经本地WASM JSON/Worker传入现有character_def。
- 独立审查发现首版误将 Referee 视为 Spectator，压掉原来有权看到的暗将定义；该版本没有合入主树。修正先红（0而非22）后绿，仅 trusted Referee 恢复权威主将定义，Player/Spectator 仍过滤。真实WS伪造role/observer/seat不能提升角色；可信保存成员的恢复/重连保留原角色及令牌权限。
- **旧裁判缺口仍未完成**：原生裁判还缺所有手牌、牌堆顺序、私有场外/技能牌区、隐藏身份及完整选择细节的统一全信息投影。这是未实现能力，不能改写为裁判与旁观者同权限，也没有测试要求这些遗漏继续存在。

## 红绿回归与独立审查

[最初实现报告](initial-report.txt)、[首次审查](independent-review.txt)、[修正报告](correction-report.txt)和[修正复审](correction-review.txt)保留原文。复审结论 OK with notes，未发现新增问题；旧裁判遗漏和超时仍明确待办。

原始命令/失败保留在 [initial/](initial/) 与 [review-fix/](review-fix/)。其中首轮全量的6个旧WS断言错误没有被后续通过覆盖；修正只将“全部牌必须属于自己”改为本人Hand断言，并为公共区域另检真实牌面、无外人手牌/牌堆。

隔离源码已有原生1644/0/73、WASM72/0、服务端首版369/0及修正最终372/0、真实WS15项及新增防提权、前端128/0、Worker82/0。修正后没有重复未变化的原生/WASM测试；父线程实际整合重跑见[integrated-results.json](integrated-results.json)：三包原生+WASM+服务端 **2088/0/73**、当前源码WASM重建、前端128/0/typecheck、真实Worker82/0、同档真实WASM32的40次比较/32次隐私检查、两场跨目标轨迹均exit0，完整preflight **21/21**，执行期间受验源码指纹不变。17个补丁文件与隔离树逐字一致，Cargo.lock另保留已验收F7三包，只增本批server开发依赖；见[逐文件核对](integrated-source-conservation.json)。原始隔离失败与后续通过分开保留。主树独立三包严格警告门禁亦[通过](integrated-gate1b-result.json)，使用已结束NC09专用warning缓存，没有清理共享target。

## 同一存档的实际 WASM32 对照

共享fixture从实际引擎导出5种状态：身份局公共各区、死亡身份、暗将、明将及标准目录所有牌种。服务端导出每座位快照，Node初始化实际生成的WASM32、恢复同一archive两次，并经现有Worker适配比较所有牌面、归属、区域、武将定义与身份；另外根据只在测试fixture中的权威ID做私有手牌/牌堆负检，不把这些ID加进网络。

隔离结果40次精确比较、32次非空隐私检查、5种状态、0失败；同一checker将服务端suit故意改null后失败，证明能识别牌面差异。主树会重新导出fixture、从当前F7+投影源码重建WASM再跑同矩阵，不能用原生执行WASM封装的Rust测试替代真正WASM32。

## 超时后续与兼容

实际actor的30秒反例到31秒仍无应答，红灯保留在初始证据。本批没有添加无消费者的设置、倒计时、托管字段、timer或半接线命令。J2专供v329；超时拟在J2提交后另请示v329→v330，预计保持当时ABI和存档105，如确需存档升号仍另裁决。该方案不等于v330已授权。

WS5现有CardView可选suit/rank、区域与kind及SeatView.character_def均可承载新值；本批编解码回归验证完整载荷，旧schema可解析，没有新增tag或枚举。WASM内部characterDef为现有适配字段补值，旧本地产物保留fallback0；正式验收使用本批新构建。

尚未宣称：完整裁判支持、超时、所有目标平台、上游规则等价、37来源、整体收敛验收。主稿/台账按整合实际结果回填。
