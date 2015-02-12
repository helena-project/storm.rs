use syntax;
use syntax::codemap::Span;
use syntax::parse::{token, parser};
use syntax::ast::{self, TokenTree};
use syntax::ext::base::{ExtCtxt, MacResult, MacExpr};
use plugin_utils::*;
use tree_plugin_utils::*;

type QuoteStmt = syntax::ptr::P<ast::Stmt>;

const PLATFORM_PATH: &'static str = "platform";

fn mk_location_field(path: &SimplePath, span: &Span, number: usize,
                         cx: &mut ExtCtxt) -> SimpleField {
    let loc_segment = ident_to_segment(&token::str_to_ident("Location"));
    let (_, terminal) = path.split_terminal();
    let base_path = SimplePath(ast::Path {
        global: false,
        segments: vec![loc_segment, terminal.unwrap()],
        span: span.clone()
    });

    let location_path = base_path.clone_with_concat_terminal(number);
    SimpleField(ast::Field {
        ident: span_item(token::str_to_ident("location"), span.clone()),
        expr: quote_expr!(cx, $location_path),
        span: span.clone()
    })
}

fn parse_nodes(parser: &mut parser::Parser, cx: &mut ExtCtxt) -> Vec<Node> {
    let resource = parse_resource(parser, cx);
    parser.expect(&token::Colon);
    span_note!(cx, resource.span, "Resource: {}", resource);

    let path = parser.parse_path(parser::PathParsingMode::NoTypesAllowed);
    span_note!(cx, parser.last_span, "Path: {}", SimplePath(path.clone()));

    let fields = if parser.eat(&token::OpenDelim(token::DelimToken::Brace)) {
        let parsed_fields = parse_fields(parser);
        span_note!(cx, parser.last_span, "Fields: {:?}", parsed_fields);
        parser.expect(&token::CloseDelim(token::DelimToken::Brace));
        Some(parsed_fields)
    } else {
        parser.expect(&token::Semi);
        None
    };

    let single_resources = resource.to_singles();
    single_resources.into_iter().map(|resource| {
        let simple_path = SimplePath(path.clone());

        // TODO: do this when the platform driver has only the location param
        // that, it has no fields when parsed (so, create array here, etc.)
        // issue: where to get span from?
        let mut fields = fields.clone();
        if fields.is_some() {
            if let ResourceLocation::Single(n) = resource.location {
                let fields_ref = fields.as_mut().unwrap();
                let field_span = fields_ref[0].ident.span;
                let field = mk_location_field(&simple_path, &field_span, n, cx);
                fields_ref.push(field);
            }
        }

        Node {
            name: token::str_to_ident(&resource.to_string()),
            path: simple_path,
            resources: vec![],
            fields: fields
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
        // TODO: Error is fields is none.
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
            statements.push(statement_from_node(node, cx));
        }
    }

    let decl = quote_expr!(cx, {
        use $driver_path_id;
        $statements
    });

    MacExpr::new(decl)
}
