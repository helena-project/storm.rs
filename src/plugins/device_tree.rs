use syntax::ptr;
use syntax::codemap::Span;
use syntax::parse::{token, parser};
use syntax::ast::{TokenTree, Lit_, Variant};
use syntax::ext::base::{ExtCtxt, MacResult, DummyResult, MacExpr};

type VariantVec = Vec<ptr::P<Variant>>;

pub fn expand(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree])
        -> Box<MacResult + 'static> {
    let mut parser = cx.new_parser_from_tts(args);

    let item_name = parser.parse_ident();
    parser.expect(&token::Token::Colon);

    // DEBUG
    span_note!(cx, sp, "Item Name: {}", item_name);

    let path = parser.parse_path(parser::PathParsingMode::NoTypesAllowed);
    parser.expect(&token::OpenDelim(token::DelimToken::Paren));

    // DEBUG
    span_note!(cx, sp, "Path: {:?}", path);

    let resource = parser.parse_ident();
    let location = if parser.eat(&token::At) {
        Some(parse_int_lit!(parser, cx, sp))
    } else {
        None
    };

    // DEBUG
    span_note!(cx, sp, "Resource {} at {:?}", resource, location);

    parser.expect(&token::CloseDelim(token::DelimToken::Paren));
    parser.expect(&token::Semi);

    let decl = quote_expr!(cx, { let $item_name = $location; });
    MacExpr::new(decl)
}
