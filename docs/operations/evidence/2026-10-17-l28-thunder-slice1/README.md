# L28 雷霆万钧 切片 1：可移植决策面（配置 + 同名替换表 + 标记补丁）（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L28 雷霆万钧**（`archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js`，442 KB）
- 本批：装载器 `content-native-loader::thunder` + 转换包 `packages/extensions/thunder-upstream-liuli`（生成器 `gen_thunder_package.rs`）
- **规则/存档版本不变**（v321 / 存档 v105；无引擎改动）
- 台账：L28 由「待重构；未验收」→ **部分重构（切片 1/N）**

## G0 清点（本批结论）

| 面 | 上游内容 | 处置 |
|---|---|---|
| 配置 | `gameSpeed`（9 档）、`replaceCha`、`hidePack`、`forbidSame`、`UIpatch`、`jinyu`、`shhbiaoji`、`shh_new`、`jiufatext`、`guanning`、`zhaoxiang`、`quanhuijie` | 本批 10 项入 `ThunderConfig`（`zhaoxiang`/`quanhuijie` 为技能内容补丁 → 后续切片） |
| 同名替换 | `sameHero`（12）×`chgHero`（12）平行表；`lib.character[i][3] = lib.character[chg][3]` | `SAME_HERO`/`CHG_HERO` 常量 + `replace_plan()`（配置关闭→空计划） |
| 隐藏专属包 | `hidePack` → 删 `lib.characterPack["thunder"]` / 从 `all.characters` 移除 | `hides_exclusive_pack()` + `EXCLUSIVE_PACK = "thunder"` |
| 禁同名 | `forbidSame` → 重名武将标 `unseen` + 选将列表去重 | 策略开关（原生选将面后续切片） |
| 标记补丁 | 曹金玉标记转文字、孙寒华妙剑/莲华分级（`shh_new` 上标数字 `⁰¹²³⁴⁵⁶⁷⁸⁹`）、神姜维九伐改文字 | `mark_labels(cfg, miaojian, lianhua)`（分级文本/上标两态；补丁关闭→空） |
| 武将内容 | 31 处 `lib.character`（`th_*` 小游戏武将）+ 若干 `lib.skill` 补丁 | **deferred**（后续切片） |
| 渲染面 | UI 补丁、立绘/主题切换、立绘命名规则 | 登记差异（原生不迁移） |

## 验证

```
cargo test --offline -p content-native-loader --test thunder_loader
cargo test --offline -p content-native-loader
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（装载器） | **3 passed / 0 failed**（[loader-tests.log](loader-tests.log)）：配置解析 + 速度档 0–8 越界报错 + 缺字段报错；两表平行与首末配对、关闭替换→空计划、专属包名；标记文本三态（「N级」/上标/补丁关闭为空） |
| engine-native 全量 | **1060 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **72/0/0**（[deps-tests.log](deps-tests.log)；较上批 +3） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 包与生成器

- 包：`packages/extensions/thunder-upstream-liuli`（`manifest.json` + `package.npack.json` + `assets/config.json`）。
- 生成器：`crates/content-runtime/examples/gen_thunder_package.rs`（可重复生成）。

## 登记差异

- 同名替换以**计划**（`ReplacePlan`）暴露，由宿主在装配期执行；原生武将装配面尚未接入该计划（后续切片）。
- 禁同名策略仅登记开关（原生选将面未接入）。
- 标记补丁只覆盖孙寒华妙剑/莲华（曹金玉、神姜维的文本补丁待后续切片，其上游实现为 `lib.translate` 全局重写）。
- UI 补丁/立绘/主题为渲染面，原生不迁移。
