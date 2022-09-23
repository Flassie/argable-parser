#[derive(Debug, PartialEq)]
pub enum Value {
    String(String),
    Boolean(bool),
    Integer(i32),
    Float(f32),
}

#[derive(Debug)]
pub enum Arg<'a> {
    Flag(&'a str),
    Value(&'a str, Value),
}

#[derive(Debug)]
pub struct Placeholder<'a> {
    pub name: &'a str,
    pub args: Option<Vec<Arg<'a>>>,
}

impl<'a> From<(&'a str, Vec<Arg<'a>>)> for Placeholder<'a> {
    fn from(v: (&'a str, Vec<Arg<'a>>)) -> Self {
        Self {
            name: v.0,
            args: Some(v.1),
        }
    }
}

impl<'a> From<&'a str> for Placeholder<'a> {
    fn from(v: &'a str) -> Self {
        Self {
            name: v,
            args: None,
        }
    }
}

#[derive(Debug)]
pub enum Item<'a> {
    Text(String),
    Placeholder(Placeholder<'a>),
}
