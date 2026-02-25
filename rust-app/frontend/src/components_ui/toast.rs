use leptos::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ToastType {
    Info,
    Error,
    #[allow(dead_code)]
    Success,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub toast_type: ToastType,
}

#[derive(Clone, Copy)]
pub struct ToastContext {
    pub toasts: RwSignal<Vec<Toast>>,
    counter: RwSignal<u64>,
}

impl ToastContext {
    pub fn add(&self, message: String, toast_type: ToastType) {
        let id = self.counter.get() + 1;
        self.counter.set(id);

        self.toasts.update(|t| {
            t.push(Toast {
                id,
                message,
                toast_type,
            })
        });

        let toasts = self.toasts;
        set_timeout(
            move || {
                toasts.update(|t| t.retain(|item| item.id != id));
            },
            std::time::Duration::from_secs(3),
        );
    }

    pub fn remove(&self, id: u64) {
        self.toasts.update(|t| t.retain(|item| item.id != id));
    }
}

pub fn provide_toast_context() {
    provide_context(ToastContext {
        toasts: create_rw_signal(Vec::new()),
        counter: create_rw_signal(0),
    });
}

pub fn use_toast() -> ToastContext {
    use_context::<ToastContext>().expect("ToastContext must be provided")
}

#[component]
pub fn ToastContainer() -> impl IntoView {
    let ctx = use_toast();

    view! {
        <div class="toast-container" style="position: fixed; top: 20px; right: 20px; z-index: 10000; display: flex; flex-direction: column; gap: 10px;">
            <For
                each=move || ctx.toasts.get()
                key=|t| t.id
                children=move |t| {
                    let style = match t.toast_type {
                        ToastType::Info => "background: #007bff; color: white;",
                        ToastType::Error => "background: #dc3545; color: white;",
                        ToastType::Success => "background: #28a745; color: white;",
                    };
                    let id = t.id;
                    view! {
                        <div
                            class="toast"
                            style=format!("padding: 10px 20px; border-radius: 4px; box-shadow: 0 2px 4px rgba(0,0,0,0.2); min-width: 200px; animation: fadein 0.3s; cursor: pointer; {}", style)
                            on:click=move |_| ctx.remove(id)
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
