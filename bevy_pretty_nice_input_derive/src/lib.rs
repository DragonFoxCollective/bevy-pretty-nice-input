use proc_macro::TokenStream;

mod derive_action;
mod derive_try_from_action_data;
mod input;
mod input_transition;

#[proc_macro_derive(Action, attributes(action))]
pub fn derive_action(input: TokenStream) -> TokenStream {
    derive_action::derive_action(input)
}

#[proc_macro_derive(TryFromActionData, attributes(action_data))]
pub fn derive_try_from_action_data(input: TokenStream) -> TokenStream {
    derive_try_from_action_data::derive_try_from_action_data(input)
}

#[proc_macro]
pub fn input(input: TokenStream) -> TokenStream {
    input::input_impl(input)
}

#[proc_macro]
pub fn input_transition(input: TokenStream) -> TokenStream {
    input_transition::input_transition_impl(input)
}
