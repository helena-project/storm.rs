use syntax;
use syntax::codemap::Span;
use syntax::parse::{token, parser};
use syntax::ast::{self, TokenTree, Lit_, Ident, Field, PathSegment};
use syntax::ext::base::{ExtCtxt, MacResult, MacExpr};
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
use std::ascii::OwnedAsciiExt;
use syntax::fold::Folder;
use plugin_util::*;

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
    resources: Vec<Resource>,
    fields: Option<Vec<Field>>
}

fn parse_resources(parser: &mut parser::Parser, cx: &mut ExtCtxt) -> Vec<Resource> {
    let mut resources = vec![];
    loop {
        let resource_name = parser.parse_ident();
        let resource_location = if parser.eat(&token::At) {
            Some(parse_int_lit!(parser, cx, sp) as usize)
        } else {
            None
        };

        resources.push(Resource {
            name: resource_name,
            location: resource_location
        });

        if !parser.eat(&token::Comma) {
            break;
        }
    }

    resources
}

fn parse_fields(parser: &mut parser::Parser) -> Vec<Field> {
    let mut fields = vec![];
    loop {
        fields.push(parser.parse_field());
        if !parser.eat(&token::Comma) {
            break;
        }
    }

    fields
}

fn parse_node(parser: &mut parser::Parser, cx: &mut ExtCtxt) -> Node {
    let item_name = parser.parse_ident();
    parser.expect(&token::Token::Colon);
    span_note!(cx, parser.last_span, "Item Name: {}", item_name);

    let path = parser.parse_path(parser::PathParsingMode::NoTypesAllowed);
    parser.expect(&token::OpenDelim(token::DelimToken::Paren));
    span_note!(cx, parser.last_span, "Path: {}", SimplePath(path.clone()).to_string());

    let resources = parse_resources(parser, cx);
    span_note!(cx, parser.last_span, "Resources: {:?}", resources);

    parser.expect(&token::CloseDelim(token::DelimToken::Paren));
    let fields = if parser.eat(&token::OpenDelim(token::DelimToken::Brace)) {
        let parsed_fields = parse_fields(parser);
        span_note!(cx, parser.last_span, "Fields: {:?}", parsed_fields);
        parser.expect(&token::CloseDelim(token::DelimToken::Brace));
        Some(parsed_fields)
    } else {
        parser.expect(&token::Semi);
        None
    };

    Node {
        name: item_name,
        path: SimplePath(path),
        resources: resources,
        fields: fields
    }
}

fn statement_from_node(node: &Node, cx: &mut ExtCtxt) -> QuoteStmt {
    let name = node.name;
    let ref path = node.path;
    let resources = connect_tokens(&node.resources, token::Comma, cx);
    // let fields = node.fields.unwrap();
    quote_stmt!(cx, let $name = $path::simple_new($resources);)
}

fn canonicalize_node_paths(base_segment: &PathSegment, node: &mut Node) {
    node.path.0.segments.insert(0, base_segment.clone());

    if let Some(fields) = node.fields.take() {
        let device_segments = node.path.without_terminal().0.segments;
        let mut prepender = PathPrepender::new(device_segments);
        let new_fields = fields.map_in_place(|field| {
            prepender.fold_field(field)
        });

        node.fields = Some(new_fields);
    }
}

pub fn expand(cx: &mut ExtCtxt, _: Span, args: &[TokenTree])
        -> Box<MacResult + 'static> {
    let mut parser = cx.new_parser_from_tts(args);
    let driver_path_id = token::str_to_ident(DRIVER_PATH);
    let base_path_segment = ident_to_segment(&driver_path_id);

    let mut statements = vec![];
    while !parser.check(&token::Eof) {
        let mut node = parse_node(&mut parser, cx);
        canonicalize_node_paths(&base_path_segment, &mut node);
        span_note!(cx, parser.last_span, "Node: {:?}", node);
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

        $statements
    });

    MacExpr::new(decl)
}
