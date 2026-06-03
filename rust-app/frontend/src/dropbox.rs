use leptos::*;
use shared::ClientMessage;

#[derive(Clone)]
pub struct DropboxService {
    pub send_signal: Callback<ClientMessage>,
}

impl DropboxService {
    pub fn new(send_signal: Callback<ClientMessage>) -> Self {
        Self { send_signal }
    }

    pub fn save_file(&self, filename: String) {
        self.send_signal.call(ClientMessage::SaveToDropbox(filename));
    }
}

pub fn provide_dropbox_context(send_signal: Callback<ClientMessage>) {
    provide_context(DropboxService::new(send_signal));
}

pub fn use_dropbox() -> DropboxService {
    use_context::<DropboxService>().expect("DropboxService not provided")
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::ClientMessage;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn test_dropbox_service_save() {
        let _runtime = create_runtime();
        let last_msg = Rc::new(RefCell::new(None::<ClientMessage>));
        let last_msg_clone = last_msg.clone();

        let service = DropboxService::new(Callback::new(move |msg| {
            *last_msg_clone.borrow_mut() = Some(msg);
        }));

        service.save_file("test.pdf".to_string());

        let msg = last_msg.borrow().clone();
        if let Some(ClientMessage::SaveToDropbox(filename)) = msg {
            assert_eq!(filename, "test.pdf");
        } else {
            panic!("Expected SaveToDropbox message");
        }
    }
}
