use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, parse_macro_input};

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

    // Collect the types for SystemState<(...)>
    let mut param_types = Vec::new();

    // Reconstruct the argument list for the hotpatched function
    let inputs = &sig.inputs;

    for input in inputs {
        match input {
            syn::FnArg::Typed(pat_type) => {
                let ty = &pat_type.ty;
                param_types.push(quote! { #ty });
            }
            syn::FnArg::Receiver(_) => {
                panic!("`#[hot]` does not support methods (`self` parameter)");
            }
        }
    }

    // Outer function taking only `world: &mut World`
    let outer_fn = quote! {
        #vis fn #fn_name #generics(world: &mut bevy::ecs::world::World) #output #where_clause {
            use bevy::ecs::system::SystemState;
            let mut system_state: SystemState<(#(#param_types),*)> = SystemState::new(world);
            let inputs = system_state.get(world);
            bevy_simple_subsecond_system::dioxus_devtools::subsecond::HotFn::current(#hotpatched_name)
                .call(inputs);
            system_state.apply(world);
        }
    };

    // Original logic becomes the hotpatched function
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
