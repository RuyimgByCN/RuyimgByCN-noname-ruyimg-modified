# U07 技能批六：绝道（刘焉）（规则 v116，2026-10-08）

玩点论杀武将技能声明批六：**绝道**（juedao）映射
`host:wudian_juedao`——出牌阶段横置（自连环，已横置不可发动）；**持技
者**横置时双向距离 +1（上游 mod 挂在技能上：globalFrom 查 from 的技
能、globalTo 查 to 的技能——非持技者的横置不影响距离）。未映射声明
4→3。前置：[U07 技能批五](../2026-10-08-u07-skill-batch5/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/玩点论杀/character/skill.js:252）

- `juedao`：enable phaseUse、filter `isLinked()===false`、content
  `player.link()`（横置）；mod globalFrom（from.isLinked() → +1）/
  globalTo（to.isLinked() → +1）——**mod 属于 juedao 技能**，仅持技者
  参与距离判定时生效。

## 改动（分开陈述）

- **代码已实现**：`WudianJuedao`（tag 103）——activate 守卫（未横置才
  可发动）+ 单步机（emit Linked 翻转，commit_linked 既有原语）+
  `computed_distance` 绝道钩（(a,def)/(b,def) 实例 + linked 双向
  plus+1——**持技者限定**，初版误为「任意横置者」致铁索传导测试距离
  炸裂，已修正为技能作用域）。mapping 接线（刘焉 juedao）。
- **登记差异**：无（content 无弃牌代价——上游 filterCard/check 为转
  换技 AI 残留未挂 content，原生按 content 实现：无代价横置）。
- **转换包**：cards.json 刘焉 juedao mapping（skills 段登记），三哈希
  重算。

## 验收口径

- **回归已加入**：`liuyan_juedao_link_and_distance`——发动横置 → 双向
  对座 2→3 → 已横置拒绝（InvalidChoice）→ 1 号（其回合）横置不影响
  0↔2、0↔1 双横置叠加（1+1+1=3）。
- **全量通过**：engine-native **432 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 115→116 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（绝道不在标准场景，纯版本位移）。
- **剩余**：U07 未映射 3 条（夺启暂缓 + 割据/诱战）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
