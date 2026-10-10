# sp 包遗留技能存档

本目录存放已被父技能取代、退出编译池（rebuild-pack 的 .skill glob 不递归本
目录）的折叠遗留文件（sync 脚本 foldedIntoParentSkills 表登记、catalog 以
covered_by 指向父体）。保留仅供追溯，不进入 npack。

## 2026-08-28 第一百四十一批移入
- hanyong3/fengpo2/qiangwu3：bookkeeping/计数/双 modifier 助手；权威语义由
  父技 hanyong/fengpo/qiangwu 承载（qiangwu.skill 含同款双 modifier 臂），
  本地为占位/近似假体，移档。shanjia2 与 sp2:langmie_damage 同批审计为诚实
  noop 保留在池。
  同批退役 original_sp_fengpo2.rs 测试（其断言对象即 fengpo2 固定 +1 占位，
  非上游「按风魄出杀计数加伤」语义；父技 fengpo 语义由 npack 装载路径覆盖）。
