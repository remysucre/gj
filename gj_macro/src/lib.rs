use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro]
pub fn sql(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_sql(&ast)
}

fn impl_sql(ast: &syn::LitStr) -> TokenStream {
    let gen = quote! {
        println!("Hello, Macro! My name is {}!", stringify!(#ast));
    };
    gen.into()
}
