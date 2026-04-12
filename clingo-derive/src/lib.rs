use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, parse_macro_input};

/// Convert a PascalCase identifier to snake_case.
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }
    result
}

#[proc_macro_derive(Symbolic)]
pub fn derive_symbolic(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = match &input.data {
        Data::Struct(data) => derive_struct(name, &data.fields),
        Data::Enum(data) => derive_enum(name, data),
        Data::Union(_) => {
            return syn::Error::new_spanned(name, "Symbolic cannot be derived for unions")
                .to_compile_error()
                .into();
        }
    };

    expanded.into()
}

fn derive_struct(name: &syn::Ident, fields: &Fields) -> proc_macro2::TokenStream {
    let func_name = to_snake_case(&name.to_string());

    match fields {
        Fields::Unit => {
            quote! {
                impl Symbolic for #name {
                    fn from_symbol(sym: clingo::Symbol) -> Option<Self> {
                        if sym.symbol_type() != clingo::SymbolType::Function { return None; }
                        if sym.is_positive() != Some(true) { return None; }
                        if sym.name()? != #func_name { return None; }
                        let args = sym.arguments()?;
                        if !args.is_empty() { return None; }
                        Some(#name)
                    }
                    fn to_symbol(&self) -> clingo::Symbol {
                        clingo::Symbol::id(#func_name, true).unwrap()
                    }
                }
            }
        }
        Fields::Unnamed(fields) => {
            let field_count = fields.unnamed.len();
            let field_indices: Vec<syn::Index> = (0..field_count).map(syn::Index::from).collect();
            let field_vars: Vec<syn::Ident> = (0..field_count)
                .map(|i| syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site()))
                .collect();

            quote! {
                impl Symbolic for #name {
                    fn from_symbol(sym: clingo::Symbol) -> Option<Self> {
                        if sym.symbol_type() != clingo::SymbolType::Function { return None; }
                        if sym.is_positive() != Some(true) { return None; }
                        if sym.name()? != #func_name { return None; }
                        let args = sym.arguments()?;
                        if args.len() != #field_count { return None; }
                        Some(#name(
                            #(Symbolic::from_symbol(args[#field_indices])?,)*
                        ))
                    }
                    fn to_symbol(&self) -> clingo::Symbol {
                        let #name(#(#field_vars),*) = self;
                        clingo::Symbol::function(#func_name, &[
                            #(#field_vars.to_symbol(),)*
                        ], true).unwrap()
                    }
                }
            }
        }
        Fields::Named(fields) => {
            let field_count = fields.named.len();
            let field_names: Vec<&syn::Ident> = fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap())
                .collect();
            let field_indices: Vec<syn::Index> = (0..field_count).map(syn::Index::from).collect();

            quote! {
                impl Symbolic for #name {
                    fn from_symbol(sym: clingo::Symbol) -> Option<Self> {
                        if sym.symbol_type() != clingo::SymbolType::Function { return None; }
                        if sym.is_positive() != Some(true) { return None; }
                        if sym.name()? != #func_name { return None; }
                        let args = sym.arguments()?;
                        if args.len() != #field_count { return None; }
                        Some(#name {
                            #(#field_names: Symbolic::from_symbol(args[#field_indices])?,)*
                        })
                    }
                    fn to_symbol(&self) -> clingo::Symbol {
                        clingo::Symbol::function(#func_name, &[
                            #(self.#field_names.to_symbol(),)*
                        ], true).unwrap()
                    }
                }
            }
        }
    }
}

fn derive_enum(name: &syn::Ident, data: &syn::DataEnum) -> proc_macro2::TokenStream {
    let mut from_arms = Vec::new();
    let mut to_arms = Vec::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let func_name = to_snake_case(&variant_name.to_string());

        match &variant.fields {
            Fields::Unit => {
                from_arms.push(quote! {
                    (#func_name, 0) => Some(#name::#variant_name),
                });
                to_arms.push(quote! {
                    #name::#variant_name => clingo::Symbol::id(#func_name, true).unwrap(),
                });
            }
            Fields::Unnamed(fields) => {
                let field_count = fields.unnamed.len();
                let field_indices: Vec<syn::Index> =
                    (0..field_count).map(syn::Index::from).collect();
                let field_vars: Vec<syn::Ident> = (0..field_count)
                    .map(|i| syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site()))
                    .collect();

                from_arms.push(quote! {
                    (#func_name, #field_count) => Some(#name::#variant_name(
                        #(Symbolic::from_symbol(args[#field_indices])?,)*
                    )),
                });
                to_arms.push(quote! {
                    #name::#variant_name(#(#field_vars),*) => {
                        clingo::Symbol::function(#func_name, &[
                            #(#field_vars.to_symbol(),)*
                        ], true).unwrap()
                    }
                });
            }
            Fields::Named(fields) => {
                let field_count = fields.named.len();
                let field_names: Vec<&syn::Ident> = fields
                    .named
                    .iter()
                    .map(|f| f.ident.as_ref().unwrap())
                    .collect();
                let field_indices: Vec<syn::Index> =
                    (0..field_count).map(syn::Index::from).collect();

                from_arms.push(quote! {
                    (#func_name, #field_count) => Some(#name::#variant_name {
                        #(#field_names: Symbolic::from_symbol(args[#field_indices])?,)*
                    }),
                });
                to_arms.push(quote! {
                    #name::#variant_name { #(#field_names),* } => {
                        clingo::Symbol::function(#func_name, &[
                            #(#field_names.to_symbol(),)*
                        ], true).unwrap()
                    }
                });
            }
        }
    }

    quote! {
        impl Symbolic for #name {
            fn from_symbol(sym: clingo::Symbol) -> Option<Self> {
                if sym.symbol_type() != clingo::SymbolType::Function { return None; }
                if sym.is_positive() != Some(true) { return None; }
                let name = sym.name()?;
                let args = sym.arguments()?;
                match (name, args.len()) {
                    #(#from_arms)*
                    _ => None,
                }
            }
            fn to_symbol(&self) -> clingo::Symbol {
                match self {
                    #(#to_arms)*
                }
            }
        }
    }
}
