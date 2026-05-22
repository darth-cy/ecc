# ECDSA Library Improvement Suggestions

## Correctness (must-fix bugs)

### 1. `add_points` assertion is wrong
`secp256k1.rs:76` has `assert!(pt1.y != pt2.y)` before the zero-point guards. This panics for any private key where the accumulator lands on a point with the same y-coordinate as G. The correct behavior:
- If `pt1 == pt2` → call `double_point`
- If `pt1.x == pt2.x && pt1.y != pt2.y` → they are inverses, return `zero_point`
- Otherwise → current addition formula

### 2. No point-on-curve validation
Input points are never checked against `y² ≡ x³ + 7 (mod p)`. An invalid point passed to `pr_to_pub` or future verify functions silently produces garbage.

---

## Missing Core Functionality

### 3. ECDSA sign and verify
The library has key derivation but no signing or verification — the two things you actually need. Sign produces `(r, s)` from `(privkey, hash, nonce)`, verify checks `(pubkey, hash, r, s)`.

### 4. RFC 6979 deterministic nonce
Never let callers supply a raw nonce. Random nonce reuse leaks the private key (this is literally how the PS3 was broken). RFC 6979 derives the nonce deterministically from `(privkey, hash)` via HMAC-SHA256, making it impossible to reuse.

---

## Security

### 5. Constant-time scalar multiplication
The `if d > 0` branch in `pr_to_pub` (`secp256k1.rs:150`) executes different code paths per bit of the private key — a textbook timing side-channel. Replace with the **Montgomery ladder**: always do both a double and an add, choose which result to keep.

### 6. Zeroize private key material
Private key bytes in `pr_to_pub` sit in a plain `[u8; 32]` stack array. Use the `zeroize` crate to wipe it after use.

---

## Performance

### 7. Jacobian/projective coordinates for point operations
Every `add_points` and `double_point` call computes a `div_mod`, which internally runs a full 256-bit modular exponentiation via Fermat's little theorem (`ru256.rs:178`). In Jacobian coordinates, you defer the inversion until the very end of scalar multiplication — one inversion for the entire `pr_to_pub` instead of 256+.

### 8. Extended Euclidean Algorithm for modular inverse
Even for the single end-of-multiplication inversion, `b^(p-2) mod p` is ~256 multiplications. The Extended Euclidean Algorithm computes the same result in ~50 steps.

---

## API and Serialization

### 9. Compressed point encoding
`to_hex_string` always outputs the 65-byte uncompressed form `04 || x || y`. Standard practice is the 33-byte compressed form `02/03 || x` (prefix depends on parity of y). Add `from_compressed` too, which requires computing the square root mod p.

### 10. DER-encoded signature serialization
ECDSA signatures are conventionally serialized as DER-encoded ASN.1, which is what every other tool (OpenSSL, browsers, hardware wallets) expects.

---

## Code Hygiene

### 11. Remove `println!` from hot paths
`secp256k1.rs:75` and `secp256k1.rs:111` fire on every point operation during scalar multiplication (256+ times per key derivation).

### 12. Private key range check
Enforce `1 <= privkey <= n-1` at the boundary. Zero and values ≥ n are invalid and produce degenerate results silently.

---

## Priority Order

The highest-leverage items in order:
1. Fix the `add_points` bug (correctness)
2. Add RFC 6979 + sign/verify (completeness)
3. Switch to Jacobian coordinates (~256x speedup, also fixes edge-case correctness)
