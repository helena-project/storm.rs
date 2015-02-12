use syntax::parse::{token, parser};
use syntax::ast::{self, TokenTree, Lit_, Ident};
use syntax::ext::base::{ExtCtxt};
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
use syntax::fold::Folder;
use std::ops::Deref;
use std::fmt::{Display, Formatter, Error};

#[derive(Debug, Clone)]
pub struct SimplePath(pub ast::Path);

impl SimplePath {
    pub fn split_terminal(&self) -> (SimplePath, Option<ast::PathSegment>) {
        let mut new_path = self.clone();
        let terminal = new_path.0.segments.pop();
        (new_path, terminal)
    }

    pub fn without_terminal(&self) -> SimplePath {
        let (without, _) = self.split_terminal();
        without
    }

    pub fn clone_with_concat_terminal<T: ToString>(&self, t: T) -> SimplePath {
        let (mut base_path, terminal) = self.split_terminal();
        if let Some(term) = terminal {
            let new_terminal = concat_ident(&term.identifier, t);
            base_path.0.segments.push(ident_to_segment(&new_terminal));
        }

        base_path
    }
}

// Also implements ToString since impl<T: Display + ?Sized> ToString for T
impl Display for SimplePath {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let path = &(self.0);
        let segments = path.segments.iter().map(|ref s| s.identifier.as_str());
        let strings: Vec<&str> = segments.collect();

        let final_string: String = strings.connect("::");
        final_string.fmt(f)
    }
}

impl ToTokens for SimplePath {
    fn to_tokens(&self, cx: &ExtCtxt) -> Vec<TokenTree> {
        cx.parse_tts(self.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct SimpleField(pub ast::Field);

impl Deref for SimpleField {
    type Target = ast::Field;

    fn deref<'a>(&'a self) -> &'a ast::Field {
        &self.0
    }
}

impl ToTokens for SimpleField {
    fn to_tokens(&self, cx: &ExtCtxt) -> Vec<TokenTree> {
        let mut token_tree = vec![];
        token_tree.push_all(&self.ident.to_tokens(cx));

        let span = token_tree[token_tree.len() - 1].get_span();
        token_tree.push(ast::TokenTree::TtToken(span.clone(), token::Colon));

        token_tree.push_all(&self.expr.to_tokens(cx));
        token_tree
    }
}

pub struct PathPrepender<'a> {
    base_path_segments: &'a Vec<ast::PathSegment>
}

impl<'a> PathPrepender<'a> {
    pub fn new(base_path_segments: &'a Vec<ast::PathSegment>) -> PathPrepender {
        PathPrepender {
            base_path_segments: base_path_segments
        }
    }
}

impl<'a> Folder for PathPrepender<'a> {
    fn fold_path(&mut self, mut p: ast::Path) -> ast::Path {
        let mut segments = self.base_path_segments.clone();
        segments.append(&mut p.segments);

        let mut new_path = p.clone();
        new_path.segments = segments;
        new_path
    }
}

pub fn parse_int_lit(parser: &mut parser::Parser, cx: &mut ExtCtxt) -> u64 {
    match parser.parse_lit().node {
        Lit_::LitInt(n, _) => n,
        _ => {
            cx.span_err(parser.last_span, "Expected an integer literal.");
            0
        }
    }
}

pub fn concat_ident<T: ToString>(ident: &Ident, other: T) -> Ident {
    let mut new_ident = String::from_str(ident.as_str());
    new_ident.push_str(other.to_string().as_slice());
    token::str_to_ident(new_ident.as_slice())
}

pub fn ident_to_segment(ident: &Ident) -> ast::PathSegment {
    ast::PathSegment {
        identifier: ident.clone(),
        parameters: ast::PathParameters::none()
    }
}

pub fn connect_tokens<T: ToTokens>(items: &Vec<T>, token: token::Token,
        cx: &ExtCtxt) -> Vec<TokenTree> {
    let mut tokens: Vec<TokenTree> = items.iter().flat_map(|t| {
        let mut v = t.to_tokens(cx);
        let span = v[v.len() - 1].get_span();
        v.push(ast::TokenTree::TtToken(span.clone(), token.clone()));
        v.into_iter()
    }).collect();

    tokens.pop();
    tokens
}
