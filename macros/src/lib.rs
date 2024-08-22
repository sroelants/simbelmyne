use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{punctuated::Punctuated, visit::Visit, Attribute, Expr, ExprLit, Item, ItemConst, Lit, Meta,Token };

#[proc_macro_attribute]
pub fn tunable(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::Item = syn::parse(item.clone()).unwrap();

    let mut params = Params { params: vec![] };
    params.visit_item(&ast);

    let decls = &params.params.clone()
        .into_iter()
        .map(|item| replace_decl(item))
        .collect::<Vec<_>>();

    let uci_decls = if cfg!(feature = "spsa") {
        params.params
            .iter()
            .filter(|item| is_uci_option(item))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let uci_opts = uci_decls
        .iter()
        .map(|item| to_uci_option(item))
        .collect::<Vec<_>>();

    let uci_option_names = uci_decls
        .iter()
        .map(|item| item.ident.to_string().to_lowercase())
        .collect::<Vec<_>>();

    let uci_atomic_idents = uci_decls
        .iter()
        .map(|item| &item.ident)
        .collect::<Vec<_>>();

    let num_uci_opts = uci_opts.len();

    let Item::Mod(mod_item) = &ast else { 
        panic!("#[tunable] proc macro should be used on a module") 
    };

    let mod_ident = &mod_item.ident;

    let rewritten = quote! {
        pub mod #mod_ident {
            use std::sync::atomic::*;
            use uci::options::OptionType;
            use uci::options::UciOption;

            #(#decls)*

            pub const SPSA_UCI_OPTIONS: [UciOption; #num_uci_opts] = [
                #(#uci_opts),*
            ];

            pub fn set_param(name: &str, value: i32) {
                match name {
                    #(#uci_option_names => #uci_atomic_idents.store(value, Ordering::Relaxed),)*
                    _ => println!("Invalid UCI option: {name}"),
                };
            }
        }
    };

    TokenStream::from(rewritten)
}


fn is_uci_option(item: &ItemConst) -> bool {
    if item.attrs.len() == 0 {
        return false;
    }

    item.attrs[0].path().is_ident("uci")
}

fn to_uci_option(item: &ItemConst) -> impl ToTokens {
    let ident = &item.ident;
    let default = &item.expr;

    let uci_option_string = &ident.to_string().to_ascii_lowercase();
    let (min, max, step) = parse_uci_attr(&item.attrs[0]);

    quote! {
        UciOption {
            name: #uci_option_string,
            option_type: OptionType::Spin { 
                min: #min, 
                max: #max, 
                default: #default,
                step: #step,
            }
        }
    }
}

fn parse_uci_attr(attr: &Attribute) -> (i32, i32, i32) {
    let mut min = 0;
    let mut max = 0;
    let mut step = 0;

    let nested = attr
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .expect("Failed to parse arguments to uci attribute");

    for meta in nested {
        let Meta::NameValue(meta) = meta else { panic!("Invalid param passed to uci attr") };

        if meta.path.is_ident("min") {
            let Expr::Lit(ExprLit { 
                lit: Lit::Int(value)
                , .. 
            }) = &meta.value else { panic!("Value passed to min is not an int literal") };

            min = value.base10_parse().expect("Failed to parse min value");
        }

        if meta.path.is_ident("max") {
            let Expr::Lit(ExprLit { 
                lit: Lit::Int(value)
                , .. 
            }) = &meta.value else { panic!("Value passed to min is not an int literal") };

            max = value.base10_parse().expect("Failed to parse min value");
        }

        if meta.path.is_ident("step") {
            let Expr::Lit(ExprLit { 
                lit: Lit::Int(value)
                , .. 
            }) = &meta.value else { panic!("Value passed to min is not an int literal") };

            step = value.base10_parse().expect("Failed to parse min value");
        }
    }

    (min, max, step)
}

////////////////////////////////////////////////////////////////////////////////
//
// Params
//
// A Params struct holds all the const declarations of the params
//
////////////////////////////////////////////////////////////////////////////////

/// A struct that holds all of the const assignments
struct Params<'ast> {
    params: Vec<&'ast ItemConst>
}

impl<'ast> Visit<'ast> for Params<'ast> {
    fn visit_item_const(&mut self, item: &'ast ItemConst) {
        self.params.push(item);
    }
}

fn replace_decl(item: &ItemConst) -> impl ToTokens {
    let ident = &item.ident;
    let ty = &item.ty;
    let expr = &item.expr;

    // Generate atomic type identifier

    let getter_ident = syn::Ident::new(
        &ident.to_string().to_ascii_lowercase(), 
        ident.span()
    );

    quote! {
        #[cfg(not(feature = "spsa"))]
        const #ident: #ty = #expr;

        #[cfg(not(feature = "spsa"))]
        #[inline(always)]
        pub const fn #getter_ident() -> #ty {
            #ident
        }

        #[cfg(feature = "spsa")]
        const #ident: AtomicI32 = AtomicI32::new(#expr);

        #[cfg(feature = "spsa")]
        #[inline(always)]
        pub fn #getter_ident() -> #ty {
            #ident.load(Ordering::Relaxed) as #ty
        }
    }
}
