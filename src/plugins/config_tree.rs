use platform_tree;
use device_tree;
use syntax::codemap::Span;
use syntax::parse::{token, parser};
use syntax::ast::{TokenTree};
use syntax::ext::base::{ExtCtxt, MacResult, MacExpr};

// Returns the indexes of matching braces, one after the opening brace and one
// at the closing braces { here ... to_}_here
fn find_matching_braces(parser: &mut parser::Parser) -> (usize, usize) {
    parser.expect(&token::OpenDelim(token::DelimToken::Brace));
    let start = parser.tokens_consumed;

    let mut count = 1;
    while count != 0 {
        let token = parser.bump_and_get();
        match token {
            token::CloseDelim(token::DelimToken::Brace) => count -= 1,
            token::OpenDelim(token::DelimToken::Brace) => count += 1,
            _ => { }
        }
    }

    let end = parser.tokens_consumed - 1;
    (start, end)
}

pub fn expand(cx: &mut ExtCtxt, _: Span, args: &[TokenTree])
        -> Box<MacResult + 'static> {
    let mut parser = cx.new_parser_from_tts(args);
    let driver_path_id = token::str_to_ident(device_tree::DRIVER_PATH);
    let platform_path_id = token::str_to_ident(platform_tree::PLATFORM_PATH);

    let mut parsed_sub_trees = vec![];
    while !parser.check(&token::Eof) {
        let item_ident = parser.parse_ident();
        let item_name = item_ident.as_str();
        let (start, end) = find_matching_braces(&mut parser);

        span_note!(cx, parser.last_span, "Range: ({}, {})", start, end);
        let mut new_parser = cx.new_parser_from_tts(args);
        let parsed_sub_tree = match item_name {
            "platform" => platform_tree::parse(&mut new_parser, cx, start, end),
            "devices" => device_tree::parse(&mut new_parser, cx, start, end),
            _ => cx.span_fatal(parser.last_span, "Unrecognized subtree name.")
        };

        parsed_sub_trees.push(parsed_sub_tree);
    }

    let decl = quote_expr!(cx, {
        use $driver_path_id;
        use $platform_path_id;

        $parsed_sub_trees
    });

    MacExpr::new(decl)
}
