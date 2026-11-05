# R05-G/H/I/J 预勘察（2026-11-04，HEAD 35218101 只读勘察，未改源码）

## G 批：存档 107 实际缺口（比计划小）
- 已入档（无缺口）：D/R/G 机器快照 tag 90/91/92（machine_encode/decode.rs）；
  EventData::Draw 载荷含 held_cards（archive/events.rs:141-149/433-441）；
  DrawWindow/DrawOutcome/DrawTerminal/DrawGainProgress/DrawPhaseProgress/DetachedKind 全套 tag 编解码（event/draw.rs）。
  DrawMachine{root} 恢复从已存档 EventData 重派生 held_cards——中途挂起 R 往返天然成立。
- 剩余三件：
  1. state_encode.rs:345 注释处（table 后、rng 前）编码 detached_cards；
     state_decode.rs:534 Vec::new() 改接 dec_detached（state_decode.rs:4 已写好）。
  2. ARCHIVE_FORMAT_VERSION 106→107（archive.rs:116）；版本门禁同步两处：
     pa12_tag656_version.rs:8、native_kernel/d4.rs:47。
  3. golden 哈希域补 detached（engine/src/engine/hash.rs:435 push_cards(table) 后，
     id+kind 追加；b27 先例）。goldens 本就在 J 重录，无额外代价。
- 红测锚：draw.rs:310 牌堆弃堆皆枯竭→DetachedKind::Unclaimed 生产者存在；
  抽干后存档往返 detached 守恒 + 106 旧档拒收。
- 生产者现状注记：state_encode.rs:345 注释称"此时尚无生产者"已过时（Exhausted 路径即生产者）。

## C 批：147 处迁移形状（比计划轻）
- rg 全量核对：147 处全部 ctx.draw_cards(p, n) 弃返回值；零「= ctx.draw_cards」结果消费点。
- 同 step 手牌读：抽样 top3 文件（swd_status 19/skills_status 12/boss 8）3 行窗口无 hand/len 读。
- 迁移=机械替换 ctx.start_draw(p, n)＋逐文件回归（轨迹敏感面靠 1936 基线暴露）。
- 语义注意：start_draw 只压 R 帧（无 D 窗）；count==0 不发射；RNG 时点移入 R 泵（golden 位移归 J）。

## J 批程序要点
- ABI-MAX=758、领取表末行 758；759 未领取。升号前须先在 ABI 领取表加行
  +ABI-MAX 提 759（engine-api/src/version.rs abort_bump_without_claim 测试拦截）。
- WS8 = network_protocol 7→8（engine-api/src/version.rs VersionBundle::current）。
- 版本四轴：规则 332→333（content/standard.rs:163）、NGSA 106→107（G 批时刻）、ABI 758→759、WS 7→8。
- golden 唯一失败= replay_scenarios（A2 回合模型位移，待 J 三步探针重录）。
