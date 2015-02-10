use syntax::parse::{token};
use syntax::ast::{self, TokenTree, Ident};
use syntax::ext::base::{ExtCtxt};
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};
use syntax::fold::Folder;

#[derive(Debug, Clone)]
pub struct SimplePath(pub ast::Path);

impl SimplePath {
    fn split_terminal(&self) -> (SimplePath, Option<ast::PathSegment>) {
        let mut segments = self.0.segments.clone();
        let terminal = segments.pop();

        let mut new_path = self.0.clone();
        new_path.segments = segments;
        (SimplePath(new_path), terminal)
    }

    pub fn without_terminal(&self) -> SimplePath {
        let (without, _) = self.split_terminal();
        without
    }
}

impl ToString for SimplePath {
    fn to_string(&self) -> String {
        let path = &(self.0);
        let segments = path.segments.iter().map(|ref s| s.identifier.as_str());
        let strings: Vec<&str> = segments.collect();
        strings.connect("::")
    }
}

impl ToTokens for SimplePath {
    fn to_tokens(&self, cx: &ExtCtxt) -> Vec<TokenTree> {
        cx.parse_tts(self.to_string())
    }
}

pub struct PathPrepender {
    base_path_segments: Vec<ast::PathSegment>
}

impl PathPrepender {
    pub fn new(base_path_segments: Vec<ast::PathSegment>) -> PathPrepender {
        PathPrepender {
            base_path_segments: base_path_segments
        }
    }
}

impl Folder for PathPrepender {
    fn fold_path(&mut self, mut p: ast::Path) -> ast::Path {
        let mut segments = self.base_path_segments.clone();
        segments.append(&mut p.segments);

        let mut new_path = p.clone();
        new_path.segments = segments;
        new_path
    }
}

#[macro_export]
macro_rules! parse_int_lit {
    ($parser:expr, $cx:expr, $sp:expr) => (
        match $parser.parse_lit().node {
            Lit_::LitInt(n, _) => n,
            _ => {
                ($cx).span_err(($parser).last_span,
                    "Expected an integer literal.");
                0
            }
        }
    );
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
    items.iter().flat_map(|t| {
        let mut v = t.to_tokens(cx);
        let span = v[v.len() - 1].get_span();
        v.push(ast::TokenTree::TtToken(span.clone(), token.clone()));
        v.into_iter()
    }).collect()
}
