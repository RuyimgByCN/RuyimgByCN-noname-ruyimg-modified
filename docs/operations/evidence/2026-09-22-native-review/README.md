# 2026-09-22 独立复核证据

当前结论见 [进度主稿](../../../architecture/native-progress.md)，性能口径见 [phase19](../../phase19-native-baseline.md)。

- [测试](native-tests.log)：213 项通过；14 golden 在一个场景测试中，不另加计数。
- [runner](runner.log)、[plain 对照](runner-plain.log)：同种子、规则 24，各运行一次；
  [samples](samples.csv)、[plain samples](samples-plain.csv) 均无表头，四列 load,game,seq,ns，各 11758 条。
- [探针源码](probes.rs)、[输出](probes.log)：记录确认存在的缺口；断言是“复现缺口”，不是修复验收。
  将探针链接到当前 debug engine-native 库即可运行；使用与 cargo 构建该库相同的 rustc 工具链。
  编译参数为 `--edition=2021 -L dependency=target/debug/deps --extern engine_native=<当前 rlib> -o /tmp/noname-review-probes`。
  `probes.rs` 引用现有 host_assembler 示例；长度溢出探针使用 catch_unwind，预期输出 panic 并继续，最终退出 0。
- [writer](writer.log)、[reader](reader.log)：两个进程的 H25/H35 本轮分别提取后比较相等；同宿主，不是跨平台验证。
- [装配器](host-assembler.log)、[文本渲染](render-host.log)、[KOF](kof.log)、[table](table.log)：示例运行输出，能力边界见主稿。
- [环境](environment.txt)、[源码摘要](source-hashes.json)：用于识别本轮版本；未记录设备型号/温控，不构成发布性能验收。

旧证据按原日期保留。本目录是本轮新增记录，未覆写早期日志。
