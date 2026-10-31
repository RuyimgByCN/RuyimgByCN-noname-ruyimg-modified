# C331 phase9伤害流程部分封存：BLOCK（2026-10-31）

私有5fc+C7+C8入场修复的42文件增量/191组合，331/NGSA106(private)/ABI758/WS6。**未接纳C9，禁止Main合入**；源排序262/290、剩28、+0保持。[实现报告](implementation.raw.txt)、[独立语义triage](independent-triage.raw.txt)、[父完整性审计](parent-audit.json)与[组合源清单](original/full-source/source-manifest.json)分别记录实现、问题和哈希，不互相替代。

已实现canonical signed amount/独立HP loss及receipt、post-HP普通Source/End/After child、target-before-Dying、完整Dying后逐子chain、codec/hash/root引用；九项定向真实通过。全原生仍1774/8/75、exit101，独立Gate1b与行数/分层通过不能代替全量绿。原始取消/目标顺序红、fixture错误、编译红与全部golden保持。

独立审查确认三P1：Flow仍在HP/旧target队列之后启动，Before/Begin1..4未形成真正可挂起的前置driver；chain资格/解链使用晚期board状态，偏离changeHp资格记录及逐目标重查；restore未严格限制settlement/outcome/progress/continuation有效矩阵。全部需在实际consumer修，不能靠DTO或删断言。

八失败包括Feiren链量、协律朱羽、楚江王死亡、A0变体枚举、灵躯零量、旧chain、立即Dying死亡及golden组；golden原日志实际有12个scenario差异。triage逐项区分携带canonical量与重施modifier、缺linked前提、较早可选target窗、sample顺序假设及合法zero终止。未无因重录hash或回退正确target-before-Dying。

后续phase9b正在同一私有writer实现已批前置driver/callback归位、chain与restore修复。本次下发任务把SubmitHp与child windows连写不精确，主管已明确更正：**SubmitHp是root持久进度和一次原子receipt/armor/HP transition，不发明damageSubmitHp可订阅事件**；完成既有changeHP children后再Target child。合同v2/v3优先，无新裁决/版本/ABI/query。

父fresh核261证据SHA、42增量tar与191组合SHA并解析全量8红，仅证明封存完整；原代码/证据均保留。未证明全部Damage生命周期、所有native来源回调、provider/生命周期/死亡/救援机制、完整伪造域、C331产品及全37。当前状态见[进度主稿](../../../architecture/native-progress.md)。
