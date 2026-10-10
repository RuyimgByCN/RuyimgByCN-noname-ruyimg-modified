# 0470 ABI 575 SP【豹变】与 XIANDING 新杀夏侯霸

日期：2026-08-03  
状态：已落地  
领取者：B（Codex overflow）

## 决策

旧 JavaScript 只作为语义合同，不进入内容包或运行时。`rebaobian` 的唯一规则定义放在 SP；
XIANDING 的 `dc_xiahouba` 通过既有 SP 依赖引用它，不复制技能实现。

ABI 575 新增通用效果
`add_first_missing_skill(player, first, second, third)`：按参数顺序检查三个非零且互异的技能定义，
永久授予第一个尚未拥有的技能；全部拥有时无事发生，且不会打开玩家选择。豹变依次引用
SHENHUA `tiaoxin(695232805)`、REFRESH `olpaoxiao(2829854661)`、SHENHUA
`xinshensu(2561255694)`，准确保留旧规则的固定获得顺序。

## 验收

- Compiler ABI 合同 1/1；Golden 覆盖连续三次受伤的固定授技顺序、跳过已有前项、第四次无事
  发生和不打开选择，3/3。
- SP 包：ABI 575，25 名武将、133 项技能、86 个模块；文件 SHA-256
  `b0152b9f5e8068ae4666cc5afcc015bab1d2cd40f56243d182915f8457630232`。
- XIANDING 包：ABI 521，3 名武将、64 项技能、55 个模块；文件 SHA-256
  `4f0ba0db0830df5aa59f27d55bdf72cda6950fa84497c4cbb5d6ab31231661ca`。
- 两包源码产物与各自 `extensions/` 的两份投放字节一致；相关 SP 包级冻结测试 19/19、
  conformance 8/8、engine-api 4/4、26 包冷导入通过，ContentLock 为
  `f2601bd2227f0e5439700bf338be082ddfc1e7c49cbc80304bfaff5582714048`。
- 编译产物按架构计划 §37.10 在批次结束后清理；`.npack` 与 `extensions/` 保留。
