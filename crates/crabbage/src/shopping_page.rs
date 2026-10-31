#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces the server function body with a stub that does not await"
    )
)]

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::t;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct ShoppingRow {
    id: i64,
    ingredient: Option<String>,
    text: Option<String>,
    amount: Option<f64>,
    unit: Option<String>,
    checked: bool,
    section_id: Option<i64>,
    section_position: Option<i64>,
}

#[server(prefix = "/api", endpoint = "shopping_list")]
async fn shopping_list() -> Result<(Vec<ShoppingRow>, Vec<(i64, String)>), ServerFnError> {
    use crabbage_store::catalog::sections;
    use crabbage_store::households::active_household;
    use crabbage_store::shopping;
    let (pool, user) = crate::access::signed_in().await?;
    let household = active_household(&pool, user)
        .await
        .map_err(|error| crate::access::store_failure(&error))?
        .ok_or_else(|| ServerFnError::new("No active household"))?;
    let items = shopping::list(&pool, household)
        .await
        .map_err(|error| crate::access::store_failure(&error))?
        .into_iter()
        .map(|item| ShoppingRow {
            id: item.id,
            ingredient: item.ingredient,
            text: item.text,
            amount: item.amount,
            unit: item.unit,
            checked: item.checked,
            section_id: item.section_id,
            section_position: item.section_position,
        })
        .collect();
    let sections = sections::list(&pool)
        .await
        .map_err(|error| crate::access::store_failure(&error))?
        .into_iter()
        .map(|section| (section.id, section.name))
        .collect();
    Ok((items, sections))
}

#[component]
pub(crate) fn ShoppingPage() -> impl IntoView {
    let resource = Resource::new(|| (), |()| shopping_list());
    let rows = RwSignal::new(Vec::<ShoppingRow>::new());
    let sections = RwSignal::new(Vec::<(i64, String)>::new());
    #[cfg(feature = "hydrate")]
    connect_events(rows);
    view! {
        <h1>{t("shopping.heading")}</h1>
        <form class="shopping-add" method="post" action="/shopping/items">
            <label for="shopping-text">{t("shopping.add.label")}</label>
            <input id="shopping-text" name="text" type="text" required/>
            <button>{t("shopping.add.button")}</button>
        </form>
        <Suspense fallback=|| ()>
            {move || Suspend::new(async move {
                if let Ok((items, names)) = resource.await {
                    rows.set(items);
                    sections.set(names);
                }
                view! {
                    <Show when=move || rows.get().iter().any(|item| item.checked)>
                        <form method="post" action="/shopping/clear-checked"><button class="secondary">{t("shopping.clear")}</button></form>
                    </Show>
                    <Show when=move || rows.get().is_empty()>
                        <p>{t("shopping.empty")}</p>
                    </Show>
                    <For each=move || grouped_rows(rows.get(), &sections.get()) key=|group| group.0.clone() let(group)>
                        <section class="shopping-group">
                            <h2>{group.0.clone()}</h2>
                            <For each=move || group.1.clone() key=|item| item.id let(item)>
                                <form class="shopping-row" class:checked=item.checked method="post" action=format!("/shopping/items/{}/checked", item.id)>
                                    <input type="hidden" name="on" value=if item.checked { "false" } else { "true" }/>
                                    <button class="shopping-item" aria-pressed=item.checked>
                                        <span class="shopping-check" aria-hidden="true">{if item.checked { "✓" } else { "" }}</span>
                                        <span>{item_label(&item)}</span>
                                    </button>
                                </form>
                            </For>
                        </section>
                    </For>
                }
            })}
        </Suspense>
    }
}

#[derive(Clone)]
struct Group(String, Vec<ShoppingRow>);

fn grouped_rows(mut rows: Vec<ShoppingRow>, sections: &[(i64, String)]) -> Vec<Group> {
    rows.sort_by_key(|item| {
        (
            item.checked,
            item.section_position.unwrap_or(i64::MAX),
            item.section_id.unwrap_or(i64::MAX),
            item.checked,
            item.id,
        )
    });
    let mut groups = Vec::<Group>::new();
    for row in rows {
        let name = if row.checked {
            t("shopping.checked").to_owned()
        } else {
            row.section_id
                .and_then(|id| {
                    sections
                        .iter()
                        .find(|(section, _)| *section == id)
                        .map(|(_, name)| name.clone())
                })
                .unwrap_or_else(|| t("shopping.other").to_owned())
        };
        if groups.last().is_none_or(|group| group.0 != name) {
            groups.push(Group(name, Vec::new()));
        }
        if let Some(group) = groups.last_mut() {
            group.1.push(row);
        }
    }
    groups
}

fn item_label(item: &ShoppingRow) -> String {
    let name = item
        .ingredient
        .as_deref()
        .or(item.text.as_deref())
        .unwrap_or_default();
    match (item.amount, item.unit.as_deref()) {
        (Some(amount), Some(unit)) => format!("{amount} {unit} {name}"),
        (Some(amount), None) => format!("{amount} {name}"),
        _ => name.to_owned(),
    }
}

#[cfg(feature = "hydrate")]
fn connect_events(rows: RwSignal<Vec<ShoppingRow>>) {
    use std::cell::RefCell;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::Closure;
    use web_sys::{Event, EventSource, MessageEvent};
    type Source = (EventSource, Closure<dyn FnMut(Event)>);
    let Ok(source) = EventSource::new("/shopping/events") else {
        return;
    };
    let callback = Closure::<dyn FnMut(Event)>::new(move |event: Event| {
        if let Ok(message) = event.dyn_into::<MessageEvent>()
            && let Some(json) = message.data().as_string()
            && let Ok(items) = serde_json::from_str::<Vec<ShoppingRow>>(&json)
        {
            rows.set(items);
        }
    });
    let _listener_result =
        source.add_event_listener_with_callback("list", callback.as_ref().unchecked_ref());
    thread_local! {
        static SOURCE: RefCell<Option<Source>> = const { RefCell::new(None) };
    }
    SOURCE.with(|current| {
        if let Some((previous, _)) = current.borrow_mut().replace((source, callback)) {
            previous.close();
        }
    });
    on_cleanup(move || {
        SOURCE.with(|current| {
            if let Some((source, callback)) = current.borrow_mut().take() {
                source.close();
                drop(callback);
            }
        });
    });
}
