# 0448 onlyOL 可表达性评估与缺口登记（Hermes E 段）

日期：2026-08-03
状态：已落地（评估 + 登记，无技能交付）
领取者：E (Hermes)

## 背景

onlyOL 上游顶层 176 个技能,已定义 23 个,缺失 157 个。本批对全部缺失技能做了系统性
可表达性评估(触发/启用/继承/强制/长度特征 + 精读抽样),结论:纯 DSL 可保真表达的技能
极少,绝大多数依赖引擎缺失能力。

## 评估结论

| 缺口 | 代表技能 |
|---|---|
| 技能继承 inherit | olenyuan1/2、olganlu、olchengxiang、olchaozheng、olzhenlie、olqianxi 等 |
| 装备技 equipSkill | sizhaojian_skill、olzhiti2 |
| expansion 技能牌堆取牌/交换 | dengji、oljuxiang、chuyuan、olzhuiyi、olsbxutu |
| 通用当前伤害修改 | oljiaojin、olzhiti、olchenshi、olguifu |
| 本回合未造成伤害查询 | olrumo |
| 交换装备区 swapEquip | olganlu |

同批评估了 key（缺 200）与 yijiang（缺 170,其中大量为父技能已移植的折叠子技能）,
缺口清单已合并登记到 ABI-COORDINATION.md「E 段原语缺口登记」小节。

## 后续

按 ABI-COORDINATION.md 规则,推进 onlyOL/key/yijiang 前先在 E 段（560 起）逐项领取
对应原语,再改 engine-api 并更新 ABI-MAX。本批零新 ABI、零技能交付。
