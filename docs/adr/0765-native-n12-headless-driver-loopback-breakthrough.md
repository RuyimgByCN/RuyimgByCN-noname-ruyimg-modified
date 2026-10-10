# ADR 0765：N12 旧线无头驱动器——回环贯通与服务端要件更正

日期：2026-09-22（第六十六批）
状态：部分完成（驱动器前四步会话内贯通；完局统计与真机验证留待收尾批）
上游：[0743](0743-native-n12-headless-driver-attempt.md) / [0742](0742-native-traitor-anchor-oldline-install.md)

## 背景与更正

ADR 0743 将 N12 阻断归因为「本工作区沙箱本地网络不可达（curl 000）」。
本批以直接探针**更正**该结论：当前沙箱内本地回环完全可达
（`python3 -m http.server` 绑定 127.0.0.1 → curl 200，含响应体），
浏览器自动化通道（chrome-devtools）可用。真正的加载失败原因重新
定界如下。

## 驱动器链路会话内贯通记录

1. **静态服务不可用**：纯 `http.server` 服务 dist 时引导报
   `game.checkFile is not a function`——浏览器路径需要文件管理桥
   注入（`game.checkFile` 在 dist 内仅是类型占位）。
2. **正确服务端 = `@noname/fs` 桥**（`pnpm -F @noname/fs dev
   --dirname=../../dist`，默认端口 8089）：桥服务下旧线在真实
   Chrome 中**完整加载**——246+ 资源全部 200，主菜单/模式列表/
   设置项完整呈现（a11y 树取证）。
3. **UI 自动化进局**：教程向导「跳过向导」点击成功，对局进入
   `chooseCharacter` 事件（扩展错误栈证实：
   `event.name: chooseCharacter`，StepCompiler 帧号齐备）。

## 剩余阻塞（旧线自身缺陷，非环境）

1. **捆绑扩展兼容性**：`千幻聆音` 扩展调用 `get.prefixBoldName`
   （本构建不存在）在 chooseCharacter 阶段抛错中断选将；
   `AI优化` 扩展动态导入 404。扩展失败确认框连环弹出亦会阻塞
   自动化通道。
2. **修复路径（收尾批执行）**：加载前经配置关闭全部扩展
   （`lib.config.all.extensions` 清空并持久化，或移动
   `dist/extension/*` 后清空配置引用），随后单挑/身份局 +
   托管（AI 完局）+ 聚合统计提取（命令数/事件数/守恒/终局，
   沿 n09 口径；旧线侧经 DOM 与 `Record` 日志提取）。
3. **真机验证**：依赖真机载体，维持会话外条件不变。

## 影响

- N12 阻断性质由「环境不可达」更正为「服务端要件 + 旧线扩展缺陷」，
  两者均已在会话内定界并给出修复路径；驱动器闭环的最后一步收窄为
  「扩展禁用后的完局统计提取」。
- 官方迁移/引擎侧开发不受影响：全量 211 passed / 0 failed（版本 24）。
