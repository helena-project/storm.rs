use syntax::parse::{token};
use syntax::ast::{self, TokenTree, Ident};
use syntax::ext::base::{ExtCtxt};
use syntax::ext::quote::rt::{ToTokens, ExtParseUtils};

#[derive(Debug)]
pub struct SimplePath(pub ast::Path);

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
