// EXPECTED: 64
fn f(a, b, c) { let d = (a - b) * c; return d * 4; } fn main() { printf(f(7, 3, 4)); return 0; }
