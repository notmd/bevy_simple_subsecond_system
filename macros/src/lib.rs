use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, Pat, PatIdent, parse_macro_input};

#[proc_macro_attribute]
pub fn hot(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let original_output = &sig.output;
    let original_fn_name = &sig.ident;
    let block = &input_fn.block;
    let inputs = &sig.inputs;
    let generics = &sig.generics;
    let where_clause = &sig.generics.where_clause;

    // Generate new identifiers
    let hotpatched_fn = format_ident!("{}_hotpatched", original_fn_name);
    let original_wrapper_fn = format_ident!("{}_original", original_fn_name);

    // Capture parameter types and names
    let mut param_types = Vec::new();
    let mut param_idents = Vec::new();

    for input in inputs {
        match input {
            FnArg::Typed(pat_type) => {
                param_types.push(&pat_type.ty);
                if let Pat::Ident(PatIdent { ident, .. }) = &*pat_type.pat {
                    param_idents.push(ident);
                } else {
                    panic!("`#[hot]` only supports simple identifiers in parameter patterns.");
                }
            }
            FnArg::Receiver(_) => {
                panic!("`#[hot]` does not support `self` methods.");
            }
        }
    }

    // Generate code
    let result = quote! {
        // Outer entry point: stable ABI, hot-reload safe
        #vis fn #original_fn_name(world: &mut bevy::ecs::world::World) #original_output {
            bevy_simple_subsecond_system::dioxus_devtools::subsecond::HotFn::current(#hotpatched_fn)
                .call((world,))
        }

        // Hotpatched version with stable signature
        #vis fn #hotpatched_fn(world: &mut bevy::ecs::world::World) #original_output {
            use bevy::ecs::system::SystemState;
            let mut __system_state: SystemState<(#(#param_types),*)> = SystemState::new(world);
            let (#(#param_idents),*) = __system_state.get(world);
            let _result = #original_wrapper_fn(#(#param_idents),*);
            __system_state.apply(world);
            _result
        }

        // Original function body moved into a standalone fn
        #vis fn #original_wrapper_fn #generics(#inputs) #where_clause {
            #block
        }
    };

    result.into()
}
