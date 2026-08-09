// EXPECTED: 5050
fn f(a: i32) -> i32 {
    let mut i = a;
    let mut b = 0;
    while i > 0 {
        b = b + i;
        i = i - 1;
    }

    return b;
}

fn main() {
    printf("%d", f(100));
}
