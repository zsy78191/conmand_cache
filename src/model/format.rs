pub trait CommandFormatter {
    fn format(&self, args: &[&str]) -> String;
}

pub struct SimpleFormatter;

impl CommandFormatter for SimpleFormatter {
    fn format(&self, args: &[&str]) -> String {
        args.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_formatter_basic() {
        let formatter = SimpleFormatter;
        let args = vec!["echo", "hello"];
        assert_eq!(formatter.format(&args), "echo hello");
    }

    #[test]
    fn test_simple_formatter_empty() {
        let formatter = SimpleFormatter;
        assert_eq!(formatter.format(&[]), "");
    }
}
