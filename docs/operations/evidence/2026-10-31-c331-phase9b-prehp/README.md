# C9b：真实 pre-HP driver，最终仍 BLOCK

核验2026-10-31，私有5fc+C8+C9叠加，Main无C生产合入；331／106 private／758／WS6。最终46增量manifest e0ef75ea3d3c0a7a77bb2103db36e5b9e23e1f2d146286716d4c971a62c3a174，完整212 manifest d1c89c4cd195e0dc6847bcffbfb3d01f52f6aa6e086ad7649451f14329ded049。见[实现报告](implementation.raw.txt)、[原索引](artifact-index.json)及[父审计](parent-audit.json)。

实现既有栈内单一DamageFlow及真实Start→Before→Begin1…4→HP提交→Target／Dying→Source／End／After／Chain；SubmitHp为持久root进度和原子receipt／armor／HP转换，不是新增可订阅事件。chain资格在自身HP转换捕获，仅解当前受害者，后续逐recipient检查alive／linked。

初版共享构造误使非Damage Equipped增加BeforeTriggers，原1790/1/75保留；最终scope-fix只共享Damage／Window构造，其他旧入口保持，新增第19项回归通过。最终自然完成原生1791/1/75，唯一golden组12场不符；独立Gate1b0、source9030／unit2／layering0。parent-owned-validation保存已排队full→strict→cheap的原执行结果；父接管既有队列、无PID信号，完成后两PID退出，无新writer竞态。

[独立复审](review-two-P1.raw.txt)仍BLOCK两P1：native inline先于First VM handler，恢复子窗自身stage可伪造Intrinsic／BeforeTriggers。原C9三结构问题大部分修复不等同完整Damage正确；golden未重录。下一C9c修复及真实C8逐场语义处置另见[后续证据](../2026-10-31-c331-phase9c-dispositions/README.md)。

本目录保存原源码tar／patch／命令／失败／警告及父后补队列结果；旧结果与最终scope-fix结果明确分开，未宣C331／Main／上游规则验收。
