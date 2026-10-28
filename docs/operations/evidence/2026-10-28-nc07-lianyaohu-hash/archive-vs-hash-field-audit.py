# 静态比对：state_encode.rs（入档）引用的 PlayerState(p.)/TurnState(t.) 字段，
# 是否在 engine/hash.rs 中以 `.field` 形式被引用。只是字段名口径，不证明语义影响；
# 每项是否入摘要须另做反例并经版本升号授权。
import re
enc=open('crates/engine-native/src/archive/state_encode.rs').read()
hsh=open('crates/engine-native/src/engine/hash.rs').read()
def fields(src, var):
    return set(re.findall(r'\b%s\.([a-z_][a-z0-9_]*)\b' % var, src))
skip={'len','iter','is_empty','clone','as_ref','map','copied'}
for var,label in [('p','PlayerState'),('t','TurnState')]:
    e=fields(enc,var)-skip
    missing=sorted(f for f in e if not re.search(r'\.%s\b' % re.escape(f), hsh))
    print(f"[{label}] encoded in archive but never referenced in hash.rs: {len(missing)}")
    for f in missing: print("   ", f)
