const LOW: i32 = 4000;
const MID: i32 = 5000;
const HIGH: i32 = 6000;

fn limit_to(base: i32, size: i32, origin: i64) -> i32 {
    let scale = if origin <= 0 {
        0
    } else {
        (origin / i64::max_value() * (size as i64)) as i32
    };

    return base + scale;
}

pub fn highest() -> i32 {
    return HIGH + 1000;
}

pub fn high(origin: i64) -> i32 {
    limit_to(HIGH, 1000, origin)
}

pub fn middle(origin: i64) -> i32 {
    limit_to(MID, 1000, origin)
}

pub fn low(origin: i64) -> i32 {
    limit_to(LOW, 1000, origin)
}
