//! **生成 fuzz 语料种子**（维护工具，非产品路径）。
//!
//! 背景：`fuzz/corpus/fuzz_bytecode_module/valid_seed.bin` 的头 4 字节为旧格式 `NVM1`，
//! 而当前 `MODULE_MAGIC = b"NNVM"` ⇒ 该「valid」语料**已无法解码**（见
//! `docs/operations/evidence/2026-10-19-fuzz-corpus-replay/`）。
//!
//! 本工具用**当前格式**写一份最小合法模块到语料目录，供 stable 语料回放测试断言
//! 「至少有一个样本能解码」。
//!
//! 用法：
//! ```text
//! cargo run --offline -p rule-vm --example gen_fuzz_seed
//! ```

use rule_vm::bytecode::{assemble, BytecodeModule, Op, MODULE_MAGIC};

fn main() {
    let module = BytecodeModule {
        name: "fuzz_seed_nnvm".to_string(),
        code: vec![],
        local_count: 0,
        max_stack: 1,
        entry_pc: 0,
        logic_hash: [0u8; 32],
        caps_mask: 0,
    };
    let bytes = module.to_bytes();
    assert_eq!(&bytes[..4], &MODULE_MAGIC, "写出的头 4 字节应为当前 magic");
    // 回读自检：写出去必须自己能读回
    match BytecodeModule::from_bytes(&bytes) {
        Ok(m) => println!("回读成功：name={} code_len={}", m.name, m.code.len()),
        Err(e) => {
            eprintln!("回读失败（不写文件）：{e}");
            std::process::exit(1);
        }
    }
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fuzz/corpus/fuzz_bytecode_module");
    std::fs::create_dir_all(&dir).expect("mkdir corpus");
    let out = dir.join("valid_nnvm.bin");
    std::fs::write(&out, &bytes).expect("write seed");
    println!(
        "已写入 {} （{} 字节，magic={:?}）",
        out.display(),
        bytes.len(),
        String::from_utf8_lossy(&bytes[..4])
    );

    // 第二份种子：**能过校验器**的最小模块（`Op::Halt`）。
    // 动机：`valid_nnvm.bin` 的 `code` 为空 ⇒ 解码成功但 **校验器拒绝**
    // （`InvalidBytecode("empty code")`），导致 verify 回放只覆盖拒绝路径。
    let verifiable = BytecodeModule {
        name: "fuzz_seed_nnvm_halt".to_string(),
        code: assemble(&[Op::Halt]),
        local_count: 0,
        max_stack: 1,
        entry_pc: 0,
        logic_hash: [0u8; 32],
        caps_mask: 0,
    };
    let vbytes = verifiable.to_bytes();
    let vm = BytecodeModule::from_bytes(&vbytes).expect("可校验种子应能解码");
    rule_vm::verify_module(&vm).expect("可校验种子应通过 verify_module");
    let vout = dir.join("verify_ok_nnvm.bin");
    std::fs::write(&vout, &vbytes).expect("write verify seed");
    println!(
        "已写入 {} （{} 字节，带 Op::Halt，**校验通过**）",
        vout.display(),
        vbytes.len()
    );
}
