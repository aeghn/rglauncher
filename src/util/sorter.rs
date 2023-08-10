pub const H_1000: i32 = 10000;
pub const H_900: i32 = 9000;
pub const H_800: i32 = 8000;
pub const H_700: i32 = 7000;
pub const H_600: i32 = 6000;
pub const H_500: i32 = 5000;
pub const H_400: i32 = 4000;
pub const H_300: i32 = 3000;
pub const H_200: i32 = 2000;
pub const H_100: i32 = 1000;
pub const H_0: i32 = 0;

pub fn norm_score(base: i32, ori: i64) -> i32 {
    if ori < 0 {
        return base;
    }

    ori as i32
}
