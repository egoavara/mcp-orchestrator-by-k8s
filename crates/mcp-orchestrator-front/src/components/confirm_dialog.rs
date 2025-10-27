use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ConfirmDialogProps {
    pub title: String,
    pub message: String,
    pub on_confirm: Callback<()>,
    pub on_cancel: Callback<()>,
    pub show: bool,
}

#[function_component(ConfirmDialog)]
pub fn confirm_dialog(props: &ConfirmDialogProps) -> Html {
    if !props.show {
        return html! {};
    }

    let on_confirm = {
        let callback = props.on_confirm.clone();
        Callback::from(move |_| callback.emit(()))
    };

    let on_cancel = {
        let callback = props.on_cancel.clone();
        Callback::from(move |_| callback.emit(()))
    };

    html! {
        <div class="modal-overlay">
            <div class="modal">
                <h3>{&props.title}</h3>
                <p>{&props.message}</p>
                <div class="modal-actions">
                    <button class="btn-secondary" onclick={on_cancel}>{"Cancel"}</button>
                    <button class="btn-danger" onclick={on_confirm}>{"Confirm"}</button>
                </div>
            </div>
        </div>
    }
}
