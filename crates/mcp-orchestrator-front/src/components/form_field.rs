use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FormFieldProps {
    pub label: String,
    pub error: Option<String>,
    pub children: Children,
}

#[function_component(FormField)]
pub fn form_field(props: &FormFieldProps) -> Html {
    html! {
        <div class="form-group">
            <label>{&props.label}</label>
            {for props.children.iter()}
            if let Some(error) = &props.error {
                <span class="field-error">{error}</span>
            }
        </div>
    }
}
