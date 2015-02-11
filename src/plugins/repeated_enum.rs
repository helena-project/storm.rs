use syntax::ptr;
use syntax::codemap::Span;
use syntax::parse::token;
use syntax::ast::{TokenTree, Lit_, Item_, Variant, Ident, Visibility};
use syntax::ext::base::{ExtCtxt, MacResult, MacItems};
use syntax::ext::build::AstBuilder;
use plugin_utils::*;

type VariantVec = Vec<ptr::P<Variant>>;

fn gen_variants(cx: &ExtCtxt, sp: Span, variants: &mut VariantVec,
                ident: Ident, count: u64) {
    for i in 0..(count) {
        let new_ident = concat_ident(&ident, i);
        let mut variant = cx.variant(sp, new_ident, vec![]);
        variant.node.vis = Visibility::Inherited;
        variants.push(ptr::P(variant));
    }
}

pub fn expand(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree])
        -> Box<MacResult + 'static> {
    let mut parser = cx.new_parser_from_tts(args);

    let is_pub = parser.eat_keyword(token::keywords::Keyword::Pub);
    parser.expect_keyword(token::keywords::Keyword::Enum);

    let enum_name = parser.parse_ident();
    parser.expect(&token::OpenDelim(token::DelimToken::Brace));

    // Create the repeated enum values for each pair of ident + count.
    let mut variants = vec![];
    loop {
        let ident = parser.parse_ident();
        parser.expect(&token::BinOp(token::BinOpToken::Star));
        let number = parse_int_lit(&mut parser, cx);

        // Adds all of the requested variants to the 'variants' array.
        gen_variants(cx, sp, &mut variants, ident, number);

        // If there are no more pairs, we're done.
        if !parser.eat(&token::Comma)
            || parser.check(&token::CloseDelim(token::DelimToken::Brace)) {
                break;
        }
    }

    parser.expect(&token::CloseDelim(token::DelimToken::Brace));

    // Create the enum item. Set the enum's variants and visiblity.
    let enum_id = quote_item!(cx, enum $enum_name { }).unwrap().map(move |mut f| {
        f.vis = if is_pub { Visibility::Public } else { Visibility::Inherited };
        if let Item_::ItemEnum(ref mut def, _) = f.node {
            def.variants = variants;
        };

        f
    });

    let copy_derive = quote_item!(cx, impl Copy for $enum_name {}).unwrap();
    MacItems::new(vec![enum_id, copy_derive].into_iter())
}
