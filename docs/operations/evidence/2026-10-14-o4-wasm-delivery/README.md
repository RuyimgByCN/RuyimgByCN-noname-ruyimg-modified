# O4 WASM 交付测量 + 功能回归（2026-10-14）

## 任务来源与口径

[进度主稿 O4](../../../architecture/native-progress.md)：固定 wasm-pack/wasm-opt 版本做可重复实验；比较原始、gzip/Brotli 体积与加载耗时；浏览器功能回归通过后选择配置；不以原文「7.75 MB→1.5 MB」作验收值。

环境：**wasm-pack 0.15.0**（固定）；**wasm-opt 未安装**（离线不可得——`--no-opt` 构建，opt 配置对比登记不可行）；rustc 1.95.0 / wasm32-unknown-unknown；Node v26.4.0（功能回归运行时）；git `6ed40511` 基线+本批。

## 体积测量（wasm-pack build --target nodejs --release --no-opt）

| 配置 | 体积 |
|---|---|
| 原始 `engine_wasm_bg.wasm` | **8,378,991 B（8.38 MB）** |
| gzip -9 | **2,255,909 B（2.26 MB）** |
| brotli q11 | **1,490,241 B（1.49 MB）** |
| 构建耗时 | 119s |

（对照原文口径「7.75 MB→1.5 MB」：本实测 gzip 后 2.26 MB、brotli 后 1.49 MB——brotli 已达原文下界量级；wasm-opt 进一步收缩面待工具可用后补测。）

## 加载/运行数据（Node v26.4.0，bundle=standard 全依赖闭包 19 包）

| 项 | 数值 |
|---|---|
| require（glue+wasm 实例化） | 7ms |
| loadContentBundle（19 包全内容锁校验+装载） | 102–104ms |
| startGame（4 人） | 10ms |
| autoStep ×80 | 39ms（finished=false，对局健康推进） |
| observerJson 投影 | 4,821 B |

## 功能回归

Node 全链路**通过**：实例化 → 内容束装载（19 包规范内容锁）→ startGame(4, 4242) → autoStep×80 → observerJson——[smoke-final.txt](smoke-final.txt)。**浏览器 UI 层回归不在本批**（需要 web-client dev server+真实浏览器，归 T13 产品面）。

## 冒烟实测发现的两项存量内容债（登记，非本批引入）

1. **`loadEmbeddedStandard` 单包回退路径必然失败**：内嵌 standard 的 roster 刻意跨包引用 refresh 技能（ADR 0542 delta，standard_pack.rs 注释明示"defers character skill validation to the full content lock"），单包加载后 `validate_character_skill_closure` 必然拒绝——产品路径本就走 `loadContentBundle` 全内容锁，回退路径的语义应改为「装载 standard+全依赖闭包」或标注不可用。
2. **跨包 roster 悬空引用 3 处**：yijiang.liyan→newjiang.dcduliang、tw.huanjie/tw.sb_sp_zhugeliang→bingshi/sb 技能——manifest dependencies 未声明提供包（存量内容债）。修复：`scripts/o4_build_bundle.sh`（新增）的依赖闭包种子补齐 bingshi/newjiang/sb 三包后 bundle 可装载。

## 登记边界

- wasm-opt（binaryen）离线不可得：opt 配置对比未做；上线构建前需固定 binaryen 版本补测。
- 浏览器 UI 回归（web-client dev server+浏览器）归 T13 产品面批。
