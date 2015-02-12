use syntax;
use syntax::codemap::Span;
use syntax::parse::{token, parser};
use syntax::ast::{self, TokenTree};
use syntax::ext::base::{ExtCtxt, MacResult, MacExpr};
use plugin_utils::*;
use tree_plugin_utils::*;

type QuoteStmt = syntax::ptr::P<ast::Stmt>;

const PLATFORM_PATH: &'static str = "platform";

// TODO: Use resource location as paramter to struct::new.
fn parse_nodes(parser: &mut parser::Parser, cx: &mut ExtCtxt) -> Vec<Node> {
    let resource = parse_resource(parser, cx);
    parser.expect(&token::Colon);
    span_note!(cx, resource.span, "Resource: {}", resource);

    let path = parser.parse_path(parser::PathParsingMode::NoTypesAllowed);
    parser.expect(&token::Semi);
    span_note!(cx, parser.last_span, "Path: {}", SimplePath(path.clone()));

    let single_resources = resource.to_singles();
    single_resources.into_iter().map(|resource| {
        Node {
            name: token::str_to_ident(&resource.to_string()),
            path: SimplePath(path.clone()),
            resources: vec![],
            fields: None
        }
    }).collect()
}

fn statement_from_node(node: &Node, cx: &mut ExtCtxt) -> QuoteStmt {
    let name = node.name;
    let path = &node.path;

    // Don't allow resources for now.
    if node.fields.is_some() {
        let node_fields = node.fields.as_ref().unwrap();
        let fields = connect_tokens(&node_fields, token::Comma, cx);
        let params_path = path.clone_with_concat_terminal("Params");
        let params = quote_expr!(cx, $params_path { $fields });
        quote_stmt!(cx, let $name = $path::new($params);)
    } else {
        quote_stmt!(cx, let $name = $path::simple_new();)
    }
}

pub fn expand(cx: &mut ExtCtxt, _: Span, args: &[TokenTree])
        -> Box<MacResult + 'static> {
    let mut parser = cx.new_parser_from_tts(args);
    let driver_path_id = token::str_to_ident(PLATFORM_PATH);
    let platform_name = parser.parse_ident();
    parser.expect(&token::Comma);

    let base_path_segment = ident_to_segment(&driver_path_id);
    let platform_path_segments = ident_to_segment(&platform_name);
    let base_segments = vec![base_path_segment, platform_path_segments];

    let mut statements = vec![];
    while !parser.check(&token::Eof) {
        let mut nodes = parse_nodes(&mut parser, cx);
        for node in nodes.iter_mut() {
            canonicalize_node_paths(&base_segments, node);
            span_note!(cx, parser.last_span, "Node: {:?}", node);
            statements.push(statement_from_node(node, cx));
        }
    }

    let decl = quote_expr!(cx, {
        use $driver_path_id;
        $statements
    });

    MacExpr::new(decl)
}
