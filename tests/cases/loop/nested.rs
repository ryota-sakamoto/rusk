// EXPECTED: 9900
fn main() {
    let mut result = 0;
    let mut i = 0;
    loop {
        let mut j = 0;
        loop {
            j += 1;
            if j == 100 {
                break;
            }

            result += 1;
        }

        i += 1;
        if i == 100 {
            break;
        }
    }

    printf("%d", result);
}
