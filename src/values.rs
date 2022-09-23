use crate::item::Value;
use crate::utils::whitespaced;
use nom::branch::alt;
use nom::bytes::complete::{escaped_transform, tag};
use nom::character::complete::{char, none_of};
use nom::combinator::map;
use nom::number::complete::recognize_float;
use nom::sequence::delimited;
use nom::IResult;

pub fn boolean(input: &str) -> IResult<&str, Value> {
    // Help Clion to infer the type of `value`.
    let (rest, value): (_, &str) = whitespaced(alt((tag("true"), tag("false"))))(input)?;

    Ok((rest, Value::Boolean(value.parse().unwrap())))
}

pub fn number(input: &str) -> IResult<&str, Value> {
    // Help Clion to infer the type of `value`.
    let (rest, value): (_, &str) = whitespaced(recognize_float)(input)?;

    Ok(value
        .parse::<i32>()
        .map(|i| (rest, Value::Integer(i)))
        .unwrap_or_else(|_| (rest, Value::Float(value.parse().unwrap()))))
}

pub fn string(input: &str) -> IResult<&str, Value> {
    let escaped = escaped_transform(none_of("\\'"), '\\', char('\''));

    let empty = map(tag(""), |_| String::new());
    let inner = alt((escaped, empty));
    let delimited = delimited(tag("'"), inner, tag("'"));
    let (rest, m) = map(whitespaced(delimited), Value::String)(input)?;

    Ok((rest, m))
}

#[cfg(test)]
mod tests {
    use crate::item::Value;
    use nom::combinator::eof;
    use nom::multi::many_till;

    #[test]
    fn boolean() {
        let (rest, value) = super::boolean("true").unwrap();
        assert_eq!(rest, "");
        assert_eq!(value, Value::Boolean(true));

        let (rest, value) = super::boolean("false").unwrap();
        assert_eq!(rest, "");
        assert_eq!(value, Value::Boolean(false));

        let (rest, value) = super::boolean("true  ").unwrap();
        assert_eq!(rest, "");
        assert_eq!(value, Value::Boolean(true));

        let (rest, value) = super::boolean("  false").unwrap();
        assert_eq!(rest, "");
        assert_eq!(value, Value::Boolean(false));

        let v = super::boolean("  test");
        assert!(v.is_err())
    }

    #[test]
    fn number() {
        let inputs = [
            ("0", Some(Value::Integer(0))),
            (" 1", Some(Value::Integer(1))),
            ("1.0", Some(Value::Float(1.0))),
            ("   1.1", Some(Value::Float(1.1))),
            ("0", Some(Value::Integer(0))),
            ("-1", Some(Value::Integer(-1))),
            ("-1.0", Some(Value::Float(-1.0))),
            (" -1.1", Some(Value::Float(-1.1))),
            ("1.1.1", None),
            ("test", None),
            ("-test", None),
        ];

        for (input, expected) in inputs {
            let result = super::number(input);

            match expected {
                Some(expected) => {
                    let (rest, value) = result.unwrap();
                    assert_eq!(rest, "");
                    assert_eq!(value, expected);
                }
                None => {
                    assert!(result.is_err() || !result.unwrap().0.is_empty());
                }
            }
        }
    }

    #[test]
    fn string() {
        let inputs = [
            (r#"'asd \' '"#, Some("asd ' "), true),
            (r#"'Hello \'World\''"#, Some("Hello 'World'"), true),
            (r#"'asd"#, None, false),
        ];

        for (input, check, expected) in inputs {
            let result = many_till(super::string, eof)(input);
            assert_eq!(result.is_ok(), expected);

            if let Some(check) = check {
                let (_, (values, _)) = result.unwrap();

                assert_eq!(values.len(), 1);
                assert_eq!(values[0], Value::String(check.to_string()), "");
            }
        }
    }
}
