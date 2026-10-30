# shiji 包遗留技能存档

本目录存放已被父技能取代、退出编译池（rebuild-pack 的 .skill glob 不递归本
目录）的折叠遗留文件（sync 脚本 foldedIntoParentSkills 表登记、catalog 以
covered_by 指向父体）。保留仅供追溯，不进入 npack。

## 2026-08-28 第一百四十一批移入
- qinzheng_count：上游为 UI/存储计数器，权威累计由引擎回合历史承载；本地
  文件注释自称「无独立规则行为」却带 optional draw-1 假体（自相矛盾），移档。

## 已发布包原件

- `shiji-0.2.4.npack.json`：J2 修订前的当前发布字节，供内容锁在升级、回滚与卸载后恢复。
- `shiji-0.1.0.npack.json`：更早的已发布原件，从 Git 历史保留。

当前修订另发 `0.2.5`；`0.2.1`–`0.2.3` 为元数据中的中间试作记录，未补造发布物。
