use combine::{
    attempt, choice, many1,
    parser::char::{char, digit, letter, string},
    sep_end_by, ParseError, Parser, Stream,
};

use crate::core::utils::whitespaces;
use crate::core::cssom::{CSSValue, Declaration, Unit};

pub fn declarations<Input>() -> impl Parser<Input, Output = Vec<Declaration>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    sep_end_by(
        declaration().skip(whitespaces()),
        char(';').skip(whitespaces()),
    )
}

fn declaration<Input>() -> impl Parser<Input, Output = Declaration>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (
        many1(letter().or(char('-'))).skip(whitespaces()),
        char(':').skip(whitespaces()),
        css_value(),
    )
        .map(|(k, _, v)| Declaration { name: k, value: v })
}

fn css_value<Input>() -> impl Parser<Input, Output = CSSValue>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    let keyword = many1(letter()).map(|s| CSSValue::Keyword(s));

    let em_length = (
        many1(digit()).map(|s: String| s.parse::<usize>().unwrap()),
        string("em"),
    )
        .map(|(num, _unit)| CSSValue::Length((num, Unit::Em)));

    let px_length = (
        many1(digit()).map(|s: String| s.parse::<usize>().unwrap()),
        string("px"),
    )
        .map(|(num, _unit)| CSSValue::Length((num, Unit::Px)));

    let percent_length = (
        many1(digit()).map(|s: String| s.parse::<usize>().unwrap()),
        string("%"),
    )
        .map(|(num, _unit)| CSSValue::Length((num, Unit::Percent)));

    choice((keyword, attempt(em_length).or(px_length).or(percent_length)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_value() {
        assert_eq!(
            css_value().parse("white"),
            Ok((CSSValue::Keyword("white".to_string()), ""))
        );
        assert_eq!(
            css_value().parse("100em"),
            Ok((CSSValue::Length((100usize, Unit::Em)), ""))
        );
        assert_eq!(
            css_value().parse("100px"),
            Ok((CSSValue::Length((100usize, Unit::Px)), ""))
        );

        // FIXME: ?????????
        // assert_eq!(
        //     css_value().parse("100%"),
        //     Ok((CSSValue::Length((100usize, Unit::Percent)), ""))
        // );
    }

    #[test]
    fn test_declaration() {
        assert_eq!(
            declaration().parse("width: 100px;"),
            Ok((
                Declaration {
                    name: "width".to_string(),
                    value: CSSValue::Length((100usize, Unit::Px))
                },
                ";"
            ))
        );
    }

    #[test]
    fn test_declarations() {
        assert_eq!(
            declarations().parse("foo: bar; piyo: piyopiyo;"),
            Ok((
                vec![
                    Declaration {
                        name: "foo".to_string(),
                        value: CSSValue::Keyword("bar".to_string())
                    },
                    Declaration {
                        name: "piyo".to_string(),
                        value: CSSValue::Keyword("piyopiyo".to_string())
                    }
                ],
                ""
            ))
        );

        assert_eq!(
            declarations().parse("width: 1024px; font-size: 12px;"),
            Ok((
                vec![
                    Declaration {
                        name: "width".to_string(),
                        value: CSSValue::Length((1024usize, Unit::Px))
                    },
                    Declaration {
                        name: "font-size".to_string(),
                        value: CSSValue::Length((12usize, Unit::Px))
                    }
                ],
                ""
            ))
        );
    }
}
