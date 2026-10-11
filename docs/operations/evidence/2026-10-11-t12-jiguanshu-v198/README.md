# 2026-10-11 T12 机关术批：灵剑池 7 张 + 机关妖术补偿触发（规则 v198，存档 v44）

任务来源：[进度主稿 T12](../../../architecture/native-progress.md)（领取范围：**机关术批整批**——灵剑池+补偿触发；机关术×2 本体暂缓登记 T18）；开工 HEAD `8680740a`（领取锁）。上游：`swd/card.js:1259–1545`（灵剑池）、`card.js:1102`（机关术，暂缓）、`skill.js:1883`（jiguanyaoshu_skill）。

## 交付内容

### 1. `CardKind::Hslingjian`（键 `hslingjian`，编码 133 追加式）

七处接线：枚举/ALL 133→134/key/from_key/archive `card_kind_to_num/from_num` 双表/装载器自动（from_key 派生）。

### 2. 灵剑池 7 张（swd 包 cards.json，包 36→43 张）

| 灵剑 | 登记值 | 行为（上游同口径） |
|---|---|---|
| 旋风之刃 | ♦4 | 目标随机弃 1 张 he（装备弃置发弃牌事件） |
| 中行虎甲 | ♥4 | 防具 Kind 实体发现装配+随机弃 1 手牌（目标非 isMin） |
| 星体改造 | ♠6 | 自目标摸 1+手牌上限 -1 累加（`xingtigaizao_count`） |
| 时间回溯 | ♦6 | 目标装备区全部牌入手（装备技随迁撤销） |
| 紧急冷冻 | ♠8 | 目标+2 护甲+翻面+冻结（`jileng_active`：不能使用/不能被指定，翻回解除） |
| 圣休号角 | ♥8 | 目标嘲讽（`chaofeng_active`：相邻角色只能以其为杀目标，其下一阶段开始解除） |
| 隐秘离场 | ♣10 | 目标潜行（`yinmi_active`：不能成为牌的目标，本回合结束解除） |

**登记差异**：灵剑实体花色点数为原生登记值（上游凭空造牌花色随机）；发现池=牌堆+弃堆可见实体（上游全定义池）；星体改造重置时点=下一回合开始（上游上一结束阶段，差一夜）。

### 3. 机关妖术补偿触发补齐（`SwdJiguanyaoGain` tag 234）

防重标记授予同时授补偿实例（CardsDiscarded forced）——非自己回合弃出装备 Kind 牌时随机补获等量灵剑（牌堆+弃堆发现）。**登记差异**：①弃牌事件近似失去装备区牌（替换/获得离去路径不触发）；②发现池同上，无候选不补。旋风之刃的装备弃置分支相应补发弃牌事件（上游 loseEnd 含装备区牌同口径）。

### 4. 状态钩（存档 v44）

PlayerState 尾部追加 `jileng_active`/`chaofeng_active`/`xingtigaizao_count`/`yinmi_active`；`hand_limit()` 读星体改造计数；PlayCard 冷冻自用禁；2248 目标钩扩四态；sha 目标校验嘲讽钩（相邻 seat 判定）；kernel 相位钩（Judge begin 解嘲讽+重置改造计数[登记差异如上]）、翻回块解冷冻。

### 5. 机关术×2 暂缓（**T18 登记**）

运行时卡牌合成/改造原语缺口（lingjianduanzao 名字组合升级数值、lib.card 动态定义）——结构性缺口同类登记（同虚拟用牌缺口）。机关术×2 进 swd cards.json `deferred`。

## 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native --test g1_shahai_swd` | **26/26**（24 既有+4 新增：旋风+时间回溯、紧急冷冻全链、嘲讽/潜行/星体改造、补偿确定性构造） |
| 双版本回退探针（198/44→197/43 复跑 `replay_scenarios`） | 2/2 全绿（恢复后重录 14 哈希+依据注释） |
| `cargo test --offline -p engine-native --no-fail-fast` | **544 通过 / 0 失败 / 1 ignore**（540+4，exit 0）——[完整日志](engine-native-tests.log) |
| `RUSTFLAGS="-D warnings" cargo check --offline -p engine-native --lib`（独立 target 目录，用完删） | exit 0 |

## 边界

- 灵剑/补偿在标准牌堆对局中依赖 deck composition 含 swd 灵剑实体（发现池口径）；补偿在无灵剑实体时不补（登记差异）。
- 冷冻 DamagePrevented 面（急冻同款伤害免疫）**非**本卡语义——紧急冷冻仅禁用+翻面，无免疫（上游同）。
- 机关术×2 见 T18。
