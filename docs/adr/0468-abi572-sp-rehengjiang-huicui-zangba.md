# 0468 ABI 572 SP【横江】与 HUICUI 臧霸

日期：2026-08-03  
状态：已落地  
领取者：B（Codex overflow）

## 决策

旧 JavaScript 只作为语义合同，不进入内容包或运行时。SP 是 `rehengjiang` 的唯一技能定义包；
HUICUI 的 `zangba` 通过既有 SP 精确依赖引用该定义，不复制技能实现。

ABI 572 新增
`resolve_rehengjiang(player, actor, action, hand_limit_def, draw_def)`：每个伤害点可独立发动，
以绑定当前回合的临时技能叠加手牌上限减值，并在拥有者的结算 helper 中记录当前回合角色与
发动次数。回合结束时若该角色在弃牌阶段弃过牌则摸一张，否则摸发动次数张；结算状态随后
清空，两个 helper 由统一回合末临时技能生命周期删除。

## 验收

- Compiler ABI 合同通过；Golden 覆盖两次叠加、未弃牌摸二、弃牌摸一和跨包武将闭包，3/3。
- SP 包：ABI 572，25 名武将、132 项技能、85 个模块；文件 SHA-256
  `baab2ebfc584550f4d8e5c419b07001f62cf08e989ed1ef3a8e6fd91a62e7285`。
- HUICUI 包：ABI 487，15 名武将、34 项技能、38 个模块；文件 SHA-256
  `dbe3af630886bdf1bec65964e63c4a5956acdd3ac0169c36ece2a9c35d674209`。
- 两包源码产物与各自 `extensions/` 的两份投放字节一致；相关冻结测试共 22/22、conformance
  8/8、26 包冷导入通过，ContentLock 为
  `863a70260da3989e915e7d325018907d5be5f3ba308642b24ce65e5febb07272`。
- 共享工作区已有 ABI 573 神话包，因此引擎支持上限同步至已领取的 573；横江包自身仍精确
  声明 ABI 572。编译产物按架构计划 §37.10 在批次结束后清理，`.npack` 与 `extensions/`
  不属于清理对象。
