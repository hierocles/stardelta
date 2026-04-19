use std::cmp::max;

pub fn get_u32_bit_count(value: u32) -> u32 {
  32 - value.leading_zeros()
}

pub fn get_i32_bit_count(value: i32) -> u32 {
  match value {
    0 => 0,
    v if v < 0 => 33 - (!v).leading_zeros(),
    v => 33 - v.leading_zeros()
  }
}

pub fn get_u32_min_bit_count<I: Iterator<Item = u32>>(values: I) -> u32 {
  values.map(get_u32_bit_count).fold(0, max)
}

pub fn get_i32_min_bit_count<I: Iterator<Item = i32>>(values: I) -> u32 {
  values.map(get_i32_bit_count).fold(0, max)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_get_u32_bit_count() {
    assert_eq!(get_u32_bit_count(0), 0);
    assert_eq!(get_u32_bit_count(1), 1);
    assert_eq!(get_u32_bit_count(2), 2);
    assert_eq!(get_u32_bit_count(3), 2);
    assert_eq!(get_u32_bit_count(4), 3);
    assert_eq!(get_u32_bit_count(5), 3);
    assert_eq!(get_u32_bit_count(7), 3);
    assert_eq!(get_u32_bit_count(8), 4);
    assert_eq!(get_u32_bit_count(15), 4);
    assert_eq!(get_u32_bit_count(16), 5);
    assert_eq!(get_u32_bit_count(2147483647), 31);
  }

  #[test]
  fn test_get_i32_bit_count() {
    assert_eq!(get_i32_bit_count(0), 0);
    assert_eq!(get_i32_bit_count(1), 2);
    assert_eq!(get_i32_bit_count(2), 3);
    assert_eq!(get_i32_bit_count(3), 3);
    assert_eq!(get_i32_bit_count(4), 4);
    assert_eq!(get_i32_bit_count(5), 4);
    assert_eq!(get_i32_bit_count(7), 4);
    assert_eq!(get_i32_bit_count(8), 5);
    assert_eq!(get_i32_bit_count(15), 5);
    assert_eq!(get_i32_bit_count(16), 6);
    assert_eq!(get_i32_bit_count(2147483647), 32);
    assert_eq!(get_i32_bit_count(-1), 1);
    assert_eq!(get_i32_bit_count(-2), 2);
    assert_eq!(get_i32_bit_count(-3), 3);
    assert_eq!(get_i32_bit_count(-4), 3);
    assert_eq!(get_i32_bit_count(-5), 4);
    assert_eq!(get_i32_bit_count(-2147483648), 32);
  }

  #[test]
  fn test_get_u32_min_bit_count() {
    assert_eq!(get_u32_min_bit_count(vec![].into_iter()), 0);
    assert_eq!(get_u32_min_bit_count(vec![0].into_iter()), 0);
    assert_eq!(get_u32_min_bit_count(vec![0, 0].into_iter()), 0);
    assert_eq!(get_u32_min_bit_count(vec![1].into_iter()), 1);
    assert_eq!(get_u32_min_bit_count(vec![1, 1].into_iter()), 1);
    assert_eq!(get_u32_min_bit_count(vec![0, 1].into_iter()), 1);
    assert_eq!(get_u32_min_bit_count(vec![1, 0].into_iter()), 1);
    assert_eq!(get_u32_min_bit_count(vec![0, 0, 2, 3].into_iter()), 2);
    assert_eq!(get_u32_min_bit_count(vec![4, 0, 2, 3].into_iter()), 3);
    assert_eq!(get_u32_min_bit_count(vec![2, 1, 2].into_iter()), 2);
    assert_eq!(get_u32_min_bit_count(vec![2147483647, 3, 0, 1000].into_iter()), 31);
  }

  #[test]
  fn test_get_i32_min_bit_count() {
    assert_eq!(get_i32_min_bit_count(vec![].into_iter()), 0);
    assert_eq!(get_i32_min_bit_count(vec![0].into_iter()), 0);
    assert_eq!(get_i32_min_bit_count(vec![-1].into_iter()), 1);
    assert_eq!(get_i32_min_bit_count(vec![0, -1].into_iter()), 1);
    assert_eq!(get_i32_min_bit_count(vec![-1, 0].into_iter()), 1);
    assert_eq!(get_i32_min_bit_count(vec![16, 0, -5].into_iter()), 6);
    assert_eq!(get_i32_min_bit_count(vec![2147483647, -2147483648].into_iter()), 32);
  }
}
