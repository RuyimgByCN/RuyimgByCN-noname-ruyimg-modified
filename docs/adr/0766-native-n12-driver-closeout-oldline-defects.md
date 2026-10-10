# ADR 0766：N12 旧线驱动器收尾批——两次进局与旧线自身缺陷定界

日期：2026-09-22（第六十七批）
状态：部分完成（驱动器已推进至对局内部；剩余阻塞定性为旧线自身代码缺陷）
上游：[0765](0765-native-n12-headless-driver-loopback-breakthrough.md) / [0743](0743-native-n12-headless-driver-attempt.md)

## 执行记录（按 ADR 0765 定界路径）

1. **服务**：`@noname/fs` 桥（8091）启动正常（HTTP 200）。
2. **对话框免疫**：经 chrome-devtools `initScript` 在游戏脚本前注入
   `confirm/alert` 自动应答，引导期弹窗（GPL 确认等）不再阻塞。
3. **关键发现——服务 worker 注册循环**：首次注入式引导仍整体卡死；
   拦截 `navigator.serviceWorker.register` 后页面恢复响应
   （`document.readyState=complete`、eval 秒回、`window.game` 就绪，
   SW 注册尝试计数 = 1）。卡死根因与 SW 注册/更新高度相关。
4. **两次进局**：
   - 教程向导流：跳过向导成功、`chooseCharacter` 启动（被扩展
     `千幻聆音` 的 `get.prefixBoldName is not a function` 中断）。
   - 单挑流：模式点击成功、`chooseCharacter` step 1 推进，随后
     命中**核心编译器**缺陷：`StepCompiler.packStep` 编译出的对局
     代码引用 `undefined.randomRemove`（`Cannot read properties of
     undefined (reading 'randomRemove')`），对局事件循环中止。

## 结论（阻塞定性变更）

- 环境侧（网络/服务/浏览器自动化）**全部排除**：回环可达、桥服务
  正确、SW 卡死有规避手段、UI 自动化可进局。
- 剩余阻塞 = **旧线自身两处缺陷**：①捆绑扩展与构建不兼容
  （`get.prefixBoldName` 缺失、`AI优化` 动态导入 404）；
  ②单挑/身份流核心事件代码 `randomRemove` 空引用。两者均属
  `archive/Noname-ruyimg-Modified` 旧线代码修复范畴（需旧线
  会话/授权），不属于 engine-native 引擎重写工作。
- 修复旧线缺陷后的驱动器最后一步（托管 AI 完局 + n09 口径聚合
  统计提取：命令数/事件数/守恒/终局）与真机验证（真机载体）
  待具备条件时执行。

## 驱动器操作手册（沉淀）

- 服务：`pnpm -F @noname/fs dev --dirname=../../dist --port=8091`；
- initScript：`confirm/alert` 自动应答 + `serviceWorker.register`
  拒绝桩（防卡死）；
- 进局：菜单「单挑」→（自动选将流）；统计：DOM `.player` 采集 +
  终局对话框文本。
