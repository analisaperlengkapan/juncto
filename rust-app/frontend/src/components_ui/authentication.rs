use leptos::*;

#[component]
pub fn LoginDialog(
    #[prop(into)] auth_error: Signal<Option<String>>,
    on_login: Callback<(String, Option<String>)>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let (username, set_username) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());

    let handle_submit = move |_| {
        let u = username.get();
        let p = password.get();
        if !u.is_empty() {
            let pass = if p.is_empty() { None } else { Some(p) };
            on_login.call((u, pass));
        }
    };

    view! {
        <div class="login-dialog-overlay" style="position: fixed; top: 0; left: 0; width: 100vw; height: 100vh; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;">
            <div class="login-dialog" style="background: #2a2a2a; padding: 20px; border-radius: 8px; width: 300px; color: white;">
                <h3 style="margin-top: 0; margin-bottom: 15px;">"Authentication Required"</h3>
                <div class="form-group" style="margin-bottom: 15px;">
                    <label style="display: block; margin-bottom: 5px;">"Username"</label>
                    <input
                        type="text"
                        placeholder="user@domain.com"
                        on:input=move |ev| set_username.set(event_target_value(&ev))
                        prop:value=username
                        style="width: 100%; padding: 8px; border-radius: 4px; border: 1px solid #444; background: #111; color: white; box-sizing: border-box;"
                    />
                </div>
                <div class="form-group" style="margin-bottom: 15px;">
                    <label style="display: block; margin-bottom: 5px;">"Password"</label>
                    <input
                        type="password"
                        placeholder="Password"
                        on:input=move |ev| set_password.set(event_target_value(&ev))
                        prop:value=password
                        style="width: 100%; padding: 8px; border-radius: 4px; border: 1px solid #444; background: #111; color: white; box-sizing: border-box;"
                    />
                </div>
                <Show when=move || auth_error.get().is_some()>
                    <div style="color: #ff4444; margin-bottom: 15px; font-size: 14px;">
                        {move || auth_error.get().unwrap_or_default()}
                    </div>
                </Show>
                <div style="display: flex; justify-content: flex-end; gap: 10px;">
                    <button
                        on:click=move |_| on_cancel.call(())
                        style="padding: 8px 16px; background: #444; color: white; border: none; border-radius: 4px; cursor: pointer;"
                    >
                        "Cancel"
                    </button>
                    <button
                        on:click=handle_submit
                        disabled=move || username.get().is_empty()
                        style=move || format!("padding: 8px 16px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: {}; opacity: {};", if username.get().is_empty() { "not-allowed" } else { "pointer" }, if username.get().is_empty() { "0.5" } else { "1" })
                    >
                        "Login"
                    </button>
                </div>
            </div>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_login_dialog_compiles() {
        let _ = create_runtime();
        let (auth_error, _set_auth_error) = create_signal::<Option<String>>(None);
        let on_login = Callback::new(|_: (String, Option<String>)| {});
        let on_cancel = Callback::new(|_: ()| {});

        let _view = LoginDialog(
            LoginDialogProps {
                auth_error: auth_error.into(),
                on_login,
                on_cancel,
            }
        );
        assert!(true); // Verifies that instantiation succeeds within a reactive scope
    }
}
