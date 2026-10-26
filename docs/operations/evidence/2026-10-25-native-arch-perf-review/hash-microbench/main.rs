use std::time::Instant;
fn main() {
    // 近似 canonical_hash 的输入形态：约 900 次 1/4/8 字节的小块写入，合计约 3.6KB
    let sizes: Vec<usize> = (0..900).map(|i| [4usize, 4, 1, 8, 4][i % 5]).collect();
    let total: usize = sizes.iter().sum();
    let data = vec![7u8; total];
    let iters = 20000;
    let t = Instant::now();
    for _ in 0..iters {
        let mut h = blake3::Hasher::new();
        let mut off = 0;
        for &s in &sizes { h.update(&data[off..off + s]); off += s; }
        std::hint::black_box(h.finalize());
    }
    let small = t.elapsed().as_nanos() / iters;
    let mut buf = Vec::with_capacity(total);
    let t = Instant::now();
    for _ in 0..iters {
        buf.clear();
        let mut off = 0;
        for &s in &sizes { buf.extend_from_slice(&data[off..off + s]); off += s; }
        std::hint::black_box(blake3::hash(&buf));
    }
    let buffered = t.elapsed().as_nanos() / iters;
    println!("bytes={total} updates={} small_updates_ns={small} buffered_ns={buffered}", sizes.len());
}
