use syntax::ptr;
use syntax::codemap::Span;
use syntax::parse::{token, parser};
use syntax::ast::{TokenTree, Lit_, Ident, Path};
use syntax::ext::base::{ExtCtxt, MacResult, DummyResult, MacExpr};

#[derive(Show)]
struct Resource {
    name: Ident,
    location: Option<usize>
}

#[derive(Show)]
struct Node {
    name: Ident,
    path: Path,
    resources: Vec<Resource>
}

fn parse_node(parser: &mut parser::Parser, cx: &mut ExtCtxt, sp: Span)
        -> Option<Node> {
    let item_name = parser.parse_ident();
    parser.expect(&token::Token::Colon);

    // DEBUG
    span_note!(cx, sp, "Item Name: {}", item_name);

    let path = parser.parse_path(parser::PathParsingMode::NoTypesAllowed);
    parser.expect(&token::OpenDelim(token::DelimToken::Paren));

    // DEBUG
    span_note!(cx, sp, "Path: {:?}", path);

    let resource_name = parser.parse_ident();
    let resource_location = if parser.eat(&token::At) {
        Some(parse_int_lit!(parser, cx, sp) as usize)
    } else {
        None
    };

    let resource = Resource {
        name: resource_name,
        location: resource_location
    };

    // DEBUG
    span_note!(cx, sp, "Resource {:?}", resource);

    parser.expect(&token::CloseDelim(token::DelimToken::Paren));
    parser.expect(&token::Semi);

    Some(Node {
        name: item_name,
        path: path,
        resources: vec![resource]
    })
}

pub fn expand(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree])
        -> Box<MacResult + 'static> {
    let mut parser = cx.new_parser_from_tts(args);

    let mut statements = vec![];
    while !parser.check(&token::Eof) {
        if let Some(node) = parse_node(&mut parser, cx, sp) {
            let (name, location) = (node.name, node.resources[0].location);
            statements.push(quote_stmt!(cx, let $name = $location; ));
        } else {
            break;
        }
    }

    let decl = quote_expr!(cx, { $statements });
    MacExpr::new(decl)
}
