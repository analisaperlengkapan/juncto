use leptos::*;
use shared::{ClientMessage, SalesforceConfig};

#[derive(Clone)]
pub struct SalesforceService {
    pub send_signal: Callback<ClientMessage>,
}

impl SalesforceService {
    pub fn new(send_signal: Callback<ClientMessage>) -> Self {
        Self { send_signal }
    }

    pub fn link_object(&self, object_id: String, object_type: String) {
        logging::log!(
            "SalesforceService: Linking {} as {}",
            object_id,
            object_type
        );
        self.send_signal
            .call(ClientMessage::LinkSalesforce(SalesforceConfig {
                is_linked: true,
                object_id: Some(object_id),
                object_type: Some(object_type),
            }));
    }

    pub fn unlink_object(&self) {
        logging::log!("SalesforceService: Unlinking");
        self.send_signal
            .call(ClientMessage::LinkSalesforce(SalesforceConfig {
                is_linked: false,
                object_id: None,
                object_type: None,
            }));
    }
}

pub fn provide_salesforce_context(send_signal: Callback<ClientMessage>) {
    provide_context(SalesforceService::new(send_signal));
}

pub fn use_salesforce() -> SalesforceService {
    use_context::<SalesforceService>().expect("SalesforceService not provided")
}

#[component]
pub fn LinkSalesforceDialog(
    show: ReadSignal<bool>,
    on_close: Callback<()>,
    #[prop(into)] config: Signal<SalesforceConfig>,
) -> impl IntoView {
    let service = use_salesforce();
    let (object_id, set_object_id) = create_signal(String::new());
    let (object_type, set_object_type) = create_signal("Lead".to_string());

    create_effect(move |_| {
        if show.get() {
            let current = config.get();
            set_object_id.set(current.object_id.unwrap_or_default());
            set_object_type.set(current.object_type.unwrap_or_else(|| "Lead".to_string()));
        }
    });

    let service_sv = store_value(service);
    let on_close_sv = store_value(on_close);

    view! {
        <Show when=move || show.get()>
            <div class="modal-overlay" style="position: fixed; top: 0; left: 0; width: 100%; height: 100%; background: rgba(0,0,0,0.5); display: flex; justify-content: center; align-items: center; z-index: 1000;">
                <div class="modal-content" style="background: white; color: #333; padding: 20px; border-radius: 8px; width: 400px; max-width: 90%;">
                    <div class="modal-header" style="display: flex; justify-content: space-between; margin-bottom: 20px;">
                        <h3>"Salesforce Integration"</h3>
                        <button on:click=move |_| on_close_sv.with_value(|cb| cb.call(())) style="background: none; border: none; font-size: 20px; cursor: pointer;">"×"</button>
                    </div>

                    <div class="form-group" style="margin-bottom: 15px;">
                        <label style="display: block; margin-bottom: 5px;">"Object Type"</label>
                        <select
                            on:change=move |ev| set_object_type.set(event_target_value(&ev))
                            prop:value=object_type
                            style="width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;"
                        >
                            <option value="Lead">"Lead"</option>
                            <option value="Opportunity">"Opportunity"</option>
                            <option value="Account">"Account"</option>
                            <option value="Contact">"Contact"</option>
                        </select>
                    </div>

                    <div class="form-group" style="margin-bottom: 15px;">
                        <label style="display: block; margin-bottom: 5px;">"Object ID"</label>
                        <input
                            type="text"
                            placeholder="e.g. 00Q... or 006..."
                            on:input=move |ev| set_object_id.set(event_target_value(&ev))
                            prop:value=object_id
                            style="width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px; box-sizing: border-box;"
                        />
                    </div>

                    <div style="display: flex; justify-content: flex-end; gap: 10px; margin-top: 20px;">
                        <Show when=move || config.get().is_linked>
                            <button
                                id="unlink-salesforce-btn"
                                on:click=move |_| {
                                    service_sv.with_value(|s| s.unlink_object());
                                    on_close_sv.with_value(|cb| cb.call(()));
                                }
                                style="padding: 10px 20px; background-color: #dc3545; color: white; border: none; border-radius: 4px; cursor: pointer;"
                            >
                                "Unlink"
                            </button>
                        </Show>
                        <button
                            id="link-salesforce-btn"
                            on:click=move |_| {
                                service_sv.with_value(|s| s.link_object(object_id.get(), object_type.get()));
                                on_close_sv.with_value(|cb| cb.call(()));
                            }
                            disabled=move || object_id.get().is_empty()
                            style=move || format!("padding: 10px 20px; background-color: #007bff; color: white; border: none; border-radius: 4px; cursor: {}; opacity: {};", if object_id.get().is_empty() { "not-allowed" } else { "pointer" }, if object_id.get().is_empty() { "0.5" } else { "1" })
                        >
                            {move || if config.get().is_linked { "Update Link" } else { "Link Meeting" }}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::ClientMessage;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_salesforce_service_link() {
        let _runtime = create_runtime();
        let last_msg = Rc::new(RefCell::new(None::<ClientMessage>));
        let last_msg_clone = last_msg.clone();

        let service = SalesforceService::new(Callback::new(move |msg| {
            *last_msg_clone.borrow_mut() = Some(msg);
        }));

        service.link_object("001...".to_string(), "Account".to_string());

        let msg = last_msg.borrow().clone();
        if let Some(ClientMessage::LinkSalesforce(config)) = msg {
            assert!(config.is_linked);
            assert_eq!(config.object_id.unwrap(), "001...");
            assert_eq!(config.object_type.unwrap(), "Account");
        } else {
            panic!("Expected LinkSalesforce message");
        }
    }

    #[test]
    fn test_salesforce_service_unlink() {
        let _runtime = create_runtime();
        let last_msg = Rc::new(RefCell::new(None::<ClientMessage>));
        let last_msg_clone = last_msg.clone();

        let service = SalesforceService::new(Callback::new(move |msg| {
            *last_msg_clone.borrow_mut() = Some(msg);
        }));

        service.unlink_object();

        let msg = last_msg.borrow().clone();
        if let Some(ClientMessage::LinkSalesforce(config)) = msg {
            assert!(!config.is_linked);
            assert!(config.object_id.is_none());
        } else {
            panic!("Expected LinkSalesforce message");
        }
    }
}
