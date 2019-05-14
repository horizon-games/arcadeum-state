extern crate proc_macro;
extern crate quote;
extern crate syn;

use quote::ToTokens;

#[proc_macro_attribute]
pub fn asynchronous(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    syn::parse_macro_input!(item as TokenStream).0.into()
}

struct TokenStream(proc_macro2::TokenStream);

impl syn::parse::Parse for TokenStream {
    fn parse(input: syn::parse::ParseStream) -> syn::parse::Result<Self> {
        if let Ok(mut function) = input.parse::<syn::ItemFn>() {
            function.block = Box::new(expand_block(&function.block));
            Ok(TokenStream(function.into_token_stream()))
        } else if let Ok(mut method) = input.parse::<syn::ImplItemMethod>() {
            method.block = expand_block(&method.block);
            Ok(TokenStream(method.into_token_stream()))
        } else if let Ok(mut method) = input.parse::<syn::TraitItemMethod>() {
            if let Some(block) = method.default {
                method.default = Some(expand_block(&block));
                Ok(TokenStream(method.into_token_stream()))
            } else {
                Err(input.error("expected method body"))
            }
        } else {
            Err(input.error("expected function or method"))
        }
    }
}

fn expand_block(block: &syn::Block) -> syn::Block {
    let statements = expand_statements(&block.stmts);
    syn::parse2(quote::quote! { { #(#statements)* } }).unwrap()
}

fn expand_statements(mut input: &[syn::Stmt]) -> Vec<syn::Stmt> {
    let mut output = Vec::new();

    while input.len() > 0 {
        let statement = &input[0];
        input = &input[1..];

        if let syn::Stmt::Local(syn::Local {
            pats: patterns,
            init: Some((_, initializer)),
            ..
        }) = statement
        {
            if patterns.len() == 1 {
                if let syn::Expr::Field(syn::ExprField {
                    base: object,
                    member: syn::Member::Named(field),
                    ..
                }) = &**initializer
                {
                    if field.to_string() == "await" {
                        match &**object {
                            syn::Expr::Call(call) => {
                                let pattern = patterns.first().unwrap();

                                let parameters = match pattern.value() {
                                    syn::Pat::Tuple(syn::PatTuple { front, .. }) => {
                                        front.into_token_stream()
                                    }
                                    _ => pattern.value().into_token_stream(),
                                };

                                let statements = expand_statements(input);

                                let closure = syn::parse2(
                                    quote::quote! { move |#parameters| { #(#statements)* } },
                                )
                                .unwrap();

                                let mut call = call.clone();
                                call.args.push(closure);

                                output.push(syn::Stmt::Semi(
                                    syn::Expr::Call(call),
                                    syn::Token![;](proc_macro2::Span::call_site()),
                                ));

                                return output;
                            }
                            syn::Expr::MethodCall(call) => {
                                let pattern = patterns.first().unwrap();

                                let parameters = match pattern.value() {
                                    syn::Pat::Tuple(syn::PatTuple { front, .. }) => {
                                        front.into_token_stream()
                                    }
                                    _ => pattern.value().into_token_stream(),
                                };

                                let statements = expand_statements(input);

                                let closure = syn::parse2(
                                    quote::quote! { move |#parameters| { #(#statements)* } },
                                )
                                .unwrap();

                                let mut call = call.clone();
                                call.args.push(closure);

                                output.push(syn::Stmt::Semi(
                                    syn::Expr::MethodCall(call),
                                    syn::Token![;](proc_macro2::Span::call_site()),
                                ));

                                return output;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        output.push(statement.clone());
    }

    output
}
