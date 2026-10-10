# U05 ow 纵切批四：守护（天使）（规则 v121，2026-10-08）

ow 纵切批四：**守护**（shouhu，天使双技之二）映射
`host:wudian_shouhu`——出牌阶段弃一张【杀】令一名其他受伤角色回复 1
点体力；持有者不能使用【杀】（锁定禁杀）。天使三技全量接线完成
（shouhu/ziyu 映射 + feiying 暂缓）。前置：
[ow 纵切批三](../2026-10-08-ow-ziyu-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js shouhu）

- `shouhu`：mod cardEnabled（sha → false，锁定禁杀）+ enable
  phaseUse + filterCard {name:"sha"} + filterTarget 其他受伤角色 +
  content `target.recover()`。

## 改动（分开陈述）

- **代码已实现**：`WudianShouhu`（tag 113）三段机（弃手牌杀 → 选其他
  受伤角色 → recover 1）+ **禁杀门控**（PlayCard Sha 臂查持有者实例拒
  绝——碧日浓雾/光井同款先例）+ activate 守卫（手牌有杀才可发动）。
  mapping 接线（天使 shouhu；feiying mapping:null 暂缓——源定义不在
  ow/skill.js）。
- **版本号勘误**：v120 批遗漏提交版本号（常量实为 119）——本批
  119→121 合并跳变并重录（两次语义变化一次入哈希；v121 最终态测试全
  绿为准）。
- **转换包**：cards.json 增 天使（generals，shouhu/ziyu 映射 + feiying
  暂缓 reason）+ shouhu skills 条目，三哈希重算。

## 验收口径

- **回归已加入**：`tianshi_shouhu_discard_heal_and_sha_ban`——①弃杀救
  1 号（2→3、杀已弃）②持有者出杀被拒（守护禁杀）③无杀不可发动。
- **全量通过**：engine-native **439 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 119→121 入 canonical_hash（合并跳变）——14 场景全
  mismatch，重录后 replay_scenarios 2/2。
- **剩余**：ow 其余 20 武将、xianjian 36 武将逐武将纵切；U07
  duoqi/youzhan 暂缓；zhenfa 五种阵法。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
