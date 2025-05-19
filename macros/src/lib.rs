use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, Pat, PatIdent, parse_macro_input};

#[proc_macro_attribute]
pub fn hot(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;
    let fn_name = &sig.ident;

    let hotpatched_name = format_ident!("{}_hotpatched", fn_name);
    let generics = &sig.generics;
    let output = &sig.output;
    let where_clause = &sig.generics.where_clause;

    // Construct two sets of arguments:
    // - `clean_inputs`: used for outer function (without `mut`)
    // - `arg_idents`: the list of argument names for .call((...))
    let mut clean_inputs = Vec::new();
    let mut arg_idents = Vec::new();

    for input in &sig.inputs {
        match input {
            syn::FnArg::Typed(pat_type) => {
                // Assume pattern is an identifier (no destructuring)
                if let Pat::Ident(PatIdent { ident, .. }) = &*pat_type.pat {
                    arg_idents.push(quote! { #ident });

                    let ty = &pat_type.ty;
                    let attrs = &pat_type.attrs;

                    // Rebuild argument without `mut`
                    clean_inputs.push(quote! {
                        #(#attrs)* #ident: #ty
                    });
                } else {
                    panic!("`#[hot]` only supports identifier patterns (no destructuring)");
                }
            }
            syn::FnArg::Receiver(_) => {
                panic!("`#[hot]` does not support methods (`self` parameter)");
            }
        }
    }

    // Rebuild the outer signature (stripped of `mut`)
    let outer_fn = quote! {
        #vis fn #fn_name #generics(#(#clean_inputs),*) #output #where_clause {
            bevy_simple_subsecond_system::dioxus_devtools::subsecond::HotFn::current(#hotpatched_name).call((#(#arg_idents),*));
        }
    };

    // The original hotpatched function
    let inputs = &sig.inputs;

    let hotpatched_fn = quote! {
        #vis fn #hotpatched_name #generics(#inputs) #output #where_clause
            #block
    };

    quote! {
        #outer_fn
        #hotpatched_fn
    }
    .into()
}
