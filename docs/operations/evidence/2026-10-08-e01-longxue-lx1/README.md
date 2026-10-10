# E01/G0 LX.1 执行：龙血玄黄静态可读化（2026-10-08）

[立项](../2026-10-08-e01-longxue-survey/README.md) 的 LX.1 阶段完成：
混淆类型判定为 **jsjiami.com.v6**（非 obfuscator.io），packed 假说被
运行探针推翻（正文在 stub 环境下可完整定义与执行，无 eval/Function
载荷），实际为**多字符串表拼写源码**。自含式 Node 脚本 VM 隔离解码
+全字面量代换，**632/637 调用点解码成功**（余 5 处为运行时动态键/非
UTF-8 路径，静态不可代换，已登记）。

## 产物

- [extension.readable.js](extension.readable.js)——可读版（587KB；
  字符串已内联，`_0x` 标识符保留；来源哈希见
  [source-hashes.txt](source-hashes.txt)）。
- [test-logs.txt](test-logs.txt)——去混淆执行记录与段落清单。

## 功能清单初稿（可读版统计，待 LX.2 动态采样证实）

- **武将**：`mb_caomao`（手杀曹髦）、`mb_caomao2`（两形态）。
- **技能**（6+）：`mbqianlong`（乾龙，含 `_refresh/_1/_2/_3` 阶段态）、
  `mbcmqingzheng`（清正）、`mbcmjiushi`（酒诗）、`mbjuejin`（绝进）、
  `mbcmfangzhu`（放逐）、`mbweitong`、`mbpvbm4` 等。
- **UI/特效**：`<style>`×1（专属剑特效 + 「西瓜标签」@keyframes 动画
  CSS）、`updateProgress`×11（加载进度 UI）、config 元数据
  （diskURL/forumURL/version 1.0）；`files.character/card/skill` 全空
  ——资源全内联。
- **结论**：功能假说证实——手杀曹髦专属扩展（技能系统 + 剑特效 +
  UI 补丁），规则面（技能）与界面面（特效/样式）并存。

## 执行边界（与立项一致）

- VM 仅注入字符串表/解码器/访问器定义与表声明；**扩展正文从不执行**
  （代换为纯查表计算）。`debugger` 陷阱 ×2 中和为空语句。
- 附带运行探针（stub 环境 `lib/ui/game` 空对象 + document 桩）证实
  正文无 packed eval——该探针调用过一次 content 用于验证，未进入
  构建链，采样产物以静态代换结果为准。

## LX.2 待执行

隔离旧线勾桩采样：注册行为（`lib.character/skill/card` 写入时序、
config 菜单枚举、技能实际触发轨迹）——本清单为静态面，动态面待 LX.2
交叉印证后方可定案接口分配（LX.3）。
