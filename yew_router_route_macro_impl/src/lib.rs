extern crate proc_macro;
use proc_macro::{TokenStream};
use yew_router_route_parser::{PathMatcher, OptimizedToken, CaptureVariants};
use quote::{quote, ToTokens};
use syn::export::TokenStream2;
use proc_macro_hack::proc_macro_hack;
use syn::{Error, Type};
use syn::parse::{Parse, ParseBuffer};
use syn::parse_macro_input;

struct S {
    s: String,
    ty: Type
}
impl Parse for S {
    fn parse(input: &ParseBuffer) -> Result<Self, Error> {
        let s = input.parse::<syn::LitStr>()?;
        let _ = input.parse::<syn::token::FatArrow>()?;
        let ty = input.parse::<syn::Type>()?;
        Ok(
            S {
                s: s.value(),
                ty
            }
        )
    }
}

/// Expected to be used like: route!("/route/to/thing" => Component)
#[proc_macro_hack]
pub fn route(input: TokenStream) -> TokenStream {
    let s = parse_macro_input!(input as S);
    let ty = s.ty;
    let s: String = s.s;

    // Do the parsing at compile time so the user knows if their matcher is malformed.
    // It will still be their responsibility to know that the corresponding Props can be acquired from a path matcher.
//    let pm= PathMatcher::try_from(s.as_str()).expect("Invalid Path Matcher");
    let t = yew_router_route_parser::parse_str_and_optimize_tokens(s.as_str())
        .expect("Invalid Path Matcher")
        .into_iter()
        .map(ShadowOptimizedToken::from);
    let expanded = quote!{
        {
            use yew_router::yew_router_route_parser::PathMatcher as __PathMatcher;
            use yew_router::yew_router_route_parser::CaptureVariants as __CaptureVariants;
            use yew_router::yew_router_route_parser::OptimizedToken as __OptimizedToken;
            __PathMatcher {
                tokens : vec![#(#t),*],
                render_fn : __PathMatcher::create_render_fn(PhantomData<#ty>)
            }
        }
    };
    TokenStream::from(expanded)
}

impl ToTokens for ShadowOptimizedToken {
    fn to_tokens(&self, ts: &mut TokenStream2) {
        use ShadowOptimizedToken as SOT;
        let t: TokenStream2 = match self {
            SOT::Match(s) => {
                TokenStream2::from(quote!{__OptimizedToken::Match(#s.to_string())})
            }
            SOT::Capture ( variant ) => {
                TokenStream2::from(quote!{
                    __OptimizedToken::Capture(#variant)
                })
            }
            SOT::QueryCapture { ident, value } => {
                let ident = ident.clone();
                let value = value.clone();
                TokenStream2::from(quote!{
                    __OptimizedToken::QueryCapture{ident: #ident.to_string(), value: #value.to_string()}
                })
            }
        };
        ts.extend(t)
    }
}

/// A shadow of the OptimizedToken type.
/// It should match it exactly so that this macro can expand to the original.
enum ShadowOptimizedToken {
    Match(String),
    Capture(ShadowCaptureVariant),
    QueryCapture {
        ident: String,
        value: String
    }
}

enum ShadowCaptureVariant {
    Unnamed, // {} - matches anything
    ManyUnnamed, // {*} - matches over multiple sections
    NumberedUnnamed{sections: usize}, // {4} - matches 4 sections
    Named(String), // {name} - captures a section and adds it to the map with a given name
    ManyNamed(String), // {*:name} - captures over many sections and adds it to the map with a given name.
    NumberedNamed{sections: usize, name: String} // {2:name} - captures a fixed number of sections with a given name.
}

impl ToTokens for ShadowCaptureVariant {

    fn to_tokens(&self, ts: &mut TokenStream2) {
        let t = match self {
            ShadowCaptureVariant::Unnamed => TokenStream2::from(quote!{__CaptureVariants::Unnamed}),
            ShadowCaptureVariant::ManyUnnamed => TokenStream2::from(quote!{__CaptureVariants::ManyUnnamed}),
            ShadowCaptureVariant::NumberedUnnamed { sections } => TokenStream2::from(quote!{__CaptureVariants::NumberedUnnamed{#sections}}),
            ShadowCaptureVariant::Named(name) => TokenStream2::from(quote!{__CaptureVariants::Named(#name.to_string())}),
            ShadowCaptureVariant::ManyNamed(name) => TokenStream2::from(quote!{__CaptureVariants::ManyNamed(#name.to_string())}),
            ShadowCaptureVariant::NumberedNamed { sections, name } => TokenStream2::from(quote!{__CaptureVariants::NumberedNamed{#sections, #name.to_string()}}),
        };
        ts.extend(t)

    }
}

impl From<OptimizedToken> for ShadowOptimizedToken {
    fn from(ot: OptimizedToken) -> Self {
        use OptimizedToken as OT;
        use ShadowOptimizedToken as SOT;
        match ot {
            OT::Match(s) => SOT::Match(s),
            OT::Capture(variant) => SOT::Capture(variant.into()),
            OT::QueryCapture { ident, value } => SOT::QueryCapture {ident, value}
        }
    }
}

impl From<CaptureVariants> for ShadowCaptureVariant {

    fn from(cv: CaptureVariants) -> Self {
        use CaptureVariants as CV;
        use ShadowCaptureVariant as SCV;
        match cv {
            CV::Unnamed => SCV::Unnamed,
            CaptureVariants::ManyUnnamed => SCV::ManyUnnamed,
            CaptureVariants::NumberedUnnamed { sections } => SCV::NumberedUnnamed {sections},
            CaptureVariants::Named(name) => SCV::Named(name),
            CaptureVariants::ManyNamed(name) => SCV::ManyNamed(name),
            CaptureVariants::NumberedNamed { sections, name } => SCV::NumberedNamed {sections, name}
        }

    }
}
