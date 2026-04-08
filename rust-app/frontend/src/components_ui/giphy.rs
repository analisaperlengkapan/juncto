use leptos::*;
use crate::giphy::{GiphyData, GiphyService};
use gloo_timers::callback::Timeout;
use std::cell::RefCell;
use std::rc::Rc;

#[component]
pub fn GiphySearch(
    on_select: Callback<String>, // returns the URL
) -> impl IntoView {
    let (query, set_query) = create_signal("".to_string());
    let (debounced_query, set_debounced_query) = create_signal("".to_string());
    let (gifs, set_gifs) = create_signal(Vec::<GiphyData>::new());
    let (is_loading, set_is_loading) = create_signal(false);

    // Debounce: only update debounced_query 300ms after the last keystroke
    let debounce_handle: Rc<RefCell<Option<Timeout>>> = Rc::new(RefCell::new(None));
    create_effect({
        let debounce_handle = debounce_handle.clone();
        move |_| {
            let q = query.get();
            // Cancel previous timer
            debounce_handle.borrow_mut().take();
            let handle = Timeout::new(300, move || {
                set_debounced_query.set(q);
            });
            *debounce_handle.borrow_mut() = Some(handle);
        }
    });

    let search_action = create_action(move |q: &String| {
        let q = q.clone();
        let service = GiphyService::new(crate::giphy::GIPHY_API_KEY.to_string());
        async move {
            service.search(&q).await
        }
    });

    // Track whether this is the first run to avoid an unnecessary trending API
    // request the instant the GIF panel opens.  Only dispatch after the user has
    // actually typed something (or explicitly cleared the field).
    let has_interacted = Rc::new(RefCell::new(false));
    create_effect({
        let has_interacted = has_interacted.clone();
        move |_| {
            let q = debounced_query.get();
            if *has_interacted.borrow() {
                search_action.dispatch(q);
            }
        }
    });

    create_effect(move |_| {
        if let Some(res) = search_action.value().get() {
            match res {
                Ok(data) => set_gifs.set(data),
                Err(e) => web_sys::console::error_1(&e.into()),
            }
            set_is_loading.set(false);
        }
    });

    let has_interacted_for_input = has_interacted.clone();
    view! {
        <div class="giphy-search" style="display: flex; flex-direction: column; gap: 10px; padding: 10px; background: #222; border-radius: 8px;">
            <input
                type="text"
                placeholder="Search GIPHY..."
                on:input=move |ev| {
                    *has_interacted_for_input.borrow_mut() = true;
                    set_query.set(event_target_value(&ev));
                    set_is_loading.set(true);
                }
                prop:value=query
                style="padding: 8px; border-radius: 4px; border: 1px solid #444; background: #333; color: white;"
            />

            <div class="giphy-grid" style="display: grid; grid-template-columns: repeat(2, 1fr); gap: 5px; max-height: 200px; overflow-y: auto;">
                <Show when=move || is_loading.get()>
                    <div style="grid-column: span 2; text-align: center; color: #888;">"Loading..."</div>
                </Show>

                <For
                    each=move || gifs.get()
                    key=|gif| gif.id.clone()
                    children=move |gif| {
                        let url = gif.images.fixed_height.url.clone();
                        let title = gif.title.clone();
                        let url_clone = url.clone();
                        view! {
                            <img
                                src=url
                                alt=title
                                on:click=move |_| on_select.call(url_clone.clone())
                                style="width: 100%; cursor: pointer; border-radius: 4px;"
                            />
                        }
                    }
                />
            </div>

            <div style="text-align: right;">
                <img src="https://giphy.com/static/img/powered_by_giphy_light.png" alt="Powered by GIPHY" style="height: 20px;" />
            </div>
        </div>
    }
}
