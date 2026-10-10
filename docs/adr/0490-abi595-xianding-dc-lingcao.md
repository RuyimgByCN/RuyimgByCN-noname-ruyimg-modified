# ADR 0490：ABI 595 XIANDING 凌操

旧 JavaScript 仅作语义证据，不进入 npack 或运行时。凌操（`dc_lingcao`，2455294139）以新 DSL
和共享引擎原语完成「独锋」（`dcdufeng`，2523630693）技能闭包：出牌阶段开始时强制选择失去
1 点体力、废除一个当前可用装备栏，或两项各执行一次；两种成本结算后，令 X 等于已废除装备栏
数量与本次失去体力值之和且不超过体力上限，摸 X 张牌。X 大于 1 时，临时辅助技能
`dcdufeng_effect`（226166961）令本回合攻击范围和【杀】使用次数各增加 X-1。

ABI 595 新增 `resolve_dcdufeng(player, action, helper_skill_def)`，复用失去体力、装备栏废除、
摸牌、临时技能和 owner mark 修正器管线；没有新增技能专属持久状态。同步目录同时重建为
26 包、2484 名武将、5881 个顶层技能，其中 524 名武将、1517 个技能已移植；已发布的包内辅助
技能 ID 继续作为兼容锚点，新辅助技能在首次发布前使用稳定 ID 算法。

验收：凌操 Golden 3/3、XIANDING 36 个专项测试目标共 128 项、content-compiler、engine-api、
Rule VM、conformance、BINGSHI 回归与 26 包空 store 冷导入全部通过；内容锁为
`f2b1fde7e2f70ffecbde190233b5c8657615138c030b153dfd3381a26c539346`。XIANDING 包为 ABI 595、
14 名武将、81 项技能、84 个模块，logic hash 为
`474704b596e705b0885033781bea6ece192040bba8894e6ef87d72c4411877d3`；源码与
`extensions/xianding` 四份 npack SHA-256 均为
`eb45aa2e5ad8f5460605b40b8f117d1da858eff1647ef10c9f542b5bd946eff2`。

编译产物按架构计划 §37.10 在全部并行编译消费者释放后强制清理：工作区 `target/` 原本不存在，
共享 `CARGO_TARGET_DIR` 删除 30,633 个文件（Cargo 报告 13.5 GiB），两目录均确认不存在。
清理范围仅为 Rust/Cargo 编译缓存；上述 npack 与 `extensions/` 发布物明确保留，清理后四份包的
SHA-256 仍完全一致。
