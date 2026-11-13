# 独立复核命令（2026-11-14）

工作目录：`/Users/ruyimg/Documents/Claude/sanguosha/noname-next`。脚本只向本独立目录写入；均未使用Git add/commit/push、Cargo clean、产品生成器或完整preflight。

```sh
python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/capture-inputs.py > docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/capture-inputs-first.log
python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/validate-inputs.py > docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/input-validation-first.log
python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/replay-native.py > docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/native-replay01-validation.log
python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/recompute.py first > docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/recompute-first.log 2>&1
CARGO_TARGET_DIR=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/target U08_ORIGINAL=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-14-queue1-runtime/U08 U08_REPO=/Users/ruyimg/Documents/Claude/sanguosha/noname-next U08_REVIEW_OUT=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/archive-replay01 cargo run --offline --manifest-path docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/archive-probe/Cargo.toml > docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/archive-replay01.raw.log 2>&1
CARGO_TARGET_DIR=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/target U08_ORIGINAL=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-14-queue1-runtime/U08 U08_REPO=/Users/ruyimg/Documents/Claude/sanguosha/noname-next U08_REVIEW_OUT=/Users/ruyimg/Documents/Claude/sanguosha/noname-next/docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/archive-replay02 cargo run --offline --manifest-path docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/archive-probe/Cargo.toml > docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/archive-replay02.raw.log 2>&1
python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/recompute.py second > docs/operations/evidence/2026-11-14-queue1-runtime/independent/u08/recompute-second.log 2>&1
```

`archive-replay01`编译失败，未创建其运行输出目录；失败源码与日志保留。`archive-replay02`才是76份存档的独立解码/后续命令重放结果。首轮重算metadata修正在README说明，原始first输出未覆盖。
