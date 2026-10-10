# U07 技能批五：车阵（田豫）（规则 v115，2026-10-08）

玩点论杀武将技能声明批五：**车阵**（chezhen）映射
`host:wudian_chezhen`——锁定距离技：有装备时你算到他人 -1；目标无装
备时其被算到 +1（computed_distance 直读 skills 表，权限型 Noop）。
未映射声明 5→4（夺启暂缓）。剩余候选：绝道（横置+横置距离，需连环/
距离双向钩评估）、割据（杀范围势力计数）、诱战（重流程）。前置：
[U07 技能批四](../2026-10-08-u07-skill-batch4/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/玩点论杀/character/skill.js:408）

- `chezhen`：纯 mod 双向距离——globalFrom：from 有装备 → distance-1；
  globalTo：to 无装备 → distance+1。无触发无 content（锁定状态技）。

## 改动（分开陈述）

- **代码已实现**：`WudianChezhen`（tag 102，权限型 Noop——距离钩在
  `computed_distance`：`computed_distance` 内查 (a, def)/(b, def) 实
  例 → a 有装备 minus+1 / b 无装备 plus+1；mapping 通道定义键
  `WUDIAN_CHEZHEN_KEY="chezhen"`）。包 cards.json：田豫 chezhen
  mapping（skills 段登记），三哈希重算。
- **登记差异**：无（纯数据钩语义等价；距离下限 1 饱和为既有路径）。
- **转换包**：三哈希重算（cards.json 更新）。

## 验收口径

- **回归已加入**：`tianyu_chezhen_distance_mod`（5 人局对座避邻座下限
  饱和）——验证一：0 号有装备（关=攻侧-1+守侧+1 抵消=2、开=攻侧
  -1=1）→ 验证二：2→1 不受 0 号技影响（基线 1）→ 验证三：青釭转注
  1 号 → 0 号卸装备全部加成消失（0→2 回基线 2）。
- **全量通过**：engine-native **432 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 114→115 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（车阵不在标准场景，纯版本位移）。
- **剩余**：U07 未映射 4 条（夺启暂缓 + 绝道/割据/诱战）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（17 文件，相对 noname-next 根）。
