# mobile 包遗留技能存档

本目录存放已被父技能取代、退出编译池（rebuild-pack 的 .skill glob 不递归本
目录）的折叠遗留文件（sync 脚本 foldedIntoParentSkills 表登记、catalog 以
covered_by 指向父体）。保留仅供追溯，不进入 npack。

## 2026-08-28 第一百四十一批移入
- beizhu_draw：上游伤害摸牌助手；权威逐点直伤追踪由 ResolveBeizhu 承载，
  本地 draw-1 假体若被挂载会双份结算（未挂载），移档。
