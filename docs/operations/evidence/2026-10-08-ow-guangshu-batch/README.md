# U05 ow 纵切批二：虚无之光·光书（规则 v119，2026-10-08）

ow 纵切批二：**虚无之光·光书**（guangshu，ow_zhixuzhiguang 单技能+4 花
色子技）——出牌阶段弃一张牌并指定一名角色，按弃牌花色授对应子技：♥
下次受伤后回 1 / ♦下次造成伤害后摸 2 / ♣至下阶段结束不能用杀 / ♠下阶
段结束受 1 无源雷伤（各一次性移除；已有任意子技者不可再指定）。前置：
[ow 士兵76](../2026-10-08-ow-shibing-slice/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js guangshu + guangshu_{heart,spade,club,diamond}）

- 主技：enable phaseUse、filterTarget 四子技同查（已有者不可指定）、
  content `target.addSkill("guangshu_"+suit)`。
- ♥：trigger damageEnd forced → recover(1)+removeSkill；♦：trigger
  source damageEnd forced → draw(2)+remove；♣：mod cardEnabled 杀
  false + trigger phaseEnd forced removeSkill；♠：trigger phaseEnd
  forced → damage(thunder, nosource)+remove。

## 改动（分开陈述）

- **代码已实现**：`WudianGuangshu`（tag 107）三段机（弃 he 一张 → 目
  标（四子技同查排除）→ 按 suit 授子技 SkillDefId 实例——SkillDef 经
  cards 携带）+ 4 子行为（tag 108–112）：♥/♦ Damage 触发（filter
  target/source==owner && 非 prevented 伤害>0）触发臂 recover 1 / draw
  2 + 实例移除；♣ 杀门控钩（PlayCard Sha 臂查 Club 实例拒绝——同类
  碧日浓雾先例）+ PhaseEnd 触发臂移除；♠ **commit_phase PhaseEnd 段直
  推结算**（无 PhaseEnd 帧——同水没墓穴/筑城口径）emit 1 无源雷伤+移
  除。包 cards.json：虚无之光 guangshu mapping + 4 子技 SkillDef 注册
  条目（mapping 通道注册使 skill_by_name 可达），三哈希重算。
- **登记差异**：♠上游"下个结束阶段"实为任一阶段结束即触发（trigger
  phaseEnd 逐阶段）——原生同口径；无专用 mark 日志（intro UI 不迁
  移）。
- **转换包**：cards.json 增 虚无之光（generals）+ 4 子技注册，三哈希
  重算。

## 验收口径

- **回归已加入**：`zhixuzhiguang_guangshu_suit_marks`——♠光塔全链：弃
  黑桃 → 指定 1 号（子技授予断言）→ 下阶段结束受 1 雷伤（4→3）。
- **全量通过**：engine-native **437 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 118→119 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（光书不在标准场景，纯版本位移）。
- **剩余**：ow 其余 22 武将、xianjian 36 武将逐武将纵切；U07
  duoqi/youzhan 暂缓；zhenfa 五种阵法。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
