// EXPECTED: 9
fn main() {
    let mut data = [1, 2, 3, 4, 5];
    data[2] = data[2] * 3;

    printf("%d", data[2]);
}
