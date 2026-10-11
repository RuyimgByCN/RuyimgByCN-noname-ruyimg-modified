# 2026-10-11 T11 四象系朱雀首片：焚天（规则 v236）＋朱雀条目

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（四象系朱雀首刀；延续本会话批次链）。上游依据：`mode/boss.js` `boss_fentian`（6900：trigger damageBegin1 非火→hasNature("fire")；mod cardUsable 红=Infinity/targetInRange 红=true/wuxieRespondable 红+player!=target=false；group boss_fentian2 useCard 红→directHit 全场除己）；`boss_zhuque`（1008：female/shen/hp4/skills `[boss_shenyi, boss_fentian, boss_chiyan2]`）。

## 建模

- **焚天**（`BOSS_FENTIAN`；SkillDef 复用 `ZhaoHuoBehavior`+`DamageBefore` TriggerMeta）：
  - **火改写面**：ZhaoHuoBehavior 同构复用（数据less nature 改写机器不携带技能身份，快照变体共用 `BossZhaohuo`，恢复路径按技能表重挂——登记同构复用）。
  - **红牌免次数**：杀限豁免（`fentian_red` 入 sha 限 reject 条件）+酒限豁免（`fentian_red_jiu`）。
  - **红牌免距离**：sha_range_free 链追加（单/多目标共用）+顺手牵羊距离校验 `fentian_red_free` 臂。
  - **红杀无视防具（directHit）**：kernel 八卦阵判定步 `fentian_direct_hit` 短路（原生防具=八卦阵；红桃判定不再发起）。
- **朱雀条目**：`BOSS_GENERALS` 尾追加（def 索引 12，female）`[神裔, 焚天]`。
- **赤炎2（boss_chiyan2）**：未落随下批。

## 登记差异

- wuxieRespondable（红色牌禁他人无懈、仅目标可懈）未挂钩——原生 nullify 询问过滤未接入（红色锦囊 directHit 面随之空缺，同门登记）。
- 红酒免次数面：目录无实体红酒（酒=黑桃3/梅花9），面仅经声明转化（红实体视作酒）可达——代码在位、以登记代测。
- 势力 shen→群目录近似（同白虎/烛阴先例）；上游 female 保留。

## 坑位实录

- **行为触发总线只扫 `trigger: Some(meta)` 技能**（resolution.rs:285 `let Some(meta) = def.trigger else { continue }`）——焚天首版 def `trigger: None` 挂 ZhaoHuoBehavior 静默失效（火改写不触发）；补 `DamageBefore` TriggerMeta 后转绿。教训：**复用带行为的 SkillDef 必须连 TriggerMeta 一起抄**，`behavior` 字段本身不进触发总线。
- 回归②：杀被受理后目标持闪会挂起 Pending——拒收（Rejected）才不发闪询问，断言先排 Rejected 再答 Pass。
- 红酒面无实体可测（见登记）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_zhuque_fentian_red_cards` | 五段：①普杀毕方伤害视为火→免疫反证（hp 不变）②红杀无次数+无距离（连打两张、第二张隔位 3 号；3 号 Pass 后命中）③黑杀同回合仍受限 ④红酒面登记说明+守恒 ⑤红杀跳八卦阵判定（无 ArmorJudged 日志）、黑杀照常判定（有日志）+焚天实例往返+守恒 |
| 既有回归 | boss 全滤 32/32；机械审计：无多重注册/零属性清零 |
| 版本递增探针 | 235→236 全 14 位移；回退 235 复跑 **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 236 全量重录 2/2 绿 |
| 全量 | **582 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | exit 0 |
| 版本 | 规则 235→**236**；存档不变 v53（快照变体复用 BossZhaohuo，无新字段） |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批：朱雀续片（boss_chiyan2 赤炎/boss_chiyan3）或玄武首片（boss_lingqu 灵渠/boss_xuanlin 玄麟系；boss_xingqiu 星囚）。
