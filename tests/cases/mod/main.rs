// EXPECTED: 134
mod math;
mod math2;
mod math4;

fn main() {
    printf("%d", math::square(10) + math2::square(5) + math4::square(3));
}
