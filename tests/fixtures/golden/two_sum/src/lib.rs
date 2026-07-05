// Two Sum
//
// Given an array of integers `nums` and an integer `target`, return indices of the two
// numbers such that they add up to `target`.
//
// You may assume that each input would have exactly one solution, and you may not use
// the same element twice.
//
// You can return the answer in any order.
//
// Constraints
// -----------
// 2 <= `nums.length` <= 10^4
// -10^9 <= `nums[i]` <= 10^9
// -10^9 <= `target` <= 10^9
//
// Only one valid answer exists.

pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
    todo!("pending solution!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_1() {
        let nums = vec![2, 7, 11, 15];
        let target = 9;
        let expected = vec![0, 1];
        // Because nums[0] + nums[1] == 9, we return [0, 1].
        assert_eq!(two_sum(nums, target), expected);
    }

    #[test]
    fn example_2() {
        let nums = vec![3, 2, 4];
        let target = 6;
        let expected = vec![1, 2];
        assert_eq!(two_sum(nums, target), expected);
    }

    #[test]
    fn example_3() {
        let nums = vec![3, 3];
        let target = 6;
        let expected = vec![0, 1];
        assert_eq!(two_sum(nums, target), expected);
    }
}
