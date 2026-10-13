# 2026-10-12 用户决定 D1–D2 核查证据

> 本 README 为 2026-10-14 补记（孟婆批回填断链时发现当日只落了原始数据未写 README）。

## D1 史诗卡牌 3 个文件（不追查，以当前工作区为准）

- 原始核查：[d1-epic-files.txt](d1-epic-files.txt)（liuli HEAD `8485dbb` 与工作区对照，只读）。
- 结论：9-23 基线只存来源级聚合，148 字节差异无法定位到具体文件；9-23 列为修改的 4 个史诗卡牌文件均为表现层（滚动库、残影特效、表情动画、`extension.js` 卡牌展开悬停 UI），规则相关的 `reguhuo`/`rangjie` 段不在差异内。
- 落点：T4 收口（[进度主稿 T4](../../../architecture/native-progress.md)）；台账 L10 登记。

## D2 旧 JS 扩展兼容范围（不承诺零改动运行）

- 原始核查：[d2-legacy-js-survey.txt](d2-legacy-js-survey.txt)（两仓来源对引擎对象 `lib.element/game/get/ui` 的 function 赋值与 DOM 直接改写只读计数）。
- 结论：旧 JS 与引擎/DOM 耦合面广，按功能重构不承诺零改动运行；第三方扩展经新 SDK（包数据 + rule-vm）发布；QuickJS 等兼容层不立项，WASM 暂缓。
- 落点：[ADR 0782](../../../adr/0782-extension-backend-and-legacy-js-scope.md)；T8 转为引擎接线任务。

## D3 测试代码警告

纳入清理并扩展门禁，由 T21 完成（623→0，Gate 1b 改 `--all-targets`）——见 [T21 证据](../2026-10-12-t21-test-warnings-gate/README.md)。
