# 0469 NEWJIANG 孙礼【伉厉】闭包重新打包

日期：2026-08-03  
状态：已落地  
领取者：B（Codex overflow）

## 决策

旧 JavaScript 只作为语义合同，不进入内容包或运行时。`kangli` 已由 DSL/Rust 表达：造成或
受到伤害后摸两张，并记录本次实际获得且仍连续留在手里的实体牌；下次造成伤害前弃置这些牌。
孙礼的 `characters.json` 已完整登记，但旧 NEWJIANG npack 未携带角色，因此本批不新增 ABI，
只修复发布闭包并重新打包投放。

打包改为读取 NEWJIANG 目录全部 `*.skill`，同时显式传入 `characters.json` 与
`metadata.json`；这取代会漏现有技能、且引用不存在临时文件的手写命令。

## 验收

- NEWJIANG 包保持 ABI 562，包含 1 名武将、17 项技能、11 个模块；孙礼为男性、魏势力、
  4 体力，唯一技能为 `kangli(2295716313)`。
- 文件 SHA-256 为
  `a5aad3fce99c487280a20d6311c9c3f8193045335f677d84b2f92a967efbb8b3`；源码产物与
  `extensions/newjiang/newjiang.npack.json`、`package.npack.json` 字节一致。
- NEWJIANG Golden 12/12、conformance 8/8、26 包冷导入通过；ContentLock 为
  `1215f0eaab9a74f2f60856682700723bf634f9cce2f23e7382a97ecfeb35975b`。
- 编译产物按架构计划 §37.10 在批次结束后清理；`.npack` 与 `extensions/` 保留。
