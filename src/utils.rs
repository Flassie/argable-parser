use nom::character::complete::multispace0;
use nom::error::ParseError;
use nom::sequence::delimited;
use nom::{AsChar, IResult, InputTakeAtPosition, Parser};

pub fn whitespaced<I, O1, E, F>(f: F) -> impl FnMut(I) -> IResult<I, O1, E>
where
    I: InputTakeAtPosition,
    F: Parser<I, O1, E>,
    E: ParseError<I>,
    <I as InputTakeAtPosition>::Item: AsChar + Clone,
{
    delimited(multispace0, f, multispace0)
}
