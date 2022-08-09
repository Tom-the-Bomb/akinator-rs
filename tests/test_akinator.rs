#[cfg(test)]
mod tests {
    use akinator_rs::Akinator;

    #[test]
    fn test_akinator() {
        let mut akinator = Akinator::new();

        akinator.start().unwrap();
    }
}