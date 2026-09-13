// EXPECTED: 135
fn main() {
    let mut data = [false, true, true, false, true];
    data[1] = false;
    data[0] = true;

    let mut i = 0;
    while i < 5 {
        if data[i] {
            printf("%d", i + 1);
        }
        i += 1;
    }
}
