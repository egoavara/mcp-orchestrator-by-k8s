use crate::models::state::{AuthState, SessionState};
use yew::prelude::*;
use yewdux::prelude::*;

#[function_component(UserMenu)]
pub fn user_menu() -> Html {
    let (auth_state, auth_dispatch) = use_store::<AuthState>();
    let (_, session_dispatch) = use_store::<SessionState>();
    let show_menu = use_state(|| false);

    let toggle_menu = {
        let show_menu = show_menu.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            show_menu.set(!*show_menu);
        })
    };

    let handle_logout = {
        let auth_dispatch = auth_dispatch.clone();
        let session_dispatch = session_dispatch.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            auth_dispatch.reduce_mut(|state| {
                state.access_token = None;
                state.oauth_config = None;
            });
            session_dispatch.reduce_mut(|state| {
                state.selected_namespace = None;
                state.breadcrumbs.clear();
            });
            if let Some(window) = web_sys::window() {
                let _ = window.location().reload();
            }
        })
    };

    if auth_state.access_token.is_none() {
        return html! {};
    }

    html! {
        <div class="user-menu">
            <button class="user-menu-button" onclick={toggle_menu}>
                <span class="user-avatar">{"👤"}</span>
                <span class="user-name">{"User"}</span>
            </button>
            if *show_menu {
                <div class="user-menu-dropdown">
                    <div class="user-menu-item user-menu-profile">
                        <div class="profile-avatar">{"👤"}</div>
                        <div class="profile-info">
                            <div class="profile-name">{"User"}</div>
                            <div class="profile-role">{"Administrator"}</div>
                        </div>
                    </div>
                    <div class="user-menu-divider"></div>
                    <button class="user-menu-item user-menu-logout" onclick={handle_logout}>
                        <span class="menu-item-icon">{"🚪"}</span>
                        <span>{"Logout"}</span>
                    </button>
                </div>
            }
        </div>
    }
}
