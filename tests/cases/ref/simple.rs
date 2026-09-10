// EXPECTED: 50
fn f(a: &i32) {
    printf("%d", *a);
}

fn main() {
    let a = 50;
    f(&a);
}
