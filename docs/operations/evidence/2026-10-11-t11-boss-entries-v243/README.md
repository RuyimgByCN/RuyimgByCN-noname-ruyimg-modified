# 2026-10-11 T11 星宿阶段2系纯入库批：炎帝/少昊双条目（规则 v243）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁延续批次链；星夏后收尾勘察结论的落地）。上游依据：`boss_yandi` 炎帝（姜|轨/shen/hp6/`[boss_shenyi, boss_shenen, boss_chiyi]`）、`boss_shaohao` 少昊（嬴|挚/shen/hp6/`[boss_shenyi, boss_shenen, boss_baiyi]`）——两将技能面全部已覆盖（神裔 v233+神恩光环 v238；赤翼/白翼=青翼同族已处置登记），无新引擎代码。

## 建模

- **炎帝条目**：`BOSS_GENERALS` 尾追加（def 索引 19，hp6）`[神裔, 神恩]`。
- **少昊条目**：`BOSS_GENERALS` 尾追加（def 索引 20，hp6）`[神裔, 神恩]`。
- 神恩光环臂（engine 距离链/hand_limit/Draw）为通用查询——新携带者装配即激活，无新臂。

## 排除记录（勘察结论）

- **共工**（boss_shuishengonggong，姜|共工/hp5）：技能表 `[神裔, boss_juehong 绝红, boss_xuanlin3]`——绝红未落（非纯入库），随绝红批入库。
- **明星**（boss_mingxingzhu，female/hp3）：`[boss_qingzhu 庆祝, boss_jiazu 夹击, boss_jiding 急叮]`——三技未落（星囚授 jiding 面的依赖将），随三技批入库。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_yandi_shaohao_pure_entries` | 五段：①双条目元数据（名/hp6）②装配面（神裔+神恩实例）③神恩光环通用臂（boss 方隔位杀可指定）④挑战者摸牌 +1（清手后=3）⑤神恩实例存档往返+守恒 |
| 既有回归 | boss 全滤 39/39；机械审计：无多重注册/零属性清零 |
| 版本递增探针 | 242→243 全 14 位移；回退 242 复跑 **全绿**——**纯条目不位移 14 场景哈希**（位移仅版本源，条目不入标准场景哈希的新数据点） |
| 14 golden 重录 | 243 全量重录 2/2 绿 |
| 全量 | **589 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | cargo 真实 exit 0 |
| 版本 | 规则 242→**243**；存档不变 v57（无新字段/快照变体） |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批候选：绝红（共工依赖技）/明星三技（庆祝/夹击/急叮）/星宿阶段 2 尾项收尾；相关将随各自内容技入库。
