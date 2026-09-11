// EXPECTED: 5
fn f(a: &mut i32) {
    *a = 5;
}

fn main() {
    let mut a = 50;
    f(&mut a);
    printf("%d", a);
}
