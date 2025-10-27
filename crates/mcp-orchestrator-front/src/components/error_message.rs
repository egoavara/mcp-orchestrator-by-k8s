use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ErrorMessageProps {
    pub message: String,
}

#[function_component(ErrorMessage)]
pub fn error_message(props: &ErrorMessageProps) -> Html {
    html! {
        <div class="error-message">
            <strong>{"Error: "}</strong>
            <span>{&props.message}</span>
        </div>
    }
}
