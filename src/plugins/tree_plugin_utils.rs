use syntax::codemap::Span;
use syntax::parse::{token, parser};
use syntax::ast::{TokenTree, Ident, PathSegment};
use syntax::ext::base::{ExtCtxt};
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
use std::ascii::OwnedAsciiExt;
use syntax::fold::Folder;
use std::fmt::{Display, Formatter, Error};
use std::mem;
use plugin_utils::*;

#[derive(Debug)]
pub enum ResourceLocation {
    None,
    Single(usize),
    Range { from: usize, to: usize }
}

#[derive(Debug)]
pub struct Resource {
    pub name: Ident,
    pub span: Span,
    pub location: ResourceLocation
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
    // This clone isn't needed...except that Rust doesn't support non-lexical borrows.
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

pub fn parse_resource(parser: &mut parser::Parser, cx: &mut ExtCtxt) -> Resource {
    let mut resource_span = parser.span.clone();
    let resource_name = parser.parse_ident();
    let resource_location = if parser.eat(&token::At) {
        parse_resource_location(parser, cx)
    } else {
        ResourceLocation::None
    };

    resource_span.hi = parser.span.lo;
    Resource {
        name: resource_name,
        span: resource_span,
        location: resource_location
    }
}

pub fn parse_resources(parser: &mut parser::Parser, cx: &mut ExtCtxt) -> Vec<Resource> {
    let mut resources = vec![];
    loop {
        resources.push(parse_resource(parser, cx));
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

pub fn canonicalize_node_paths(base_segments: &Vec<PathSegment>, node: &mut Node) {
    let mut base_prepender = PathPrepender::new(base_segments);
    node.path = SimplePath(base_prepender.fold_path(node.path.0.clone()));

    if let Some(fields) = node.fields.take() {
        let device_segments = node.path.without_terminal().0.segments;
        let mut prepender = PathPrepender::new(&device_segments);
        let new_fields = fields.map_in_place(|field| {
            SimpleField(prepender.fold_field(field.0))
        });

        node.fields = Some(new_fields);
    }
}

