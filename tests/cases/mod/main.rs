// EXPECTED: 125
mod math;
mod math2;

fn main() {
    printf("%d", math::square(10) + math2::square(5));
}
