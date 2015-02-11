use syntax::codemap::Span;
use syntax::parse::{token, parser};
use syntax::ast::{TokenTree, Lit_, Ident, PathSegment};
use syntax::ext::base::{ExtCtxt};
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
use std::ascii::OwnedAsciiExt;
use syntax::fold::Folder;
use std::fmt::{Display, Formatter, Error};
use plugin_utils::*;

#[derive(Debug)]
pub enum ResourceLocation {
    None,
    Single(usize),
    Range { from: usize, to: usize }
}

#[derive(Debug)]
pub struct Resource {
    name: Ident,
    span: Span,
    location: ResourceLocation
}

impl Display for Resource {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self.location {
            ResourceLocation::None => {
                let mut string = String::from_str(self.name.as_str());
                string = string.into_ascii_lowercase();
                string.fmt(f)
            }
            ResourceLocation::Single(ref location) => {
                let mut string = String::from_str(self.name.as_str());
                string = string.into_ascii_lowercase();
                string.push_str("_");
                string.push_str(&location.to_string());
                string.fmt(f)
            },
            ResourceLocation::Range{from, to} => {
                let mut result = Ok(());
                for i in from..to {
                    let simp_res = Resource {
                        name: self.name.clone(),
                        span: self.span.clone(),
                        location: ResourceLocation::Single(i)
                    };

                    result = result.and(simp_res.fmt(f));
                    if i != to - 1 {
                        result = result.and(f.write_str(","));
                    }
                }

                result
            }
        }
    }
}

impl ToTokens for Resource {
    fn to_tokens(&self, cx: &ExtCtxt) -> Vec<TokenTree> {
        cx.parse_tts(self.to_string())
    }
}

#[derive(Debug)]
pub struct Node {
    pub name: Ident,
    pub path: SimplePath,
    pub resources: Vec<Resource>,
    pub fields: Option<Vec<SimpleField>>
}

pub fn parse_resource_location(parser: &mut parser::Parser, cx: &mut ExtCtxt)
        -> ResourceLocation {
    // TODO: This should be possible without the clone.
    let current_token = parser.token.clone();
    match &current_token {
        t@&token::OpenDelim(token::DelimToken::Bracket) => {
            parser.eat(t);
            let from = parse_int_lit(parser, cx) as usize;
            parser.expect(&token::DotDot);
            let to = parse_int_lit(parser, cx) as usize;
            parser.eat(&token::CloseDelim(token::DelimToken::Bracket));

            ResourceLocation::Range { from: from, to: to }
        },
        &token::Literal(..) => {
            ResourceLocation::Single(parse_int_lit(parser, cx) as usize)
        }
        _ => ResourceLocation::None
    }
}

pub fn parse_resources(parser: &mut parser::Parser, cx: &mut ExtCtxt) -> Vec<Resource> {
    let mut resources = vec![];
    loop {
        let mut resource_span = parser.span.clone();
        let resource_name = parser.parse_ident();
        let resource_location = if parser.eat(&token::At) {
            parse_resource_location(parser, cx)
        } else {
            ResourceLocation::None
        };

        resource_span.hi = parser.span.lo;
        resources.push(Resource {
            name: resource_name,
            span: resource_span,
            location: resource_location
        });

        if !parser.eat(&token::Comma) {
            break;
        }
    }

    resources
}

pub fn parse_fields(parser: &mut parser::Parser) -> Vec<SimpleField> {
    let mut fields = vec![];
    loop {
        fields.push(SimpleField(parser.parse_field()));
        if !parser.eat(&token::Comma) {
            break;
        }
    }

    fields
}

pub fn parse_node(parser: &mut parser::Parser, cx: &mut ExtCtxt) -> Node {
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

    Node {
        name: item_name,
        path: SimplePath(path),
        resources: resources,
        fields: fields
    }
}

pub fn canonicalize_node_paths(base_segment: &PathSegment, node: &mut Node) {
    node.path.0.segments.insert(0, base_segment.clone());

    if let Some(fields) = node.fields.take() {
        let device_segments = node.path.without_terminal().0.segments;
        let mut prepender = PathPrepender::new(device_segments);
        let new_fields = fields.map_in_place(|field| {
            SimpleField(prepender.fold_field(field.0))
        });

        node.fields = Some(new_fields);
    }
}

