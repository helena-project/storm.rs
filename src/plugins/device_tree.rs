use syntax;
use syntax::codemap::Span;
use syntax::parse::{token, parser};
use syntax::ast::{self, TokenTree, Lit_, Ident};
use syntax::ext::base::{ExtCtxt, MacResult, MacExpr};
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
use std::ascii::OwnedAsciiExt;
use plugin_lib::*;

type QuoteStmt = syntax::ptr::P<ast::Stmt>;

const DRIVER_PATH: &'static str = "drivers";

#[derive(Debug)]
struct Resource {
    name: Ident,
    location: Option<usize>
}

impl ToString for Resource {
    fn to_string(&self) -> String {
        let mut string = String::from_str(self.name.as_str());
        string = string.into_ascii_lowercase();
        if self.location.is_some() {
            string.push_str("_");
            string.push_str(&self.location.unwrap().to_string());
        }

        string
    }
}

impl ToTokens for Resource {
    fn to_tokens(&self, cx: &ExtCtxt) -> Vec<TokenTree> {
        cx.parse_tts(self.to_string())
    }
}

#[derive(Debug)]
struct Node {
    name: Ident,
    path: SimplePath,
    resources: Vec<Resource>
}

fn parse_resources(parser: &mut parser::Parser, cx: &mut ExtCtxt, _: Span)
        -> Vec<Resource> {
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

    vec![resource]
}

fn parse_node(parser: &mut parser::Parser, cx: &mut ExtCtxt, sp: Span) -> Node {
    let item_name = parser.parse_ident();
    parser.expect(&token::Token::Colon);
    span_note!(cx, sp, "Item Name: {}", item_name);

    let path = parser.parse_path(parser::PathParsingMode::NoTypesAllowed);
    parser.expect(&token::OpenDelim(token::DelimToken::Paren));
    span_note!(cx, sp, "Path: {}", SimplePath(path.clone()).to_string());

    let resources = parse_resources(parser, cx, sp);
    span_note!(cx, sp, "Resources: {}", resources[0].to_string()); // DEBUG

    parser.expect(&token::CloseDelim(token::DelimToken::Paren));
    parser.expect(&token::Semi);

    Node {
        name: item_name,
        path: SimplePath(path),
        resources: resources
    }
}

fn statement_from_node(node: &Node, cx: &mut ExtCtxt) -> QuoteStmt {
    let name = node.name;
    let ref path = node.path;
    let ref resource = node.resources[0]; // TODO: Generalize.
    quote_stmt!(cx, let $name = $path::simple_new($resource);)
}

pub fn expand(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree])
        -> Box<MacResult + 'static> {
    let mut parser = cx.new_parser_from_tts(args);
    let driver_path_id = token::str_to_ident(DRIVER_PATH);
    let base_path_segment = ident_to_segment(&driver_path_id);

    let mut statements = vec![];
    while !parser.check(&token::Eof) {
        let mut node = parse_node(&mut parser, cx, sp);
        node.path.0.segments.insert(0, base_path_segment.clone());
        statements.push(statement_from_node(&node, cx));
    }

    let decl = quote_expr!(cx, {
        use $driver_path_id;

        // Need to get around hygenic stuff for this to work without:
        use platform::sam4l::gpio;
        let gpiopin_10 = gpio::GPIOPin::new(gpio::Params {
            location: gpio::Location::GPIOPin10,
            port: gpio::GPIOPort::GPIO2,
            function: None
        });
        // End of undoing needed.

        $statements });
    MacExpr::new(decl)
}
