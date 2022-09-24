use crate::item::{Arg, Item, Placeholder};
use crate::utils::whitespaced;
use crate::values::{boolean, number, string};
use nom::branch::alt;
use nom::bytes::complete::{escaped_transform, tag};
use nom::character::complete::{alpha1, alphanumeric1, char, none_of};
use nom::combinator::{eof, map, recognize};
use nom::multi::{many0_count, many_till, separated_list0};
use nom::sequence::{delimited, pair, preceded, separated_pair};
use nom::IResult;

pub mod item;
mod utils;
mod values;

pub fn parse(input: &str) -> anyhow::Result<Vec<Item>> {
    let text = map(text, Item::Text);
    let placeholder = map(placeholder, Item::Placeholder);

    let item = alt((text, placeholder));
    let (items, _) = many_till(item, eof)(input)
        .map_err(|err| err.map(|err| nom::error::Error::new(err.input.to_string(), err.code)))?
        .1;

    Ok(items)
}

pub fn parse_iter(input: &str) -> impl Iterator<Item = Item> {
    let text = map(text, Item::Text);
    let placeholder = map(placeholder, Item::Placeholder);

    let item = alt((text, placeholder));

    struct Iter<'a, F>
    where
        F: FnMut(&'a str) -> IResult<&'a str, Item>,
    {
        input: &'a str,
        f: F,
    }

    impl<'a, F> Iterator for Iter<'a, F>
    where
        F: FnMut(&'a str) -> IResult<&'a str, Item>,
    {
        type Item = Item<'a>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.input.is_empty() {
                return None;
            }

            let (rest, item) = (self.f)(self.input).ok()?;
            self.input = rest;

            Some(item)
        }
    }

    Iter { input, f: item }
}

pub(crate) fn identifier(input: &str) -> IResult<&str, &str> {
    whitespaced(recognize(pair(
        alt((alpha1, tag("_"))),
        many0_count(alt((alphanumeric1, tag("_")))),
    )))(input)
}

pub(crate) fn flag_or_value(input: &str) -> IResult<&str, Arg> {
    let types = alt((boolean, number, string));
    let pairs = separated_pair(identifier, char('='), types);

    let values = map(pairs, |v| Arg::Value(v.0, v.1));
    let flags = map(identifier, Arg::Flag);

    alt((values, flags))(input)
}

pub(crate) fn args(input: &str) -> IResult<&str, Vec<Arg>> {
    separated_list0(char(','), flag_or_value)(input)
}

pub(crate) fn placeholder(input: &str) -> IResult<&str, Placeholder> {
    let args = delimited(char('('), args, char(')'));
    let with_args = map(pair(identifier, args), Placeholder::from);
    let without_args = map(identifier, Placeholder::from);

    let placeholder = alt((with_args, without_args));

    preceded(char('$'), placeholder)(input)
}

pub(crate) fn text(input: &str) -> IResult<&str, String> {
    escaped_transform(none_of("\\$"), '\\', char('$'))(input)
}

#[cfg(test)]
mod tests {
    use crate::item::Value;
    use crate::{parse, Arg};
    use nom::combinator::eof;
    use nom::multi::many_till;

    fn args_equals(arg1: &Arg, arg2: &Arg) -> bool {
        match (arg1, arg2) {
            (Arg::Flag(name1), Arg::Flag(name2)) => name1 == name2,
            (Arg::Value(name1, value1), Arg::Value(name2, value2)) => {
                name1 == name2 && value1 == value2
            }
            _ => false,
        }
    }

    #[test]
    fn identifier() {
        let inputs = [
            ("_hello", true),
            ("hello", true),
            ("hello_", true),
            ("hello_world1", true),
            ("1hello", false),
        ];

        for (input, expected) in inputs {
            let result = many_till(super::identifier, eof)(input);
            assert_eq!(result.is_ok(), expected);
        }
    }

    #[test]
    fn flag_or_value() {
        let inputs = [
            ("flag", true),
            ("_flag", true),
            ("_value = 1", true),
            ("1flag", false),
            ("1value = 1", false),
        ];

        for (input, expected) in inputs {
            let result = many_till(super::flag_or_value, eof)(input);
            assert_eq!(result.is_ok(), expected);
        }
    }

    #[test]
    fn args() {
        let input = "a='123', b=123, c=true, d =    false, e = 'true',   f    = true, e = 1.23, neg = -1, negf = -1.23";
        let (rest, args) = super::args(input).unwrap();

        assert_eq!(rest, "");
        assert_eq!(args.len(), 9);
        assert!(args_equals(
            &args[0],
            &Arg::Value("a", Value::String("123".to_string()))
        ));
        assert!(args_equals(&args[1], &Arg::Value("b", Value::Integer(123))));
        assert!(args_equals(
            &args[2],
            &Arg::Value("c", Value::Boolean(true))
        ));
        assert!(args_equals(
            &args[3],
            &Arg::Value("d", Value::Boolean(false))
        ));
        assert!(args_equals(
            &args[4],
            &Arg::Value("e", Value::String("true".to_string()))
        ));
        assert!(args_equals(
            &args[5],
            &Arg::Value("f", Value::Boolean(true))
        ));
        assert!(args_equals(&args[6], &Arg::Value("e", Value::Float(1.23))));
        assert!(args_equals(
            &args[7],
            &Arg::Value("neg", Value::Integer(-1))
        ));
        assert!(args_equals(
            &args[8],
            &Arg::Value("negf", Value::Float(-1.23))
        ));
    }

    #[test]
    fn placeholder() {
        let inputs = [
            ("$placeholder", true),
            ("$placeholder_with_args(arg = 1)", true),
            ("$placeholder_empty_args()", true),
            ("\\$placeholder", false),
            ("text", false),
        ];

        for (input, expected) in inputs {
            let result = many_till(super::placeholder, eof)(input);
            assert_eq!(result.is_ok(), expected);
        }
    }

    #[test]
    fn text() {
        let inputs = [("\\$", true), ("$", false), ("\\$$", false)];

        for (input, expected) in inputs {
            let result = many_till(super::text, eof)(input);

            assert_eq!(result.is_ok(), expected);
        }
    }
}
