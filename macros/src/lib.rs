use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    FnArg, Ident, ItemFn, LitBool, Pat, PatIdent, ReturnType, Token, Type, TypePath, TypeReference,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

struct HotArgs {
    rerun_on_hot_patch: Option<bool>,
}

impl Parse for HotArgs {
    fn parse(input: ParseStream) -> std::result::Result<HotArgs, syn::Error> {
        let mut rerun_on_hot_patch = None;

        while !input.is_empty() {
            let ident: Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            if ident == "rerun_on_hot_patch" {
                let value: LitBool = input.parse()?;
                rerun_on_hot_patch = Some(value.value);
            } else {
                return Err(syn::Error::new_spanned(ident, "Unknown attribute key"));
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(HotArgs { rerun_on_hot_patch })
    }
}

#[proc_macro_attribute]
pub fn hot(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute as a Meta
    let args = parse_macro_input!(attr as HotArgs);
    let rerun_on_hot_patch = args.rerun_on_hot_patch.unwrap_or(false);

    let input_fn = parse_macro_input!(item as ItemFn);
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let original_output = &sig.output;
    let original_fn_name = &sig.ident;
    let block = &input_fn.block;
    let inputs = &sig.inputs;
    let generics = &sig.generics;

    // Generate new identifiers
    let hotpatched_fn = format_ident!("__{}_hotpatched", original_fn_name);
    let original_wrapper_fn = format_ident!("__{}_original", original_fn_name);

    // Capture parameter types, names, and mutability
    let mut param_types = Vec::new();
    let mut param_idents = Vec::new();
    let mut param_mutability = Vec::new();

    for input in inputs {
        match input {
            FnArg::Typed(pat_type) => {
                param_types.push(&pat_type.ty);
                if let Pat::Ident(PatIdent {
                    ident, mutability, ..
                }) = &*pat_type.pat
                {
                    param_idents.push(ident);
                    param_mutability.push(mutability.is_some());
                } else {
                    panic!("`#[hot]` only supports simple identifiers in parameter patterns.");
                }
            }
            FnArg::Receiver(_) => {
                panic!("`#[hot]` does not support `self` methods.");
            }
        }
    }

    // Generate correct destructuring pattern for parameters
    let destructure = param_idents
        .iter()
        .zip(param_mutability.iter())
        .map(|(ident, is_mut)| {
            if *is_mut {
                quote! { mut #ident }
            } else {
                quote! { #ident }
            }
        });
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let maybe_generics = if generics.params.is_empty() {
        quote! {}
    } else {
        quote! { ::#ty_generics }
    };

    let hot_fn = quote! {
        bevy_simple_subsecond_system::dioxus_devtools::subsecond::HotFn::current(#hotpatched_fn #maybe_generics)
    };

    let maybe_run_call = if rerun_on_hot_patch {
        quote! {
            let name = bevy::ecs::system::IntoSystem::into_system(#original_fn_name #maybe_generics).name();
            bevy::prelude::info!("Hot-patched system {name}, executing it now.");
            #hot_fn.call((world,))
        }
    } else {
        quote! {
            let name = bevy::ecs::system::IntoSystem::into_system(#original_fn_name #maybe_generics).name();
            bevy::prelude::info!("Hot-patched system {name}");
        }
    };

    let early_return = if is_result_unit(original_output) {
        quote! {
            return Ok(());
        }
    } else {
        quote! {
            return;
        }
    };

    let hotpatched_fn_definition = match has_single_world_param(sig) {
        WorldParam::Mut | WorldParam::Ref => quote! {
            #vis fn #hotpatched_fn #impl_generics(world: &mut bevy::ecs::world::World) #where_clause #original_output {
                #original_wrapper_fn #maybe_generics(world)
            }
        },
        WorldParam::None => quote! {
            #vis fn #hotpatched_fn #impl_generics(world: &mut bevy::ecs::world::World) #where_clause #original_output {
                use bevy::ecs::system::SystemState;
                let mut __system_state: SystemState<(#(#param_types),*)> = SystemState::new(world);
                let __unsafe_world = world.as_unsafe_world_cell_readonly();

                let __validation = unsafe { SystemState::validate_param(&__system_state, __unsafe_world) };

                match __validation {
                    Ok(()) => (),
                    Err(e) => {
                        if e.skipped {
                            #early_return
                        }
                    }
                }

                let (#(#destructure),*) = __system_state.get_mut(world);
                let __result = #original_wrapper_fn(#(#param_idents),*);
                __system_state.apply(world);
                #[allow(clippy::unused_unit)]
                __result
            }
        },
    };

    let result = quote! {
        // Outer entry point: stable ABI, hot-reload safe
        #vis fn #original_fn_name #impl_generics(world: &mut bevy::ecs::world::World) #where_clause #original_output {
            use std::any::Any as _;
            let type_id = #hotpatched_fn #maybe_generics.type_id();
            let contains_system = world.get_resource::<bevy_simple_subsecond_system::__macros_internal::__HotPatchedSystems>().unwrap().0.contains_key(&type_id);
            if !contains_system {
                let hot_fn_ptr = #hot_fn.ptr_address();
                let system_ptr_update_id = world.register_system(move |world: &mut bevy::ecs::world::World| {
                    let needs_update = {
                        let mut hot_patched_systems = world.get_resource_mut::<bevy_simple_subsecond_system::__macros_internal::__HotPatchedSystems>().unwrap();
                        let mut hot_patched_system = hot_patched_systems.0.get_mut(&type_id).unwrap();
                        hot_patched_system.current_ptr = #hot_fn.ptr_address();
                        let needs_update = hot_patched_system.current_ptr != hot_patched_system.last_ptr;
                        hot_patched_system.last_ptr = hot_patched_system.current_ptr;
                        needs_update
                    };
                    if !needs_update {
                        return;
                    }
                    // TODO: we simply ignore the `Result` here, but we should be propagating it
                    #maybe_run_call;
                });
                let system = bevy_simple_subsecond_system::__macros_internal::__HotPatchedSystem {
                    system_ptr_update_id,
                    current_ptr: hot_fn_ptr,
                    last_ptr: hot_fn_ptr,
                };
                world.get_resource_mut::<bevy_simple_subsecond_system::__macros_internal::__HotPatchedSystems>().unwrap().0.insert(type_id, system);
            }

            #hot_fn.call((world,))
        }

        // Hotpatched version with stable signature
        #hotpatched_fn_definition

        // Original function body moved into a standalone fn
        #vis fn #original_wrapper_fn #impl_generics(#inputs) #where_clause #original_output {
            #block
        }
    };

    result.into()
}

enum WorldParam {
    Ref,
    Mut,
    None,
}

fn has_single_world_param(sig: &syn::Signature) -> WorldParam {
    if sig.inputs.len() != 1 {
        return WorldParam::None;
    }

    let param = sig.inputs.first().unwrap();

    let pat_type = match param {
        FnArg::Typed(pt) => pt,
        _ => return WorldParam::None,
    };

    match &*pat_type.ty {
        Type::Reference(TypeReference {
            mutability, elem, ..
        }) => {
            match &**elem {
                Type::Path(type_path) => {
                    let segments = &type_path.path.segments;

                    let Some(last_segment) = segments.last().cloned() else {
                        return WorldParam::None;
                    };

                    // TODO: Make this more robust :D
                    if last_segment.ident != "World" {
                        return WorldParam::None;
                    }

                    if mutability.is_some() {
                        WorldParam::Mut
                    } else {
                        WorldParam::Ref
                    }
                }
                _ => WorldParam::None,
            }
        }
        _ => WorldParam::None,
    }
}

fn is_result_unit(output: &ReturnType) -> bool {
    match output {
        ReturnType::Default => false, // no return type, i.e., returns ()
        ReturnType::Type(_, ty) => match &**ty {
            Type::Path(TypePath { path, .. }) => {
                // Match on the outer type
                let Some(seg) = path.segments.last() else {
                    return false;
                };
                if seg.ident != "Result" {
                    return false;
                }

                // Match on the generic args: Result<(), BevyError>
                match seg.arguments {
                    syn::PathArguments::AngleBracketed(ref generics) => {
                        let args = &generics.args;

                        let Some(first) = args.first() else {
                            // Not sure this case can even happen
                            return true;
                        };

                        // Check first generic arg is ()
                        matches!(
                            first,
                            syn::GenericArgument::Type(Type::Tuple(t)) if t.elems.is_empty()
                        )
                    }
                    syn::PathArguments::Parenthesized(_) => false,
                    // TODO: This could also be a result that has a non-unit Ok variant
                    syn::PathArguments::None => true,
                }
            }
            _ => false,
        },
    }
}
