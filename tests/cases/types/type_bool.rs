// EXPECTED: 15
fn f(a: bool) -> i32 {
    if a {
        return 10;
    }
    return 5;
}

fn main() {
    printf("%d", f(true) + f(false));
}
