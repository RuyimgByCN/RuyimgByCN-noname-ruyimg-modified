# T12 swd 武将面批㉒：尚章/游兆/王思月/宇文拓

- 日期：2026-10-15
- 范围：四将转正（三整将+一部分将）——**21/79 名**（上批 17 将+4）；新增 HostBehavior tag 317-320
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{character.js,skill.js,translate.js}`（lianwu/yuchen/xielei/duishi 逐段核验）

## 四将语义与接线

| 将 | 技 | 接线 |
|---|---|---|
| swd尚章 (shu hp4) | 连舞 lianwu（锁定） | SwdLianwu(tag 317)=marker：engine PlayCard max_targets 链 +1（杀目标上限 1→2）+kernel poyue_direct_hit 臂同位查询（红杀 directHit 不可闪避）——双臂皆复用既有查询型先例（疾影/破月） |
| swd游兆 (shu hp4) | 龙胆（builtin 白名单）+浴尘 yuchen | SwdYuchen(tag 318)：双事件（CardUsed+CardResponded via extra_events，罗美先例）+回合外门+黑色花色门；Act 选目标→RNG 代弃 he 一张（装备随 revoke） |
| swd王思月 (wei hp3) | 对诗 duishi+闭月（builtin 白名单） | SwdDuishi(tag 320)=ActivateSkill 通道 usable:1：Act 弃牌→SelectCard 指定目标→TargetChoice 同花色弃牌窗（空选=拒绝）→CardsAsk 消费/随机获得面 |
| swd宇文拓 (shu hp4) | 无双（builtin 白名单）+挟雷 xielei+昆仑镜（deferred） | SwdXielei(tag 319)：双事件+>2 人局+杀类门；Act 弃牌（可取消）→TargetChoice（use 排除原目标）→1 雷伤 |

## 登记差异

1. **对诗**：上游 duishi2/duishi3 临时技（获牌后自失效/目标被弃后防重）由 usable:1 收敛为每回合一次；获得面随机代弃（上游弃牌者自选，浴尘同款 RNG 代选登记）。
2. **挟雷**：respond 路径不排除杀来源（CardResponded 载荷无来源字段——上游 trigger.source 排除面缺失登记）。
3. **浴尘**：随机代弃（上游 discardPlayerCard 自选）。
4. **昆仑镜**（宇文拓第三技）：unique+video+group 双子技复杂面 deferred（mapping null），挂后续批。

## builtin 白名单扩容

`BUILTIN_SKILL_REFS` 4→7：+龙胆/无双/闭月（content-native-loader lib.rs:332）——批 23 通道延续。

## 验证

- **定向回归** `swd_generals_b63_lianwu_yuchen_duishi_xielei`：连舞红杀双目标+直击（p1 有闪不可避、p3 无闪受创）+三目标拒反例；浴尘龙胆黑杀响应链（respond 路径双事件）+随机弃牌；挟雷弃牌→排除原目标→雷伤+取消反例；对诗同花色弃牌面+限一次反例+拒绝→随机获得面+闭月白名单授予断言。
- **全量**：engine-native 775 passed / 0 failed（本批无 TurnState/快照/哈希面改动——golden 零扰动全绿）。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0。
- 定向中勘误（入记忆）：对诗首版 SelectCard 首步读应答=active_ask 前移铁律第 8 犯（四段 stage 流重排 Act→SelectCard→TargetChoice→CardsAsk）；挟雷反例撞杀限一次（新回合再布）。

## 测试布阵勘误

- 目录无黑闪实体（标准闪全红）——浴尘黑色响应面改用龙胆黑杀打出（CardResponded kind=Sha ✓ 黑色 ✓）。
- 连舞双目标用 [1,3]（同距 1），对座 p2 距 2 越界会混入距离拒绝。
- settle_optional_confirm 会吞掉整条确认链——需逐步应答的窗口（挟雷）禁用 settle 先行。

## 结论

- **21/79 名 41+4=45 技名**（upstream 79 将 183 技）；整将累计十九（+3：尚章/游兆/王思月）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
