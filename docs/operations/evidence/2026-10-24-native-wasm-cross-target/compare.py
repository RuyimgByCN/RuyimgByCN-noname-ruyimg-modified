#!/usr/bin/env python3
"""比较 native 与真实 wasm 的同种子轨迹；逐 revision/hash/事件码，不抽样。"""
import json
import sys


def load(path):
    with open(path) as handle:
        return json.load(handle)


def compare_steps(name, native, wasm):
    if len(native) != len(wasm):
        raise SystemExit(f"{name}: length {len(native)} != {len(wasm)}")
    for index, (left, right) in enumerate(zip(native, wasm)):
        for field in ("revision", "hash", "codes"):
            if left[field] != right[field]:
                raise SystemExit(
                    f"{name}[{index}].{field}: native={left[field]!r} wasm={right[field]!r}"
                )
    return len(native)


def main():
    native = load(sys.argv[1])
    wasm = load(sys.argv[2])
    if native["seed"] != wasm["seed"] or native["players"] != wasm["players"]:
        raise SystemExit("seed/players mismatch")
    direct = compare_steps("direct", native["direct"], wasm["direct"])
    before = compare_steps("restore.before", native["restore"]["before"], wasm["restore"]["before"])
    after = compare_steps("restore.after", native["restore"]["after"], wasm["restore"]["after"])
    midpoint = native["restore"]["midpoint"]
    for index in range(midpoint):
        for field in ("revision", "hash"):
            if native["direct"][index][field] != native["restore"]["before"][index][field]:
                raise SystemExit(f"native direct/before diverged at {index}.{field}")
    for index, record in enumerate(native["restore"]["after"]):
        expected = native["direct"][midpoint + index]
        for field in ("revision", "hash", "codes"):
            if record[field] != expected[field]:
                raise SystemExit(f"native restore/direct diverged at {midpoint + index}.{field}")
    final_hash = native["direct"][-1]["hash"]
    if len(final_hash) != 64:
        raise SystemExit("final hash is not 64 hex chars")
    print(
        json.dumps(
            {
                "direct_steps": direct,
                "restore_before_steps": before,
                "restore_after_steps": after,
                "final_revision": native["direct"][-1]["revision"],
                "final_hash": final_hash,
                "native_wasm_match": True,
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
