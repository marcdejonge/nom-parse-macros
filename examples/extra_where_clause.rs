//! This example demonstrates how to use an extra where clause in the `parse_from` macro
//! to add additional trait bounds to the generated parser implementation.

use nom_parse_macros::parse_from;
use nom_parse_trait::ParseFrom;

#[derive(Debug, PartialEq)]
#[parse_from(gen_array(":") where T: Default + Copy)]
struct Point<const D: usize, T> {
    pub coords: [T; D],
}

fn gen_array<const D: usize, I: nom::Input, E: nom::error::ParseError<I>, T, P>(
    mut prefix: impl nom::Parser<I, Output = P, Error = E>,
) -> impl nom::Parser<I, Output = [T; D], Error = E>
where
    T: ParseFrom<I, E> + Default + Copy,
{
    move |mut input: I| {
        let mut array = [T::default(); D];
        for ix in 0..D {
            let (rest, _) = prefix.parse(input)?;
            let (rest, value) = T::parse(rest)?;
            input = rest;
            array[ix] = value;
        }
        Ok((input, array))
    }
}

fn main() {
    let input = ":1:2:3";
    let pair: nom::IResult<_, _> = Point::<3, u32>::parse(input);
    println!("Parsed \"{}\" as {:?}", input, pair.unwrap().1);
}
