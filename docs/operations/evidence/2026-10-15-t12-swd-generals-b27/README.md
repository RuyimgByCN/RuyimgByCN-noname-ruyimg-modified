# T12 swd 武将面批 27：还侠——屈娴整将收口（viewAs+跟踪回收）（2026-10-15）

## 承载

**还侠 huanxia**（tag 287，键数 286→**287**；目录标记技+三内核钩）：

- viewAs 转化臂：红色牌当【杀】（engine.rs PlayCard 转化链，武圣同族；
  `kind` 重写经 use_as 自动声明）。上游 he 区红牌仅使用面不含响应=登记。
- **跟踪回收**：TurnState.huanxia_track: Vec<(CardId, bool)>（kangyin2 同
  位）——转化提交时入列；commit_damage 置位 caused（上游 expire 子技
  source damageAfter 同口径）；持有者阶段末（End 帧入场）收回未造成伤
  害者（上游 gain 子技 player phaseEnd）。换回合清残留。
- **登记差异**：上游经特殊区暂存牌，原生从弃牌堆取回（牌若已离开弃牌
  堆则不收回）；多次转化逐张独立记账（per-use caused）。

**字段批**：TurnState.huanxia_track 入哈希（kangyin2 同位五处：struct/
init/enc/dec/dec-struct+canonical_hash）——存档 v69 不变（TurnState 原
有编码尾追），规则 v281→**v282**，golden 14 场重录（探针三值互异：旧
golden/281+字段/282+字段——位移来源=版本字段+新字段双源，还侠不在标准
场景可达路径行为面零扰动）。

## 测试（swd_generals_b27_huanxia_viewas_reclaim）

| 面 | 断言 |
|---|---|
| 正例① | 红闪当杀命中（p1 8-1）→ 阶段末不收回（在弃牌堆） |
| 正例② | 红桃当杀被闪避 → Play 末收回红桃到手牌 |
| 反例③ | 撤技后红闪不可当杀（标准牌堆闪全红，黑闪不存在故以撤技面为转化反例） |
| 反例④ | 撤技后红桃照常回血、p1 不受伤 |

测试侧勘误三处：黑闪布阵踩空（标准牌堆闪全红）改撤技反例；③/④ 段落
在 Discard/End 相位（前置 advance_to_play）；Gate 拦未用变量（已修）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **38/38** |
| engine-native 全量 | **757/0/0**（golden v282 14 场）——[日志](engine-native-tests.log) |
| Gate 1b | 首跑拦未用变量（已修）→ 复跑 exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- swd 武将面：**21/79 名、25/241 技转正；整将十六→十七**（屈娴 2/2 新
  晋）。
- 剩余部分转正：壶中仙 2/3（壶境挂卡牌面对账批）、拓跋玉儿 1/3（poyue/
  poyue2 ShaResolve 快照字段批——下一字段批）。
- 挂账不变：swd 卡牌面对账批（92 vs 45）。
- 批 28 候选：poyue 线（拓跋玉儿）或卡牌面对账批。
