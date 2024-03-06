use proc_macro2::TokenStream;
use proc_macro_error::abort_call_site;
use quote::quote;
use syn::DeriveInput;

pub fn into_inner(ast: DeriveInput) -> TokenStream {
    let input_name = &ast.ident;

    let data = &ast.data;

    let mut variants = vec![];

    match data {
        syn::Data::Enum(ref e) => {
            for v in &e.variants {
                variants.push(v.ident.clone());
            }
        }
        _ => abort_call_site!("Can only be derived for enums"),
    };

    quote! {
        impl #input_name {
            pub fn run(self) -> anyhow::Result<()> {
                match self {
                    #(Self::#variants (a) => a.run(),)*
                }
            }
        }
    }
}
