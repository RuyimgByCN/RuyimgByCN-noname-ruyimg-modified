# C29 genuine ChangeHp：方案v2已审，代码尚未实施

核验2026-10-31；固定Core1a548c95…／Liuli8485dbb…和C9c冻结214组合0ab2b44e…为读者，未读活跃C9d/C9e工作树。原[完整方案v1](guide-v1.raw.txt)及[源码清单](source-inventory-v1.json)保留；其notrigger解释被[独立P1复审](review-v1-BLOCK.raw.txt)否决，不重写旧报告。方案以真实普通ChangeHp child替换HP提交后的inline回调，复用既有栈／队列，持久receipt／progress／cause，覆盖Recover有限续行、LoseHp后Dying及真实qualified helper／named disable reason。

[追加v2](guide-v2.raw.txt)及[补充清单](supplement-v2.json)明确更正：正量damage.notrigger仍执行HP body里的普通changeHp触发和合法callback，即使armor全吸收令HP num0；只抑制源指定generic lifecycle及damage窗口。Core content.ts12118给child._triggered=null，GameEvent.loop仍进body，而12370显式trigger('changeHp')没有被该值禁用。不能传播一个祖先notrigger作为所有子事件禁令，也不新增ordinaryChangeHpDisabled。

父[fresh源核验](parent-source-audit-v2.json)核旧报告／清单／BLOCK审查不变、五源码SHA及完整214 manifest／源相符；[v2独立复审](review-v2-closure.raw.txt)No issues／OK with notes，仅关闭方案P1。未来须真实Shangshix手1→draw3、notrigger正损伤、armor1吸收令num0、zero／prevented对照及time-slice／byte恢复和错误输入原子性；这里只是测试规范，尚未执行或实现。

八boss helper必须用真实group／provider，仅解除对应disable blocker；HLSS First／priority−100／forceDie直接赋HP，不递归伪造Recover；Shangshix loseEnd与changeHp分别保留。31个Recover caller文件要真正await／有限续行，不能只push child后继续draw／ask。maxHP／reset／direct die和真正HP操作分别按源处理，不把一切赋值当changeHp。

实施顺序为C9d seal/review→修复复审Ciqiu窗口／元素杀及Chilongya notLink、Zuijiu直接Sha父上下文、Hutian signed量→真实ChangeHp→已审共享DeathFlow；均在裁决29既有范围，不重请用户授权。保持private331/106/758/WS6及332/106→R05-333/107，不预约新tag／公开SDK／WS／ABI。bdaf只作撤回的历史读身份，当前Main34e与5fc内核／内容未变；无Main C代码合入或I02／37来源完成结论。当前状态见[进度主稿](../../../architecture/native-progress.md)。
