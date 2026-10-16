# 2026-10-16 T9 / D5：龙血玄黄战斗触发轨迹采样

D5 已立项（[决定](../2026-10-16-d4-d8-decisions/README.md)）。本目录关闭 10-11 勘察中「无头环境未搭建 / archive 只读 / 未立项」三条阻塞里的前两条与立项前置，并交付首批轨迹样本。

## 命令与环境

```bash
# 运行副本（勿写 archive）
# scratch/legacy-harness/liuli 已打启动补丁：暴露 window.lib + legacy_harness 拉长 reset 看门狗

cd scripts/legacy-harness
npm ci   # node_modules 不入库
node sample-lx-battle.mjs /path/to/scratch/legacy-harness/liuli
node ../docs/operations/evidence/2026-10-08-e01-longxue-lx2/lx2_trace.js \
  /path/to/scratch/legacy-harness/liuli/extension/龙血玄黄/extension.js \
  ../../docs/operations/evidence/2026-10-16-t9-lx-battle-trace
```

核验日期：2026-10-16。种子：`20261016`。

## 产物

| 文件 | 内容 |
|---|---|
| [static-trigger-map.json](static-trigger-map.json) | Boss 诸葛亮四技（`xiangxing/yueyin/fengqi/gaiming`，`boss.js`）+ offline `ylyg*` 六技触发面静态摘录，**10 hits** |
| [boss-zhuge-skill-extract.json](boss-zhuge-skill-extract.json) | 诸葛亮四技行号与 trigger 摘要（禳星 damageEnd/loseHpEnd；风起 phaseZhunbei/Jieshu；改命 judgeBefore） |
| [lx-register-behavior-samples.txt](lx-register-behavior-samples.txt) | 注册期动态样本（复跑 LX.2）：`mbqianlong` 无条件；`_CMeffects` 仅 `shengjian:true` |
| [sample-log.txt](sample-log.txt) | 无头启动采样日志 |
| [runtime-skill-summaries.json](runtime-skill-summaries.json) / [runtime-trace.json](runtime-trace.json) | 对局钩挂面（本轮 content 命中 0，见边界） |

## 结论

1. **无头 harness 可用**：`scripts/legacy-harness`（jsdom + 本地断网加载 + 虚拟时钟 + Cordova/confirm 桩）；琉璃副本 `lib_ready=true`，推进至 `players=5` / `event=showCards:1`。
2. **龙血规则面注册轨迹（动态）**：与 10-08 LX.2 交叉印证——`mbqianlong` 必注册；`_CMeffects` 随 `config.shengjian` 门控。
3. **Boss 诸葛亮 / ylyg* 触发面（静态，供迁移对照）**：四技定义在 `mode/boss.js`（非 offline 文件名 `ylyg*` 前缀；T11 证据中的「ylyg* 族」指 offline 龙血玄黄/炎灵包同源命名族，诸葛亮本将四技为同主题 boss 本地定义）。offline `ylygxiangxing` 等六技触发面已入静态图。
4. **残余**：完整对局内 `skill.content` 钩挂命中仍为 0（本轮停在亮牌/选将交互，场上未装配目标技）。下一步：无头自动选将装配 `boss_zhugeliang` / `ylyg_*` / `mb_caomao` 后继续泵事件。

## 与 10-11 勘察对照

| 阻塞 | 本轮状态 |
|---|---|
| 无头运行未验证 | **已搭**：harness + scratch 副本可启动到 lib/对局事件 |
| archive 只读 / 无 node_modules | **已规避**：依赖与补丁仅在 `scratch/` 与 `scripts/legacy-harness` |
| 规则面迁移立项 | **D5 已立项**；本目录为立项后的首批采样证据 |
