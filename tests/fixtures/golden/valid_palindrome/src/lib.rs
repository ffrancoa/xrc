// Valid Palindrome
//
// A phrase is a palindrome if, after converting all uppercase letters into lowercase
// letters and removing all non-alphanumeric characters, it reads the same forward and
// backward. Alphanumeric characters include letters and numbers.
//
// Given a string `s`, return `true` if it is a palindrome, or `false` otherwise.
//
// Constraints
// -----------
// 1 <= `s.length` <= 2 * 10^5
//
// `s` consists only of printable ASCII characters.

pub fn is_palindrome(s: String) -> bool {
    todo!("pending solution!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_1() {
        let s = "A man;
        let expected = true;
        // "amanaplanacanalpanama" is a palindrome.
        assert_eq!(is_palindrome(s), expected);
    }

    #[test]
    fn example_2() {
        let s = "race a car";
        let expected = false;
        // "raceacar" is not a palindrome.
        assert_eq!(is_palindrome(s), expected);
    }

    #[test]
    fn example_3() {
        let s = " ";
        let expected = true;
        // s is an empty string "" after removing non-alphanumeric characters.
        assert_eq!(is_palindrome(s), expected);
    }
}
