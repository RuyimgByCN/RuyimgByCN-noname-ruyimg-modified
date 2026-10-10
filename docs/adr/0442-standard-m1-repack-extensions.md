# 0442 standard M1 收尾重打包与 extensions 落地（Hermes E 段）

日期：2026-08-03
状态：已落地
领取者：E (Hermes)

## 背景

`content/standard/`（M1 canonical pack，`standard_m1.skill`）为标准包 DSL 源；此前
`extensions/standard/` 仅含 `package.npack.json`（2016-07-30 编译产物），缺少
`standard.npack.json` 与 README，未达到「放入 extensions」的交付标准。

## 变更

1. 以当前 content-compiler（compiler_version 0.5.0-active-selection）重打包
   `standard_m1.skill` + `characters.json` + `metadata.json`，输出字节与既有
   `content/standard/standard.npack.json` 完全一致（40 modules / 65 skills /
   30 characters / ABI 261 / logic_hash cb25dc…e818），确认无需重打包，源与产物同步。
2. `extensions/standard/standard.npack.json` ← 复制编译产物（与 package.npack.json 同源）。
3. 新增 `extensions/standard/README.md`：武将表（30 武将，含经典标准 25 将 +
   std_panfeng / re_yuanshu / xf_yiji / std_yuejin / std_guanxing 变体）、技能清单（65）、
   打包命令与测试入口。

## 验收

- `extensions/standard/` 三件套齐备：package.npack.json / standard.npack.json / README.md。
- 与其余已交付扩展包（sixiang 等）目录结构一致。
- 纯 DSL 收尾，零新 ABI（E 段 559 起未占用）。
