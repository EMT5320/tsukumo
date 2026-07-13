/// Returns the deterministic fixture value.
pub fn fixture_value() -> u32 {
    42
}

#[cfg(test)]
mod tests {
    use super::fixture_value;

    #[test]
    fn fixture_is_stable() {
        assert_eq!(fixture_value(), 42);
    }
}
