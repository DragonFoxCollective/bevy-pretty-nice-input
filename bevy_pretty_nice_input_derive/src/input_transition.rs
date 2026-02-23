use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Token, parse_quote};

use crate::input::{Bindings, Conditions};

pub fn input_transition_impl(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as InputTransition);
    match input.transition {
        Transition::Uni {
            action,
            from,
            to,
            arrow,
        } => {
            if !to.exclusions.is_empty() {
                return syn::Error::new_spanned(
                    to.exclusions[0].to_token_stream(),
                    "Cannot transition into a not (`!`) predicate",
                )
                .to_compile_error()
                .into();
            }

            let half = InputTransitionHalf {
                action,
                from,
                to: to.inclusions,
                arrow,
                bindings: input.bindings,
                conditions: input.conditions,
            };

            match input_transition(half) {
                Ok(expr) => expr.into_token_stream().into(),
                Err(err) => err.to_compile_error().into(),
            }
        }
        Transition::Bi {
            left_action,
            left,
            right_action,
            right,
        } => {
            if !input.conditions.conditions.is_empty() {
                return syn::Error::new_spanned(
                    input.conditions.to_token_stream(),
                    "Conditions are not supported for bidirectional transitions (`<=>`)",
                )
                .to_compile_error()
                .into();
            }

            let left_half = InputTransitionHalf {
                action: left_action,
                from: right.clone(),
                to: left.inclusions.clone(),
                arrow: ObserverArrow::Left,
                bindings: input.bindings.clone(),
                conditions: input.conditions.clone(),
            };
            let left_expr = match input_transition(left_half) {
                Ok(expr) => expr,
                Err(err) => return err.to_compile_error().into(),
            };

            let right_half = InputTransitionHalf {
                action: right_action,
                from: left,
                to: right.inclusions,
                arrow: ObserverArrow::Right,
                bindings: input.bindings,
                conditions: input.conditions,
            };
            let right_expr = match input_transition(right_half) {
                Ok(expr) => expr,
                Err(err) => return err.to_compile_error().into(),
            };

            quote! {
                (
                    #right_expr,
                    #left_expr
                )
            }
            .into_token_stream()
            .into()
        }
    }
}

fn input_transition(mut input: InputTransitionHalf) -> syn::Result<syn::Expr> {
    input
        .conditions
        .conditions
        .insert(0, build_filter(&input.from.query_filter()));

    let observers = build_observers(
        input.action.action(),
        &input.remove_bundle(),
        &input.insert_bundle(),
        &input.arrow,
    )?;

    Ok(build_output(
        &input.action,
        &input.bindings,
        &input.conditions,
        &observers,
    ))
}

fn build_output(
    action: &TransitionFromAction,
    bindings: &Bindings,
    conditions: &Conditions,
    observers: &[syn::Expr],
) -> syn::Expr {
    let inner: syn::Expr = parse_quote! {
        (
            ::bevy_pretty_nice_input::prelude::input!(
                #action,
                #bindings,
                #conditions,
            ),
            #( #observers ),*
        )
    };
    match action {
        TransitionFromAction::Specified(_) => inner,
        TransitionFromAction::Generated(_) => {
            parse_quote! {
                {
                    #[derive(::bevy_pretty_nice_input::prelude::Action)]
                    struct #action;

                    #inner
                }
            }
        }
    }
}

fn build_filter(from: &syn::Type) -> syn::Expr {
    parse_quote! {
        ::bevy_pretty_nice_input::prelude::InvalidatingFilter::< #from >::default()
    }
}

fn build_observers(
    action: &syn::Type,
    remove: &syn::Type,
    insert: &syn::Type,
    arrow: &ObserverArrow,
) -> syn::Result<Vec<syn::Expr>> {
    let transition: syn::Expr = match arrow {
        ObserverArrow::Left => parse_quote! { ::bevy_pretty_nice_input::derive::transition_off },
        ObserverArrow::Right => parse_quote! { ::bevy_pretty_nice_input::derive::transition_on },
    };

    Ok(vec![parse_quote! {
        ::bevy_pretty_nice_input::bundles::observe(#transition::<#action, #remove, #insert>)
    }])
}

#[derive(Clone)]
enum ObserverArrow {
    Left,
    Right,
}

#[derive(Clone)]
enum TransitionFromAction {
    Specified(syn::Type),
    Generated(syn::Type),
}

impl TransitionFromAction {
    fn action(&self) -> &syn::Type {
        match self {
            TransitionFromAction::Specified(t) => t,
            TransitionFromAction::Generated(t) => t,
        }
    }
}

impl ToTokens for TransitionFromAction {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.action().to_tokens(tokens);
    }
}

struct InputTransition {
    transition: Transition,
    bindings: Bindings,
    conditions: Conditions,
}

struct InputTransitionHalf {
    action: TransitionFromAction,
    from: TransitionFrom,
    to: Vec<syn::Type>,
    arrow: ObserverArrow,
    bindings: Bindings,
    conditions: Conditions,
}

impl InputTransitionHalf {
    fn remove_bundle(&self) -> syn::Type {
        let mut remove = self.from.inclusions.iter().cloned().collect::<HashSet<_>>();
        for inc in &self.to {
            remove.remove(inc);
        }
        let mut remove = remove.into_iter().collect::<Vec<_>>();
        remove.sort_by_key(|t| t.to_token_stream().to_string());
        parse_quote! { ( #( #remove ,)* ) }
    }

    fn insert_bundle(&self) -> syn::Type {
        let mut insert = self.to.iter().cloned().collect::<HashSet<_>>();
        for inc in &self.from.inclusions {
            insert.remove(inc);
        }
        let mut insert = insert.into_iter().collect::<Vec<_>>();
        insert.sort_by_key(|t| t.to_token_stream().to_string());
        parse_quote! { ( #( #insert ,)* ) }
    }
}

impl Parse for InputTransition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let transition = input.parse::<Transition>()?;
        input.parse::<Token![,]>()?;
        let bindings = input.parse::<Bindings>()?;
        let conditions = if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
            let conditions = input.parse::<Conditions>().unwrap_or_default();
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
            conditions
        } else {
            Conditions::default()
        };

        Ok(InputTransition {
            transition,
            bindings,
            conditions,
        })
    }
}

#[derive(Clone)]
struct TransitionFrom {
    action: Option<syn::Type>,
    inclusions: Vec<syn::Type>,
    exclusions: Vec<syn::Type>,
}

impl TransitionFrom {
    fn query_filter(&self) -> syn::Type {
        let inclusions = &self.inclusions;
        let exclusions = &self.exclusions;
        parse_quote! { ( #( ::bevy::prelude::With<#inclusions> ,)* #( ::bevy::prelude::Without<#exclusions> ,)* ) }
    }

    fn action(
        &self,
        left: &TransitionFrom,
        arrow: &ObserverArrow,
        right: &TransitionFrom,
    ) -> syn::Result<TransitionFromAction> {
        if let Some(action) = &self.action {
            Ok(TransitionFromAction::Specified(action.clone()))
        } else {
            let generated = generated_action(left, arrow, right)?;
            Ok(TransitionFromAction::Generated(generated))
        }
    }
}

fn generated_action(
    left: &TransitionFrom,
    arrow: &ObserverArrow,
    right: &TransitionFrom,
) -> syn::Result<syn::Type> {
    fn sanitize(s: &str) -> String {
        s.chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect()
    }

    fn type_to_ident_part(ty: &syn::Type) -> String {
        sanitize(&ty.to_token_stream().to_string())
    }

    let mut left_parts = vec![];
    for inc in left.inclusions.iter() {
        left_parts.push(type_to_ident_part(inc));
    }
    for exc in left.exclusions.iter() {
        left_parts.push(format!("Not{}", type_to_ident_part(exc)));
    }
    if left_parts.is_empty() {
        left_parts.push("None".to_string());
    }
    let left = left_parts.join("_");

    let mut right_parts = vec![];
    for inc in right.inclusions.iter() {
        right_parts.push(type_to_ident_part(inc));
    }
    for exc in right.exclusions.iter() {
        right_parts.push(format!("Not{}", type_to_ident_part(exc)));
    }
    if right_parts.is_empty() {
        right_parts.push("None".to_string());
    }
    let right = right_parts.join("_");

    let arrow = match arrow {
        ObserverArrow::Left => "From",
        ObserverArrow::Right => "To",
    };

    syn::parse_str(&format!("Transition_{}_{}_{}", left, arrow, right))
}

impl Parse for TransitionFrom {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let action = if input.peek(syn::token::Paren) {
            None
        } else {
            Some(input.parse::<syn::Type>()?)
        };

        let content;
        syn::parenthesized!(content in input);

        let types = content.parse_terminated(TransitionType::parse, Token![,])?;
        let mut inclusions = vec![];
        let mut exclusions = vec![];
        for ty in types {
            match ty {
                TransitionType::Inclusion(t) => inclusions.push(t),
                TransitionType::Exclusion(t) => exclusions.push(t),
            }
        }
        inclusions.sort_by_key(|t| t.to_token_stream().to_string());
        exclusions.sort_by_key(|t| t.to_token_stream().to_string());

        Ok(TransitionFrom {
            action,
            inclusions,
            exclusions,
        })
    }
}

impl ToTokens for TransitionFrom {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let inclusions = &self.inclusions;
        let exclusions = &self.exclusions;
        tokens.extend(quote! {
            ( #( #inclusions ,)* #( ! #exclusions ,)* )
        });
    }
}

#[derive(Clone)]
enum TransitionType {
    Inclusion(syn::Type),
    Exclusion(syn::Type),
}

impl Parse for TransitionType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![!]) {
            input.parse::<Token![!]>()?;
            let ty = input.parse::<syn::Type>()?;
            Ok(TransitionType::Exclusion(ty))
        } else {
            let ty = input.parse::<syn::Type>()?;
            Ok(TransitionType::Inclusion(ty))
        }
    }
}

impl ToTokens for TransitionType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            TransitionType::Inclusion(ty) => {
                ty.to_tokens(tokens);
            }
            TransitionType::Exclusion(ty) => {
                tokens.extend(quote! { ! #ty });
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
enum TransitionArrow {
    Left,
    Right,
    Both,
}

impl Parse for TransitionArrow {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Token![<]) && input.peek2(Token![=]) && input.peek3(Token![>]) {
            input.parse::<Token![<]>()?;
            input.parse::<Token![=]>()?;
            input.parse::<Token![>]>()?;
            Ok(TransitionArrow::Both)
        } else if input.peek(Token![<]) && input.peek2(Token![=]) {
            input.parse::<Token![<]>()?;
            input.parse::<Token![=]>()?;
            Ok(TransitionArrow::Left)
        } else if input.peek(Token![=]) && input.peek2(Token![>]) {
            input.parse::<Token![=]>()?;
            input.parse::<Token![>]>()?;
            Ok(TransitionArrow::Right)
        } else {
            Err(input.error("Expected one of `<=`, `=>`, or `<=>`"))
        }
    }
}

impl ToTokens for TransitionArrow {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            TransitionArrow::Left => {
                tokens.extend(quote! { <= });
            }
            TransitionArrow::Both => {
                tokens.extend(quote! { <=> });
            }
            TransitionArrow::Right => {
                tokens.extend(quote! { => });
            }
        }
    }
}

#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
enum Transition {
    Uni {
        action: TransitionFromAction,
        from: TransitionFrom,
        to: TransitionFrom,
        arrow: ObserverArrow,
    },
    Bi {
        left_action: TransitionFromAction,
        left: TransitionFrom,
        right_action: TransitionFromAction,
        right: TransitionFrom,
    },
}

impl Parse for Transition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let left = input.parse::<TransitionFrom>()?;
        let arrow = input.parse::<TransitionArrow>()?;
        let right = input.parse::<TransitionFrom>()?;

        match arrow {
            TransitionArrow::Left => Ok(Transition::Uni {
                action: left.action(&left, &ObserverArrow::Left, &right)?,
                from: right,
                to: left,
                arrow: ObserverArrow::Left,
            }),
            TransitionArrow::Right => Ok(Transition::Uni {
                action: right.action(&left, &ObserverArrow::Right, &right)?,
                from: left,
                to: right,
                arrow: ObserverArrow::Right,
            }),
            TransitionArrow::Both => Ok(Transition::Bi {
                left_action: left.action(&left, &ObserverArrow::Left, &right)?,
                right_action: right.action(&left, &ObserverArrow::Right, &right)?,
                left,
                right,
            }),
        }
    }
}
