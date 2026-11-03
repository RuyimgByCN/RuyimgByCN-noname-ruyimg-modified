# R05-C kernel 批（20 处 P 类 draw 全迁，2026-11-03）

基线＝HEAD 386d8611 之前；核验命令＝三 crate 全量（见 logs/c-07.log：**1916/1/75**，唯余 golden）。

## 首版桥接（弃用）与教训

`draw_cards` 改「推 R 帧后内联步进 Draw/Gain 机器」的桥接版（logs/c-05-bridge.log）在 10 项恢复类测试爆
`frame_parent` 越界：G Transfer 里 `ctx.emit(CardsDrawn)` 压出的事件帧在内联收栈时被 retain 挖掉父引用。
桥接整体弃用（提交里保留弃用说明），改逐点真实迁移。

## 逐点迁移（kernel 侧 ctx.draw_cards 调用清零）

| 文件:行 | 技能/机制 | 迁移方式 |
| --- | --- | --- |
| events.rs ×8 | 回复/积财/草药/重生/忍性族（提交路径） | start_draw（69f531d1） |
| phase/end.rs 4 | 爆敛/无量/猛态/兽九 | start_draw（358caebe） |
| damage.rs 6 | 灵渠 cancel/灵渠 count/忍性/木森勾芒/伤逝 | start_draw＋DrawHeld 所有权区（358caebe） |
| phase.rs 2＋play 1＋judge 2＋death 1 | 补摸/补牌/逐张补/涅槃 | start_draw（31c27aa7） |
| death.rs 3 | 身份/双势力死亡奖励 | start_draw（592b09e2） |
| dying.rs 2 | 鬼寒回复摸 2/男性摸 2 | start_draw |
| sha/step 2＋resume 2 | 无手牌令摸/寒冰取消摸 1 | start_draw；**雌雄取消臂改 `Some(Flow::Yield)`**——开放询问会暂停调度器，先 Yield 让 R 子树完成再由 !asked 分支重开闪询问（cixiong 回归锁） |
| phase/end.rs 红莲 1 | 摸 2＋全体火伤 | 先压伤害帧后 start_draw＝栈顶先执行（先摸后伤，上游序） |
| tricks.rs 2 | 无中生有/地载人济 | WuzhongMachine 加 draw_done 续行位（真实 R 帧→Yield→再弃本体）；地载逐目标 start_draw 一次 Yield |
| rvm_ops.rs 1 | VM DrawCards 效果 | start_draw（后续步不读所抽牌） |

content 侧 147 处 `draw_cards` 保留旧同步实现（machine.rs 定义本体），随 E 批 emit_effects 新路径迁移。
