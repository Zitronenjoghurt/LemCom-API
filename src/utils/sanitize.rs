pub fn alphanumeric(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a() {
        assert_eq!(
            "abcdefghigklmnopqrstuvwxyzABCDEFGHIGKLMNOPQRSTUVWXYZ0123456789",
            alphanumeric("abcdefghigklmnopqrstuvwxyz 😃 ABCDEFGHIGKLMNOPQRSTUVWXYZ >-< 0123456789")
        );
    }
}
