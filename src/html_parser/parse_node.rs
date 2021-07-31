use combine::{
    attempt, choice,
    error::{ParseError, StreamError},
    many, many1, parser, satisfy, Parser, Stream,
};

#[allow(unused_imports)]
use combine::EasyParser;

use super::dom::{Element, Node, Text};
use super::parse_tag::{parse_end_tag, parse_start_tag};

/**
 * text Nodeとしてパース
 * htmlタグ内のテキストをパースする
 */
fn parse_text<Input>() -> impl Parser<Input, Output = Box<Node>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    many1(satisfy(|c: char| c != '<')).map(|t| Text::new(t))
}

/**
 * element Nodeとしてパース
 * @return (タグ名, 属性, 子要素)
 * 子要素: element or text
*/
pub fn parse_elements<Input>() -> impl Parser<Input, Output = Box<Node>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (parse_start_tag(), nodes(), parse_end_tag()).and_then(
        |((open_tag_name, attributes), children, close_tag_name)| {
            if open_tag_name == close_tag_name {
                Ok(Element::new(open_tag_name, attributes, children))
            } else {
                Err(<Input::Error as combine::error::ParseError<
                    char,
                    Input::Range,
                    Input::Position,
                >>::StreamError::message_static_message(
                    "tag name of open tag and close tag mismatched",
                ))
            }
        },
    )
}

/**
 * DOMツリー算出
 * elementパース、textパースを用いてツリーを構築
 */
pub fn nodes_<Input>() -> impl Parser<Input, Output = Vec<Box<Node>>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    attempt(many(choice((
        attempt(parse_elements()),
        attempt(parse_text()),
    ))))
}

parser! {
    fn nodes[Input]()(Input) -> Vec<Box<Node>>
    where [Input: Stream<Token = char>]
    {
        nodes_()
    }
}

/** ====================================================
 *   unit tests
 * ====================================================
 */
#[cfg(test)]
mod tests {
    use crate::html_parser::dom::AttrMap;

    use super::*;
    #[test]
    fn test_parse_text() {
        let predict = "hello".to_string();
        let result = parse_text().easy_parse("hello").ok().unwrap();
        dbg!(result);
        // assert_eq!(predict, result.1);
    }

    #[test]
    fn test_parse_elements() {
        assert_eq!(
            parse_elements().easy_parse("<p></p>"),
            Ok((Element::new("p".to_string(), AttrMap::new(), vec![]), ""))
        );

        assert_eq!(
            parse_elements().easy_parse("<p>hello world</p>"),
            Ok((
                Element::new(
                    "p".to_string(),
                    AttrMap::new(),
                    vec![Text::new("hello world".to_string())]
                ),
                ""
            ))
        );

        assert_eq!(
            parse_elements().easy_parse("<div><p>hello world</p></div>"),
            Ok((
                Element::new(
                    "div".to_string(),
                    AttrMap::new(),
                    vec![Element::new(
                        "p".to_string(),
                        AttrMap::new(),
                        vec![Text::new("hello world".to_string())]
                    )],
                ),
                ""
            ))
        );

        assert!(parse_elements().easy_parse("<p>hello world</div>").is_err());
    }

    /* ... */
}
