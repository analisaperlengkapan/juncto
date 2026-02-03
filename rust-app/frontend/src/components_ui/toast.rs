use leptos::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ToastMessage {
    pub id: u64,
    pub message: String,
    pub type_: ToastType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ToastType {
    #[allow(dead_code)]
    Info,
    Error,
    #[allow(dead_code)]
    Success,
}

#[component]
pub fn ToastContainer(
    toasts: ReadSignal<Vec<ToastMessage>>,
    on_dismiss: Callback<u64>,
) -> impl IntoView {
    view! {
        <div class="toast-container" style="position: fixed; top: 20px; right: 20px; z-index: 10000; display: flex; flex-direction: column; gap: 10px;">
            <For
                each=move || toasts.get()
                key=|t| t.id
                children=move |t| {
                    let id = t.id;
                    let style = match t.type_ {
                        ToastType::Info => "background: #007bff; color: white;",
                        ToastType::Error => "background: #dc3545; color: white;",
                        ToastType::Success => "background: #28a745; color: white;",
                    };
                    view! {
                        <div
                            class="toast"
                            style=format!("padding: 10px 20px; border-radius: 4px; box-shadow: 0 2px 4px rgba(0,0,0,0.2); min-width: 200px; cursor: pointer; {}", style)
                            on:click=move |_| on_dismiss.call(id)
                        >
                            {t.message}
                        </div>
                    }
                }
            />
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toast_types() {
        let t1 = ToastType::Info;
        let t2 = ToastType::Error;
        let t3 = ToastType::Success;
        assert_ne!(t1, t2);
        assert_ne!(t2, t3);
    }
}
