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
                                set_file.set(Some(crate::FragileConfirmed::new(
                                    files.get(0).unwrap(),
                                )));
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
        <header class="flex items-center justify-between h-12 px-4 bg-slate-800 border-b border-slate-700 shrink-0 z-10">
            <div class="flex items-center gap-2">
                <span class="text-blue-400 text-lg">"🗄️"</span>
                <span class="text-sm font-semibold text-slate-100 tracking-tight">"Data Explorer"</span>
                <Show when=move || state.db_filename().read().is_some() fallback=|| ()>
                    <span class="text-slate-600 mx-1">"/"</span>
                    <span class="text-xs text-slate-400 font-mono bg-slate-700 px-2 py-0.5 rounded max-w-48 truncate">
                        {move || state.db_filename().read().clone().unwrap_or_default()}
                    </span>
                </Show>
            </div>
            <div class="flex items-center gap-3">
                <input
                    type="file"
                    node_ref=input_ref
                    style="display:none"
                    accept=".db,.sqlite,.sqlite3"
                />
                <button
                    class="text-xs font-medium bg-blue-600 hover:bg-blue-500 text-white px-3 py-1.5 rounded transition-colors"
                    on:click=on_open
                >
                    "Open Database"
                </button>
            </div>
        </header>
        <Show when=move || state.error().read().is_some() fallback=|| ()>
            <div class="bg-red-950 border-b border-red-800 text-red-300 text-xs px-4 py-2 shrink-0">
                {move || state.error().read().clone().unwrap_or_default()}
            </div>
        </Show>
    }
}
