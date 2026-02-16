/// Simple expression calculator for the launcher.
/// Supports: +, -, *, /, parentheses, decimals.
pub fn evaluate(expr: &str) -> Option<f64> {
    let expr = expr.trim();
    if expr.is_empty() {
        return None;
    }

    // Quick check: must start with a digit, minus, or paren
    let first = expr.chars().next()?;
    if !first.is_ascii_digit() && first != '-' && first != '(' {
        return None;
    }

    // Must contain at least one operator to be considered a math expression
    if !expr.contains(['+', '-', '*', '/', '('])
        || expr
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
    {
        return None;
    }

    parse_expr(&mut expr.chars().peekable())
}

type Chars<'a> = std::iter::Peekable<std::str::Chars<'a>>;

fn parse_expr(chars: &mut Chars) -> Option<f64> {
    let mut result = parse_term(chars)?;
    loop {
        skip_spaces(chars);
        match chars.peek() {
            Some('+') => {
                chars.next();
                result += parse_term(chars)?;
            }
            Some('-') => {
                chars.next();
                result -= parse_term(chars)?;
            }
            _ => break,
        }
    }
    Some(result)
}

fn parse_term(chars: &mut Chars) -> Option<f64> {
    let mut result = parse_factor(chars)?;
    loop {
        skip_spaces(chars);
        match chars.peek() {
            Some('*') => {
                chars.next();
                result *= parse_factor(chars)?;
            }
            Some('/') => {
                chars.next();
                let divisor = parse_factor(chars)?;
                if divisor == 0.0 {
                    return None; // Division by zero
                }
                result /= divisor;
            }
            _ => break,
        }
    }
    Some(result)
}

fn parse_factor(chars: &mut Chars) -> Option<f64> {
    skip_spaces(chars);
    match chars.peek()? {
        '(' => {
            chars.next(); // consume '('
            let result = parse_expr(chars)?;
            skip_spaces(chars);
            if chars.peek() == Some(&')') {
                chars.next(); // consume ')'
            }
            Some(result)
        }
        '-' => {
            chars.next();
            Some(-parse_factor(chars)?)
        }
        _ => parse_number(chars),
    }
}

fn parse_number(chars: &mut Chars) -> Option<f64> {
    skip_spaces(chars);
    let mut s = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() || c == '.' {
            s.push(c);
            chars.next();
        } else {
            break;
        }
    }
    s.parse().ok()
}

fn skip_spaces(chars: &mut Chars) {
    while chars.peek() == Some(&' ') {
        chars.next();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_addition() {
        assert_eq!(evaluate("2 + 3"), Some(5.0));
    }

    #[test]
    fn operator_precedence() {
        assert_eq!(evaluate("2 + 3 * 4"), Some(14.0));
    }

    #[test]
    fn parentheses() {
        assert_eq!(evaluate("(2 + 3) * 4"), Some(20.0));
    }

    #[test]
    fn decimals() {
        assert_eq!(evaluate("1.5 * 2.5"), Some(3.75));
    }

    #[test]
    fn division() {
        assert_eq!(evaluate("10 / 4"), Some(2.5));
    }

    #[test]
    fn negative() {
        assert_eq!(evaluate("-5 + 3"), Some(-2.0));
    }

    #[test]
    fn division_by_zero() {
        assert_eq!(evaluate("1 / 0"), None);
    }

    #[test]
    fn not_an_expression() {
        assert_eq!(evaluate("hello"), None);
        assert_eq!(evaluate("42"), None); // just a number, not an expression
    }

    #[test]
    fn complex_expression() {
        let result = evaluate("(10 + 5) * 2 - 3 / 1.5").unwrap();
        assert!((result - 28.0).abs() < 0.001);
    }
}
