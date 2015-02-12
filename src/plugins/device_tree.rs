use syntax;
use syntax::codemap::Span;
use syntax::parse::{token, parser};
use syntax::ast::{self, TokenTree};
use syntax::ext::base::{ExtCtxt, MacResult, MacExpr};
use plugin_utils::*;
use tree_plugin_utils::*;

type QuoteStmt = syntax::ptr::P<ast::Stmt>;

const DRIVER_PATH: &'static str = "drivers";

fn parse_node(parser: &mut parser::Parser, cx: &mut ExtCtxt) -> Node {
    let mut node_span = parser.span.clone();
    let item_name = parser.parse_ident();
    parser.expect(&token::Colon);
    span_note!(cx, parser.last_span, "Item Name: {}", item_name);

    let path = parser.parse_path(parser::PathParsingMode::NoTypesAllowed);
    parser.expect(&token::OpenDelim(token::DelimToken::Paren));
    span_note!(cx, parser.last_span, "Path: {}", SimplePath(path.clone()));

    let resources = parse_resources(parser, cx);
    span_note!(cx, parser.last_span, "Resources: {:?}", resources);
    span_note!(cx, resources[0].span, "Resource[0]: {}", resources[0]);

    parser.expect(&token::CloseDelim(token::DelimToken::Paren));
    let fields = if parser.eat(&token::OpenDelim(token::DelimToken::Brace)) {
        let parsed_fields = parse_fields(parser);
        // span_note!(cx, parser.last_span, "Fields: {:?}", parsed_fields);
        parser.expect(&token::CloseDelim(token::DelimToken::Brace));
        Some(parsed_fields)
    } else {
        parser.expect(&token::Semi);
        None
    };

    node_span.hi = parser.span.lo;

    Node {
        name: item_name,
        path: SimplePath(path),
        resources: resources,
        fields: fields,
        span: node_span
    }
}

fn statement_from_node(node: &Node, cx: &mut ExtCtxt) -> QuoteStmt {
    let name = node.name;
    let path = &node.path;
    let resources = connect_tokens(&node.resources, token::Comma, cx);

    if node.fields.is_some() {
        let node_fields = node.fields.as_ref().unwrap();
        let fields = connect_tokens(&node_fields, token::Comma, cx);
        let params_path = path.clone_with_concat_terminal("Params");
        let params = quote_expr!(cx, $params_path { $fields });
        quote_stmt!(cx, let $name = $path::new($resources, $params);)
    } else {
        quote_stmt!(cx, let $name = $path::new($resources);)
    }
}

pub fn expand(cx: &mut ExtCtxt, _: Span, args: &[TokenTree])
        -> Box<MacResult + 'static> {
    let mut parser = cx.new_parser_from_tts(args);
    let driver_path_id = token::str_to_ident(DRIVER_PATH);
    let base_path_segment = ident_to_segment(&driver_path_id);
    let base_segments = vec![base_path_segment];

    let mut statements = vec![];
    while !parser.check(&token::Eof) {
        let mut node = parse_node(&mut parser, cx);
        canonicalize_node_paths(&base_segments, &mut node);
        // span_note!(cx, parser.last_span, "Node: {:?}", node);
        statements.push(statement_from_node(&node, cx));
    }

    let decl = quote_expr!(cx, {
        use $driver_path_id;

        // Need to get around hygenic stuff for this to work without:
        use platform::sam4l::{gpio, usart};
        let gpiopin_10 = gpio::GPIOPin::new(gpio::GPIOPinParams {
            location: gpio::Location::GPIOPin10,
            port: gpio::GPIOPort::GPIO2,
            function: None
        });

        let uart_3 = usart::USART::new(usart::USARTParams {
            location: usart::Location::USART3
        });
        // End of undoing needed.

        $statements
    });

    MacExpr::new(decl)
}
