# C331 私有phase2／phase3检查点（2026-10-30）

这里保全私有实现的阶段结果与独立审查，尚未接入主树。基线`38ab1678df9b4b7b991d3b3122b1d937fd2648a1`；C331／限定包schema4按裁决22／26实施，format1、API1、ABI758、archive105、WS6保持。源码本体及完整产品向量仍在原私有检查点；本目录是有界证据，不冒完整产品验收包。

## Phase2

[交接](phase2-handoff.raw.txt)、[核心复审](phase2-core-review.raw.txt)、[schema复审](phase2-schema-review.raw.txt)针对124源冻结。K1窗口权威／深度、K3明置授技、S1 ABI校验、S2 maxHP声明均关闭；K2原turn／座位变化拒档已修，但新owners截短可被恢复接受，复审判新增P1。K4当时仅通过loader拒绝不安全DamageBefore多handler组合关闭silent-wrong-runtime问题，功能仍缺。

[原聚合日志](phase2/affected-full-final.log)为2887 passed／1 failed／76 ignored，native三包1733/1/75；未通过项为保留330 expected的golden。父完整Rust Debug解析及记录hash探针源码／日志绑定核查保存在[parent-phase2-golden-audit](parent-phase2-golden-audit/object-differences.json)：12场仅版本，suidun另仅预算4075→4076，rollback另仅一个TriggerOrder park；完整events／choices相等、RNG全字段不变。实际54个座位view，旧56报告保留并明确纠正。复审接受这些轴的归因并允许更新331 expected；未重新执行hash探针。

## Phase3

[最终交接](phase3-handoff.raw.txt)、[133源清单](phase3/final-freeze3/source-freeze.json)及[父冻结身份](phase3/parent-freeze-identity.json)绑定133/133字节一致、相对phase2增量25路径。manifest SHA `32c3b156ebdffa6d9f1c167e0c855d46c05352ccbe4a3afd1c9a6245c7ec60fa`，原完整tar SHA `d70b34974cceefb6a2ad040ced87375abd6ad7dfb998542d0bbca92ab18a1137`；tar不在本有界目录内。

owners截断P1有[真实红](phase3/owners-red.log)／[定向绿](phase3/owners-green.log)。为保留真实boss fellow API的增玩家行为，未发布tag83内部增加`normal_owners`入窗前缀，完整owners覆盖当前玩家库存；新玩家没有当前窗Normal桶，First／Last按仍存在全局桶处理，不回退已过stage。上游依据为`gameEvent.ts addTrigger`已有doing map匹配，不凭新增库存创造普通触发桶。codec／hash／restore、正式与host增玩家回调同步；source审查仍在进行，未据作者结果宣P1最终关闭。

trigger-only DamageBefore多handler现在沿既有frame的实际window身份贯穿filter／start／following／恢复，active组合旧不支持边界保留；定向红绿记录与[golden新增字段归因](phase3/golden-audit/phase2-to-phase3.json)封存。相对phase2完整对象仅rollback park增加`normal_owners=4`，events／choices／54views不变，更新后golden定向通过。新增8个古剑食品来源证据含紫阳丹silent元数据红→绿；[来源状态](phase3/host-source-status.json)为200/290实际分支已取证，90未证，不能以216条来源行代替290条触发分支。

作者交付的实际Tauri IPC、WebSocket／actor／WAL／10秒timeout、WASM及旧读者报告分别保存在[产品汇总](phase3/desktop-ipc-result.json)、[串行专项](phase3/server-final-serial.log)、[平台命令](phase3/platform-commands.json)和[旧读者](phase3/old-reader-phase3.json)。误用旧test binary筛出0 tests的记录另留。完整18MB IPC回复、21MB WS线报和Native对照向量未复制到本目录；其完整对象及数据身份仍待独立产品证据审查，不以汇总JSON单独证明全部比较通过。

**最后全量仍红**：[十包原结果](phase3/affected-final.log)2892 passed／5 failed／76 ignored，失败为新增WS专项并行压力下10秒fixture提前进入managed链。最终仅把非计时fixture调至合法90秒，实际timer样本仍10秒，随后串行专项5/5通过；尚未在该最终源码重跑全量，不能覆盖原五红。默认并行重跑已另安排，结果未到不提前记绿。手动先赢后旧deadline实际到达样本、未知Skill的真实旧WAL读者及90来源仍待。

## Phase3独立审查补充

[核心复审](phase3-core-review.raw.txt)关闭原owners截断P1，保留trigger-only Rvm改为NO_FRAME／Intrinsic槽位可绕过window绑定的P1；已交作者做真实codec反例与最小修复。[Normal边界补充](phase3-normal-boundary-correction.raw.txt)正式撤回原4→5指控：窗前／窗后加入fellow的两条合法历史可保留不同显式成员边界，不能无可信历史依据要求restore拒绝其中一种；现有库存／前缀／队列结构校验保持，不新增第二权威。

[产品复审](phase3-product-review.raw.txt)认可限定实测资格，指出timeout回放把实际Answer统一重建为TimeoutAction的P2；作者继续改为原WAL动作逐条提交，旧报告不改写。父随后[完整解析桌面回复](parent-phase3-ipc-audit/independent-results.json)：545条记录、268份完整archive、261份投影／choice／hash、256份结果与events后缀、五非法请求均与Native向量对应，并核最终binary及现有WASM文件SHA。未对缺少Native expected的legal_actions作独立比较、事件label未重译、closed-resource抛错没有record而由driver及ID缺口支持，二进制到源码的构建来源仍引用冻结／argv记录，不以hash单独证明可复现构建。父首次把save响应kind误写NGSA的解析错误与初始脚本保留，实际kind为save，未重跑产品。

[归档映射](archive-mapping.json)保留首批原件SHA、路径及存储编码；上述后续review／解析文件为明确补充。这里的源码审查、定向执行、产品运行、上游等价和全量验收分别登记；C331整体未完成，改命332仍须等待实际331验收。
