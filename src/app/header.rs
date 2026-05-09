use leptos::{html::Input, prelude::*};
use reactive_stores::Store;
use wasm_bindgen::{JsCast, prelude::Closure};
use web_sys::{Event, FileReader, HtmlInputElement};

use crate::{LoadDbOptions, WorkerRequest, app::{AppState, AppStateStoreFields}, send_request};

#[component]
pub fn Header() -> impl IntoView {
    let state = expect_context::<Store<AppState>>();
    let input_ref = NodeRef::<Input>::new();

    let (file_signal, set_file) = signal::<Option<crate::FragileConfirmed<web_sys::File>>>(None);

    Effect::new(move || {
        if let Some(file) = &*file_signal.read() {
            let filename = file.name();
            state.db_filename().set(Some(filename.clone()));

            if let Ok(reader) = FileReader::new() {
                let on_load =
                    crate::FragileConfirmed::new(Closure::wrap(Box::new(move |ev: Event| {
                        let target = ev.target().unwrap();
                        let reader = target.unchecked_into::<FileReader>();
                        let result = reader.result().unwrap();
                        let array_buffer = result.unchecked_into::<js_sys::ArrayBuffer>();
                        let data = js_sys::Uint8Array::new(&array_buffer);
                        send_request(state, WorkerRequest::LoadDb(LoadDbOptions { data }));
                    }) as Box<dyn FnMut(_)>));

                reader.set_onload(Some(on_load.as_ref().unchecked_ref()));
                reader.read_as_array_buffer(&**file).unwrap();

                on_cleanup(move || drop(on_load));
            }
        }
    });

    let on_open = move |_| {
        if let Some(input) = &*input_ref.read() {
            if input.onchange().is_none() {
                let callback = Closure::wrap(Box::new(move |ev: Event| {
                    if let Some(target) = ev.target() {
                        let input = target.unchecked_into::<HtmlInputElement>();
                        if let Some(files) = input.files() {
                            if files.length() > 0 {
                                set_file
                                    .set(Some(crate::FragileConfirmed::new(files.get(0).unwrap())));
                            }
                        }
                    }
                }) as Box<dyn Fn(Event)>);
                input.set_onchange(Some(callback.as_ref().unchecked_ref::<js_sys::Function>()));
                callback.forget();
            }
            input.set_value("");
            input.click();
        }
    };

    view! {
        <div class="header">
            <div class="header-brand">
                <span class="header-icon">"🗄️"</span>
                <span class="header-title">"Data Explorer"</span>
            </div>
            <div class="header-actions">
                <Show
                    when=move || state.db_filename().read().is_some()
                    fallback=|| ()
                >
                    <span class="header-db-name">
                        {move || state.db_filename().read().clone().unwrap_or_default()}
                    </span>
                </Show>
                <input
                    type="file"
                    node_ref=input_ref
                    style="display:none"
                    accept=".db,.sqlite,.sqlite3"
                />
                <button class="btn-primary" on:click=on_open>
                    "Open Database"
                </button>
            </div>
        </div>
        <Show
            when=move || state.error().read().is_some()
            fallback=|| ()
        >
            <div class="error-bar">
                {move || state.error().read().clone().unwrap_or_default()}
            </div>
        </Show>
    }
}
