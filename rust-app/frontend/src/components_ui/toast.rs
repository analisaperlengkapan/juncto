use leptos::*;

#[derive(Clone, Debug, PartialEq)]
pub enum ToastType {
    Info,
    Error,
    Warning,
    Success,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub toast_type: ToastType,
    pub persistent: bool,
    pub priority: u8, // Higher is more important
}

#[derive(Clone, Copy)]
pub struct ToastContext {
    pub toasts: RwSignal<Vec<Toast>>,
    counter: RwSignal<u64>,
}

impl ToastContext {
    pub fn add(&self, message: String, toast_type: ToastType) {
        self.add_advanced(message, toast_type, false, 0);
    }

    pub fn add_advanced(&self, message: String, toast_type: ToastType, persistent: bool, priority: u8) {
        let id = self.counter.get_untracked() + 1;
        self.counter.set(id);

        self.toasts.update(|t| {
            t.push(Toast {
                id,
                message,
                toast_type,
                persistent,
                priority,
            });
            // Sort by priority (descending)
            t.sort_by(|a, b| b.priority.cmp(&a.priority));
        });

        #[cfg(target_arch = "wasm32")]
        if !persistent {
            let toasts = self.toasts;
            set_timeout(
                move || {
                    toasts.update(|t| t.retain(|item| item.id != id));
                },
                std::time::Duration::from_secs(if priority > 0 { 6 } else { 3 }),
            );
        }
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
                        ToastType::Warning => "background: #ffc107; color: black;",
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
        let t4 = ToastType::Warning;
        assert_ne!(t1, t2);
        assert_ne!(t2, t3);
        assert_ne!(t3, t4);
    }

    #[test]
    fn test_toast_context_logic() {
        let _runtime = create_runtime();
        let ctx = ToastContext {
            toasts: create_rw_signal(Vec::new()),
            counter: create_rw_signal(0),
        };

        ctx.add_advanced("Msg 1".to_string(), ToastType::Info, false, 0);
        ctx.add_advanced("Msg 2".to_string(), ToastType::Error, true, 10);

        let toasts = ctx.toasts.get();
        assert_eq!(toasts.len(), 2);
        // Should be sorted by priority
        assert_eq!(toasts[0].message, "Msg 2");
        assert_eq!(toasts[1].message, "Msg 1");
        assert!(toasts[0].persistent);
        assert!(!toasts[1].persistent);
    }
}
