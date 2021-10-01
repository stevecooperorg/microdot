use crate::{Command, GraphCommand, Id, Label, Line};
use pom::char_class::{alpha, alphanum, multispace};
use pom::parser::*;

/// space, tab, etc
fn ws<'a>() -> Parser<'a, u8, ()> {
    is_a(multispace).discard()
}

/// whitespace and comments
fn space<'a>() -> Parser<'a, u8, ()> {
    ws().repeat(0..).discard()
}

/// a parser wrapped in whitespace
fn spaced<'a, T>(parser: Parser<'a, u8, T>) -> Parser<'a, u8, T>
where
    T: 'a,
{
    space() * parser - space()
}

fn is_underscore(term: u8) -> bool {
    term == b'_'
}

fn id<'a>() -> Parser<'a, u8, String> {
    let it = ((is_a(alpha) | is_a(is_underscore))
        + (is_a(alphanum) | is_a(is_underscore)).repeat(0..))
    .map(|(first, rest)| format!("{}{}", first as char, String::from_utf8(rest).unwrap()));

    spaced(it).name("name")
}

fn is_cr(term: u8) -> bool {
    term == b'\r'
}

fn is_lf(term: u8) -> bool {
    term == b'\n'
}

fn label<'a>() -> Parser<'a, u8, String> {
    fn anything_else(term: u8) -> bool {
        !is_cr(term) && !is_lf(term)
    }

    is_a(anything_else)
        .repeat(1..)
        .map(|u8s| String::from_utf8(u8s).expect("can only parse utf"))
}

fn insert_node<'a>() -> Parser<'a, u8, String> {
    // i foo bar baz
    keyword(b"i") * label()
}

fn show_help<'a>() -> Parser<'a, u8, ()> {
    // i foo bar baz
    (keyword(b"help") | keyword(b"h")).discard()
}

fn print_dot<'a>() -> Parser<'a, u8, ()> {
    // i foo bar baz
    (keyword(b"print") | keyword(b"p")).discard()
}

fn print_json<'a>() -> Parser<'a, u8, ()> {
    // i foo bar baz
    (keyword(b"json") | keyword(b"j")).discard()
}

fn exit<'a>() -> Parser<'a, u8, ()> {
    // i foo bar baz
    keyword(b"exit").discard()
}

fn keyword<'a>(keyword: &'static [u8]) -> Parser<'a, u8, ()> {
    literal(keyword).discard().name("keyword")
}

fn literal<'a>(literal: &'static [u8]) -> Parser<'a, u8, String> {
    spaced(seq(literal))
        .map(|u8s| String::from_utf8(u8s.to_vec()).expect("can only parse utf"))
        .name("literal")
}

fn delete_node<'a>() -> Parser<'a, u8, String> {
    // d foo
    keyword(b"d") * id()
}

fn link_edge<'a>() -> Parser<'a, u8, (String, String)> {
    // e bar baz
    keyword(b"l") * id() + id()
}

fn unlink_edge<'a>() -> Parser<'a, u8, String> {
    // u edge1
    keyword(b"u") * id()
}

fn rename_node<'a>() -> Parser<'a, u8, (String, String)> {
    // e bar baz
    keyword(b"r") * id() + label()
}

pub fn parse_line(line: Line) -> Command {
    let text = &line.0.clone().into_bytes();
    if let Ok(res) = insert_node().parse(text) {
        return GraphCommand::InsertNode {
            label: Label::new(&res),
        }
        .into();
    }

    if let Ok(res) = delete_node().parse(text) {
        return GraphCommand::DeleteNode { id: Id::new(&res) }.into();
    }

    if let Ok((from, to)) = link_edge().parse(text) {
        return GraphCommand::LinkEdge {
            from: Id::new(&from),
            to: Id::new(&to),
        }
        .into();
    }

    if let Ok(id) = unlink_edge().parse(text) {
        return GraphCommand::UnlinkEdge { id: Id::new(&id) }.into();
    }

    if let Ok((id, label)) = rename_node().parse(text) {
        return GraphCommand::RenameNode {
            id: Id::new(&id),
            label: Label::new(&label),
        }
        .into();
    }

    if let Ok(()) = exit().parse(text) {
        return Command::Exit;
    }

    if let Ok(()) = show_help().parse(text) {
        return Command::ShowHelp;
    }

    if let Ok(()) = print_dot().parse(text) {
        return Command::PrintDot;
    }
    if let Ok(()) = print_json().parse(text) {
        return Command::PrintJson;
    }

    Command::ParseError { line }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_consumes_all {
        ( $ parser: expr, $input: expr ) => {
            let terminating_parser = $parser - space() - end();
            let res = terminating_parser.parse($input);
            if let Err(_) = res {
                panic!("parser failed to match and consume everything")
            }
        };
        ( $ parser: expr, $input: expr, $expected: expr) => {
            let terminating_parser = $parser - space() - end();
            let res = terminating_parser.parse($input);
            match res {
                Ok(answer) => {
                    // it parsed, but was it right?
                    assert_eq!(answer, $expected)
                }
                Err(_) => {
                    //
                    panic!("parser failed to match and consume everything")
                }
            }
        };
    }

    #[test]
    fn parser_bits() {
        assert_consumes_all![insert_node(), b"i foo", "foo"];
        assert_consumes_all![insert_node(), b"i foo bar baz", "foo bar baz"];
        assert_consumes_all![delete_node(), b"d foo", "foo"];
        assert_consumes_all![
            link_edge(),
            b"l f1 f2",
            ("f1".to_string(), "f2".to_string())
        ];
        assert_consumes_all![unlink_edge(), b"u e1", "e1"];
        assert_consumes_all![
            rename_node(),
            b"r f new name",
            ("f".to_string(), "new name".to_string())
        ];
        assert_consumes_all![show_help(), b"h", ()];
        assert_consumes_all![show_help(), b"help", ()];
        assert_consumes_all![print_dot(), b"p", ()];
        assert_consumes_all![print_dot(), b"print", ()];
        assert_consumes_all![print_json(), b"j", ()];
        assert_consumes_all![print_json(), b"json", ()];
        assert_consumes_all![exit(), b"exit", ()];
    }

    #[test]
    fn parse_line_works() {
        macro_rules! assert_parse_command {
            ($input: expr, $expected: expr) => {
                let line = Line::new($input);
                let actual = parse_line(line);
                assert_eq!(actual, $expected);
            };
        }

        assert_parse_command!(
            "i",
            Command::ParseError {
                line: Line::new("i")
            }
        );
        assert_parse_command!("h", Command::ShowHelp);
        assert_parse_command!("p", Command::PrintDot);
        assert_parse_command!("j", Command::PrintJson);
        assert_parse_command!("exit", Command::Exit);
        assert_parse_command!(
            "i foo",
            GraphCommand::InsertNode {
                label: Label::new("foo")
            }
            .into()
        );
        assert_parse_command!(
            "d foo",
            GraphCommand::DeleteNode { id: Id::new("foo") }.into()
        );
        assert_parse_command!(
            "l foo bar",
            GraphCommand::LinkEdge {
                from: Id::new("foo"),
                to: Id::new("bar")
            }
            .into()
        );
        assert_parse_command!(
            "u foo",
            GraphCommand::UnlinkEdge { id: Id::new("foo") }.into()
        );
        assert_parse_command!(
            "r foo a new name",
            GraphCommand::RenameNode {
                id: Id::new("foo"),
                label: Label::new("a new name")
            }
            .into()
        );
    }
}
