use proc_macro::TokenStream;
use quote::quote;
use syn::{Item, ItemEnum, ItemFn, ItemImpl, ItemStruct, ItemTrait, parse_macro_input};

/// A procedural macro that wraps various Rust items in unsafe blocks.
///
/// This macro can be applied to:
/// - Functions: Wraps the function body in an unsafe block
/// - Structs: Wraps field access and construction in unsafe contexts
/// - Impl blocks: Wraps all method bodies in unsafe blocks
/// - Trait implementations: Wraps trait impl methods in unsafe blocks
/// - Enums: Provides unsafe construction helpers
/// - Traits: Marks trait methods as unsafe
///
/// # Examples
///
/// ```rust
/// use noble::noble;
///
/// #[noble]
/// fn my_function() {
///     println!("This will be wrapped in unsafe");
/// }
///
/// #[noble]
/// struct MyStruct {
///     field: i32,
/// }
///
/// #[noble]
/// impl MyStruct {
///     fn new(value: i32) -> Self {
///         Self { field: value }
///     }
/// }
///
/// #[noble]
/// impl Send for MyStruct {}
/// ```
#[proc_macro_attribute]
pub fn noble(_args: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);

    match item {
        Item::Fn(func) => wrap_function(func),
        Item::Struct(struct_item) => wrap_struct(struct_item),
        Item::Impl(impl_item) => wrap_impl(impl_item),
        Item::Enum(enum_item) => wrap_enum(enum_item),
        Item::Trait(trait_item) => wrap_trait(trait_item),
        _ => {
            // For unsupported items, just return them as-is
            quote! { #item }.into()
        }
    }
}

fn wrap_function(mut func: ItemFn) -> TokenStream {
    let original_block = &func.block;

    // Create a new block that wraps the original in unsafe
    func.block = syn::parse_quote! {
        {
            unsafe #original_block
        }
    };

    quote! { #func }.into()
}

fn wrap_struct(struct_item: ItemStruct) -> TokenStream {
    let name = &struct_item.ident;
    let vis = &struct_item.vis;
    let attrs = &struct_item.attrs;
    let generics = &struct_item.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate the original struct
    let original_struct = quote! {
        #(#attrs)*
        #vis struct #name #generics #struct_item.fields #where_clause
    };

    // Generate unsafe constructor and field access methods
    let constructor = match &struct_item.fields {
        syn::Fields::Named(fields) => {
            let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
            let field_types: Vec<_> = fields.named.iter().map(|f| &f.ty).collect();

            quote! {
                impl #impl_generics #name #ty_generics #where_clause {
                    /// Unsafe constructor
                    pub unsafe fn new_unsafe(#(#field_names: #field_types),*) -> Self {
                        unsafe {
                            Self {
                                #(#field_names),*
                            }
                        }
                    }
                }
            }
        }
        syn::Fields::Unnamed(fields) => {
            let field_types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();
            // let field_indices: Vec<_> = (0..field_types.len()).map(syn::Index::from).collect();
            let param_names: Vec<_> = (0..field_types.len())
                .map(|i| syn::Ident::new(&format!("field_{}", i), proc_macro2::Span::call_site()))
                .collect();

            quote! {
                impl #impl_generics #name #ty_generics #where_clause {
                    /// Unsafe constructor
                    pub unsafe fn new_unsafe(#(#param_names: #field_types),*) -> Self {
                        unsafe {
                            Self(#(#param_names),*)
                        }
                    }
                }
            }
        }
        syn::Fields::Unit => {
            quote! {
                impl #impl_generics #name #ty_generics #where_clause {
                    /// Unsafe constructor
                    pub unsafe fn new_unsafe() -> Self {
                        unsafe { Self }
                    }
                }
            }
        }
    };

    quote! {
        #original_struct
        #constructor
    }
    .into()
}

fn wrap_impl(mut impl_item: ItemImpl) -> TokenStream {
    // Check if this is a trait implementation (impl Trait for Type)
    if impl_item.trait_.is_some() {
        // For trait implementations, mark the impl as unsafe and wrap method bodies
        impl_item.unsafety = Some(syn::token::Unsafe::default());

        // Wrap all method bodies in unsafe blocks
        for item in &mut impl_item.items {
            if let syn::ImplItem::Fn(method) = item {
                let original_block = &method.block;
                method.block = syn::parse_quote! {
                    {
                        unsafe #original_block
                    }
                };
            }
        }
    } else {
        // For regular impl blocks, just wrap method bodies
        for item in &mut impl_item.items {
            if let syn::ImplItem::Fn(method) = item {
                let original_block = &method.block;
                method.block = syn::parse_quote! {
                    {
                        unsafe #original_block
                    }
                };
            }
        }
    }

    quote! { #impl_item }.into()
}

fn wrap_enum(enum_item: ItemEnum) -> TokenStream {
    let name = &enum_item.ident;
    let vis = &enum_item.vis;
    let attrs = &enum_item.attrs;
    let generics = &enum_item.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate the original enum
    let variants = &enum_item.variants;
    let original_enum = quote! {
        #(#attrs)*
        #vis enum #name #generics #where_clause {
            #variants
        }
    };

    // Generate unsafe construction methods for each variant
    let variant_constructors: Vec<_> = enum_item
        .variants
        .iter()
        .map(|variant| {
            let variant_name = &variant.ident;
            let method_name = syn::Ident::new(
                &format!("new_{}_unsafe", variant_name.to_string().to_lowercase()),
                proc_macro2::Span::call_site(),
            );

            match &variant.fields {
                syn::Fields::Named(fields) => {
                    let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                    let field_types: Vec<_> = fields.named.iter().map(|f| &f.ty).collect();
                    quote! {
                        pub unsafe fn #method_name(#(#field_names: #field_types),*) -> Self {
                            Self::#variant_name { #(#field_names),* }
                        }
                    }
                }
                syn::Fields::Unnamed(fields) => {
                    let field_types: Vec<_> = fields.unnamed.iter().map(|f| &f.ty).collect();
                    let param_names: Vec<_> = (0..field_types.len())
                        .map(|i| {
                            syn::Ident::new(&format!("field_{}", i), proc_macro2::Span::call_site())
                        })
                        .collect();
                    quote! {
                        pub unsafe fn #method_name(#(#param_names: #field_types),*) -> Self {
                            Self::#variant_name(#(#param_names),*)
                        }
                    }
                }
                syn::Fields::Unit => {
                    quote! {
                        pub unsafe fn #method_name() -> Self {
                            Self::#variant_name
                        }
                    }
                }
            }
        })
        .collect();

    quote! {
        #original_enum

        impl #impl_generics #name #ty_generics #where_clause {
            #(#variant_constructors)*
        }
    }
    .into()
}

fn wrap_trait(mut trait_item: ItemTrait) -> TokenStream {
    // Mark all trait methods as unsafe
    for item in &mut trait_item.items {
        if let syn::TraitItem::Fn(method) = item {
            // Add unsafe to the method signature
            method.sig.unsafety = Some(syn::token::Unsafe::default());

            // If there's a default implementation, wrap it in unsafe
            if let Some(block) = &method.default {
                let original_block = block;
                method.default = Some(syn::parse_quote! {
                    {
                        unsafe #original_block
                    }
                });
            }
        }
    }

    // Mark the trait itself as unsafe
    trait_item.unsafety = Some(syn::token::Unsafe::default());

    quote! { #trait_item }.into()
}
