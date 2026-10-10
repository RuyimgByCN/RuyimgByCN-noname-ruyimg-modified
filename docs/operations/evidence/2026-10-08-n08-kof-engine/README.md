# P2/N08 模式入口批四：KOF 引擎层（规则 v83，2026-10-08）

主稿 P2/N08「模式入口扩充」批四：KOF 会话语义从示例/测试层升入引擎。
前置：[Boss 批三-a](../2026-10-08-n08-boss-mode/README.md)、
[KOF 会话切片](../2026-10-07-n08-kof-mode/README.md)（内联实现先例）。

## 语义来源（同构声明）

`kof_session_consistency_and_midrestore`（2026-10-07）内联的 KOF1 会话
语义——双池各 3 将、败者队首淘汰、胜者队首轮转至队尾、比分/局数累计、
败方池空即出线——本批原样升入引擎（引擎态版本），无语义改动。

## 改动（分开陈述）

- **代码已实现**：
  - `GameMode::Kof { pools: [Vec<u16>; 2], wins: [u32; 2], games: u32 }`
    ——会话状态随引擎存档（mode tag 6）/哈希（模式段 tag 7）；快照
    检查=2 人局 + outcome-模式一致性（Winner::Player）。
  - DeathMachine Kof 臂：单局终局自动推进会话——唯一存活者胜 →
    败者队首淘汰、胜者队首轮转（移至队尾）、比分/局数 +1；无存活者
    不推进。
  - `Engine::new_kof(seed, pools, wins, games)` 构造器（2 人局）。
- **边界登记**：多局连续会话循环（终局自动开局下一局）仍在宿主/示例
  层——引擎终局即停，会话推进结果随终局态可读；单局内将领授予仍按
  装配/授予原语。

## 验收口径

- **回归已加入**：`kof_engine_mode_session_advance`——两局推进
  （局 1：0 胜 → 1 队队首 9 淘汰、0 队 7 轮转至队尾、比分 1:0、局数 1；
  局 2（以推进后会话开局）：1 队池清空出线形态、比分 2:0、局数 2）、
  存档往返（推进后会话一致）。
- **全量通过**：engine-native **386 passed / 0 failed / 1 ignored**
  （唯一 ignore=写轮眼规模诊断探针），单线程
  `cargo test --offline -p engine-native -- --test-threads=1`；
  依赖包 content-runtime + engine-ai + content-native-loader
  **62 passed / 0 failed**，DEPS_EXIT=0。明细见 [test-logs.txt](test-logs.txt)。
- **golden**：版本 82→83 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（标准场景无 KOF 局，纯版本位移）。
- **上游等价边界**：会话推进语义等价；连续循环/选将 UI 在宿主层；
  boss 批三-b（addFellow）仍待做。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（11 文件，相对 noname-next 根）。
