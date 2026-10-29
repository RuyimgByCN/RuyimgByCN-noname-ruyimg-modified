# 2026-10-29 NC08 方案 A 统计·核心武将包扩展 + 逐组 filter 互斥核对（只统计）

范围：[裁决登记](../2026-10-28-user-decisions/README.md)第 14 项——把 arrangeTrigger 命中面统计扩展到**两仓核心武将包**，并逐组核对 filter 是否互斥。**只统计，不改语义/代码/版本**（B 的实施另立 ADR，排在 J1 之后）。开始 [start-time.txt](start-time.txt)。口径与解析器复用 [10-28 扩展面统计](../2026-10-28-nc08-plan-a-statistics/README.md)（[scan_core.py](scan_core.py) import 原模块；liuli 每包单文件，按文件清单扫描）。

## 1. 扫描面与总量

两仓核心武将包 60 包：noname `apps/core/character/` 34 子目录/散件 + noname-liuli `noname/character/` 26 个 .js。**9,178 个触发技 / 2,688 武将**，命中（同武将·同事件·等有效优先级·均非 silent·均有翻译）**48 组 / 33 武将**（54 对；29 组同监听域；24 组含锁定技）。逐包数字见 [scan-core.log](scan-core.log)，明细 [hits-core.json](hits-core.json)。

**合计两轴总命中面：扩展面 9 组（10-28）+ 核心包 48 组 = 57 组**（均为静态上界，未执行 filter/usableSkills）。

## 2. 逐组 filter 互斥核对（[filter-snippets.txt](filter-snippets.txt) 为原始片段）

48 组去重为 22 个技能对（多事件重复计组）。结论分三类：

### 明确互斥（6 组，4 对）

| 组 | 武将（包） | 对 | 互斥依据 |
|---|---|---|---|
| [22] | ol_sb_yuanshao（onlyOL） | olsbhetao / olsbshenli | 事件角色互补：hetao 只在**他人用牌指定本武将**时真、shenli 只在**本人出牌阶段用杀**时真——同一事件实例二者角色互斥 |
| [24] | liucheng（sb） | splveying / spyingwu | 牌种互斥：leying 要求 `card.name==='sha'`、yingwu 要求 `type==='trick'` |
| [36] | xujing（sp） | xjshijian / yuxu | 回合属主互斥：shijian 要求父 phaseUse 属**用牌者**且用牌者≠本人、yuxu 要求父 phaseUse 属**本人** |
| [39] | tw_xiahouen（tw） | twfujian / twjianwei | 有无武器互斥：fujian 要求武器栏空、jianwei 要求武器栏非空 |
| [42]/[43] | dc_ruiji（xianding） | dcliying / dcwangyuan | 触发名闸：wangyuan 在 gain/lose 事件上仅当 `name==='logSkill' && skill==='dcliying'` 才真——与 liying 的原生触发互斥 |
| [37]/[38] | zhugejin（sp） | olhongyuan / olmingzhe | **倾向互斥**（gain/lose 载荷互补：mingzhe 在 gain 事件上读 `getl` 为空恒假、hongyuan 在 lose 事件上读 `getg` 同理；标注「倾向」因静态未跑运行时） |

### 同条件或可同真——**不互斥**（40 组，17 对）

| 对（代表组） | 判定依据 |
|---|---|
| dawu/kuangfeng（[14] 神诸葛 extra）、pekuangfeng/petianlin（[19] offline 神诸葛） | **完全同条件**：两技能 filter 逐字相同（`getExpansions("qixing").length`）——七星牌存在即双双触发，上游必询 |
| sbhongyuan/sbmingzhe（[25-30] sb 诸葛瑾 ×6 事件）、olqiangzhi/olxiantu（[23]/[46] 张松×2）、qiangzhi 同款 | 条件维度不同（充能量 vs 回合外失牌；他人有手牌 vs 他人回合），可同真 |
| clandaojie vs 六个族技（[4-9] 族钟氏 ×6） | daojie=本回合首个非伤害锦囊，对家=首个同花色/副类别/黑色/标记名/第二次同名——「首个锦囊」与「首个♠」可在同一张牌上同时成立 |
| potdianyi/potshequan/potjianzhuan（[0-2] 曹爽） | 存储闸各自登记时机，登记集可含同一时机 |
| remeibu/remumu（[15]/[35] 孙鲁育×2）、oltuoquan/olxianglv（[10-11]）、olshouhun/olsuichong（[12-13]）、lianxi/youlve（[17-18] 蹋顿）、psquanpan/psxiaofan（[20-21]）、olbiluan/relixia（[34] 士燮）、twbaizu/twqiaosi（[40]）、dcshiyu/dcshuaijian（[44]）、maihuo/xinwanyi（[47] 杨芷）、hefeixianjian/hefeizherui（[16]） | 条件维度不同（详见 snippets），存在同时为真的局面 |
| olqiejian（无 filter 锁定直触发）×olnishou（[31-33] 曹丕）、dcpijing（无 filter）×dcsuifu（[41]）、dcjiexing（无 filter）×dcfangdu（[45]） | **一方无 filter 恒真**——不互斥由构造成立 |

## 3. 结论（供 B 方案 ADR 使用）

- 核心包命中面（48 组）显著大于扩展面（9 组），**合计 57 组**；其中明确静态互斥仅 6 组、倾向互斥 2 组，**约 40 组可同真**——上游 chooseControl 的真实询问面以核心包为主，典型如神诸葛「七星→大雾/狂风」（同条件对）与 sb/sp 诸葛瑾「弘援/明哲」族（多事件高频）。
- 24 组含锁定技：上游对锁定技同样询问顺序（arrangeTrigger 只按非 silent 过滤）；B 的选择窗须覆盖锁定技（原生现行全序对这些组全部按 def 序决胜）。
- 10-28 登记的「扩展面 9 组」结论不变；两处均为上界（未执行 filter/usableSkills——本批的互斥核对把上界中的明确不可达组标出）。

## 4. 验证

- 脚本 [scan_core.py](scan_core.py)（复用原解析器；新增方法简写 filter 提取——上游 `filter(event,player){}` 无冒号，键式提取器抓不到，此为扩展面统计未含 filter 核对的原因之一）。
- `python3 scan_core.py` → [scan-core.log](scan-core.log)（exit 0）；产物 [hits-core.json](hits-core.json)、[filter-snippets.txt](filter-snippets.txt)。
- 本批不改任何产品代码、测试、版本；行数门禁与文档检查随会话末统一跑。
