#!/usr/bin/env bash
# NC12 基线测量复跑脚本（2026-10-18）。
#
# 用法（在 noname-next/ 根目录）：
#   bash docs/operations/evidence/2026-10-18-nc12-baseline/run-all.sh
#
# 口径：全部命令 `--offline --release`，逐个**串行**执行（并发会污染尾延迟）；
# 每条命令前后记录 load average；`/usr/bin/time -l` 取进程峰值 RSS。
# 二进制冻结到临时目录后再跑，避免共享 target/ 被并发重建替换。
set -u
ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
EV="$ROOT/docs/operations/evidence/2026-10-18-nc12-baseline"
cd "$ROOT" || exit 1

cargo build --offline --release -p engine-native --example nc12_bench || exit 1
FREEZE="$(mktemp -d)/nc12_bench"
cp target/release/examples/nc12_bench "$FREEZE" || exit 1

run_logged() {
  local log="$1"; shift
  {
    echo
    echo "\$ $*"
    echo "load_before: $(uptime)"
  } >> "$log"
  /usr/bin/time -l "$@" >> "$log" 2>&1
  local rc=$?
  echo "exit=$rc" >> "$log"
  echo "load_after: $(uptime)" >> "$log"
}

{
  echo "# NC12 基线测量环境记录"
  echo
  echo "\$ date -u"; date -u
  echo; echo "\$ sw_vers"; sw_vers
  echo; echo "\$ uname -a"; uname -a
  echo; echo "\$ sysctl machdep.cpu.brand_string hw.memsize hw.ncpu hw.model"
  sysctl -n machdep.cpu.brand_string; sysctl -n hw.memsize; sysctl -n hw.ncpu; sysctl -n hw.model
  echo; echo "\$ sysctl hw.perflevel0.logicalcpu hw.perflevel1.logicalcpu"
  sysctl -n hw.perflevel0.logicalcpu; sysctl -n hw.perflevel1.logicalcpu
  echo; echo "\$ rustc --version; cargo --version"; rustc --version; cargo --version
  echo; echo "\$ rustup show active-toolchain"; rustup show active-toolchain 2>&1 | head -3
  echo; echo "\$ git rev-parse HEAD"; git rev-parse HEAD
  echo; echo "\$ git log -1 --format='%H %ad %s' --date=iso"; git log -1 --format='%H %ad %s' --date=iso
  echo; echo "\$ git status --short | wc -l"; git status --short | wc -l
  echo; echo "\$ git diff --stat -- crates/engine-native crates/content-native-loader crates/content-runtime | tail -3"
  git diff --stat -- crates/engine-native crates/content-native-loader crates/content-runtime | tail -3
  echo; echo "\$ shasum -a 256 冻结二进制与工具源码"
  shasum -a 256 "$FREEZE" | sed "s#$FREEZE#nc12_bench(frozen)#"
  shasum -a 256 crates/engine-native/examples/nc12_bench.rs
  echo; echo "\$ uptime"; uptime
} > "$EV/01-env.txt" 2>&1

run_logged "$EV/02-probe.log" "$FREEZE" probe samples=5 >/dev/null 2>&1

# 建局：逐档独立进程（每档都重新 import 全部 drop-in 包）。
: > "$EV/03-build.log"
for t in none swd huanle combo; do
  run_logged "$EV/03-build.log" "$FREEZE" build "tiers=$t" samples=30 seats=4 seed=42007
done

# 整局驱动：三档 × 节点预算 8/64/256 × 策略变体 0/1，每档 3 局（种子 42007–42009）。
# combo+variant1 命中产品源码 panic（见 09/10 号日志），该组以 catch_panics=1
# 捕获并标注 degraded=true；其余组不捕获，首次失败即中止。
: > "$EV/04-drive.log"
for t in none swd huanle combo; do
  for b in 8 64 256; do
    for v in 0 1; do
      extra=""
      if [ "$t" = "combo" ] && [ "$v" = "1" ]; then extra="catch_panics=1"; fi
      run_logged "$EV/04-drive.log" "$FREEZE" drive "tier=$t" seats=4 "budget=$b" lookahead=1 "variant=$v" games=3 seed=42007 max_steps=4000 wall_secs=300 $extra
    done
  done
done

# variant 1（更强候选）在扩展内容下会命中产品源码 panic（见 09/09b/10 号日志）；
# 该组以 catch_panics=1 再跑一遍，完整覆盖「档位 × 预算 × variant1」，运行由工具
# 自行标注 degraded（有 panic 的档位）——降级运行的延迟数字只作参考，不作收益计算。
: > "$EV/04b-drive-degraded-variant1.log"
for t in none swd huanle combo; do
  for b in 8 64 256; do
    run_logged "$EV/04b-drive-degraded-variant1.log" "$FREEZE" drive "tier=$t" seats=4 "budget=$b" lookahead=1 variant=1 games=3 seed=42007 max_steps=4000 wall_secs=300 catch_panics=1
  done
done

# 状态复制/模拟与状态规模增长。
: > "$EV/05-copy.log"
for t in none swd combo; do
  run_logged "$EV/05-copy.log" "$FREEZE" copy "tier=$t" seats=4 budget=64 steps=60 samples=30 growth_every=20 seed=42007
done

# 存读档（含扩展目录）与归档字节。
: > "$EV/06-archive.log"
for t in none swd huanle combo; do
  run_logged "$EV/06-archive.log" "$FREEZE" archive "tier=$t" seats=4 samples=30 steps=80 seed=42007
done

# AI 策略矩阵：同种子同预算，lookahead 1/2 × policy_variant 0/1。
: > "$EV/07-ai.log"
: > "$EV/07b-ai-combo-degraded.log"
run_logged "$EV/07-ai.log" "$FREEZE" ai tier=none seats=4 budgets=8,64,256 configs=1/0,1/1,2/0,2/1 cmds=200 seed=42007
run_logged "$EV/07b-ai-combo-degraded.log" "$FREEZE" ai tier=combo seats=4 budgets=64 configs=1/0,1/1,2/0,2/1 cmds=200 seed=42007 catch_panics=1

# 长局内存：≥800 命令（多局累积），计数分配器 + /usr/bin/time -l 峰值 RSS。
: > "$EV/08-long.log"
run_logged "$EV/08-long.log" "$FREEZE" long tier=none seats=4 budget=8 min_cmds=800 max_games=4 sample_every=25 seed=42007 alloc=1
run_logged "$EV/08-long.log" "$FREEZE" long tier=combo seats=4 budget=8 min_cmds=800 max_games=4 sample_every=25 seed=42007 alloc=1 catch_panics=1

# 首次失败记录（不捕获 panic，进程中止 + 完整 backtrace）。
: > "$EV/09-first-failure-panic.log"
RUST_BACKTRACE=full run_logged "$EV/09-first-failure-panic.log" "$FREEZE" drive tier=combo seats=4 budget=64 lookahead=1 variant=1 seed=42007 max_steps=4000 wall_secs=300 trace=1

# swd 单扩展 + variant1：**权威结算路径** panic（不是模拟内）——进程中止 + backtrace。
: > "$EV/09b-first-failure-swd-real-trajectory-panic.log"
RUST_BACKTRACE=1 run_logged "$EV/09b-first-failure-swd-real-trajectory-panic.log" "$FREEZE" drive tier=swd seats=4 budget=64 lookahead=1 variant=1 games=3 seed=42007 max_steps=4000 wall_secs=300 catch_panics=1 trace=1

# 崩溃边界矩阵（每条独立进程；失败保留 exit 码）。
: > "$EV/10-panic-boundary-matrix.log"
for b in 8 64 256; do
  for s in 42007 42008; do
    run_logged "$EV/10-panic-boundary-matrix.log" "$FREEZE" drive tier=combo seats=4 "budget=$b" lookahead=1 variant=1 "seed=$s" max_steps=4000 wall_secs=60
  done
done
run_logged "$EV/10-panic-boundary-matrix.log" "$FREEZE" drive tier=combo seats=4 budget=64 lookahead=2 variant=1 seed=42007 max_steps=4000 wall_secs=60
run_logged "$EV/10-panic-boundary-matrix.log" "$FREEZE" drive tier=combo seats=4 budget=64 lookahead=1 variant=0 seed=42007 max_steps=4000 wall_secs=60
run_logged "$EV/10-panic-boundary-matrix.log" "$FREEZE" drive tier=swd seats=4 budget=64 lookahead=1 variant=1 seed=42007 max_steps=4000 wall_secs=60
run_logged "$EV/10-panic-boundary-matrix.log" "$FREEZE" drive tier=huanle seats=4 budget=64 lookahead=1 variant=1 seed=42007 max_steps=4000 wall_secs=60

# 存读档失败的跨档/跨步数边界（swd 单扩展）。
: > "$EV/11-restore-failure-matrix.log"
for t in none huanle swd combo; do
  for s in 5 20 40; do
    run_logged "$EV/11-restore-failure-matrix.log" "$FREEZE" archive "tier=$t" samples=3 "steps=$s" seed=42007
  done
done

echo "NC12 基线测量完成：$EV"
