// EXPECTED: 5050
fn f(a: i32) -> i32 {
    let mut i = a;
    let mut b = 0;
    loop {
        b = b + i;
        i = i - 1;

        if i == 0 {
            break;
        }
    }

    return b;
}

fn main() {
    printf("%d", f(100));
}
