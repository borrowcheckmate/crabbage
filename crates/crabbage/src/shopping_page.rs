#![cfg_attr(
    not(feature = "ssr"),
    expect(
        clippy::unused_async_trait_impl,
        reason = "the client build replaces the server function body with a stub that does not await"
    )
)]

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{t, t_with};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct ShoppingRow {
    ids: Vec<i64>,
    label: String,
    quantity: String,
    ingredient_id: Option<i64>,
    ingredient: Option<String>,
    text: Option<String>,
    amount: Option<f64>,
    unit: Option<String>,
    amounts: Vec<(f64, String)>,
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
            ids: vec![item.id],
            label: String::new(),
            quantity: String::new(),
            ingredient_id: item.ingredient_id,
            ingredient: item.ingredient,
            text: item.text,
            amount: item.amount,
            unit: item.unit,
            amounts: Vec::new(),
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
    let load_failed = RwSignal::new(false);
    #[cfg(feature = "hydrate")]
    connect_events(rows, load_failed);
    view! {
        <h1>{t("shopping.heading")}</h1>
        <Suspense fallback=|| ()>
            {move || Suspend::new(async move {
                match resource.await {
                    Ok((items, names)) => {
                        rows.set(merge_rows(items));
                        sections.set(names);
                    }
                    Err(_) => load_failed.set(true),
                }
                view! {
                    <p class="shopping-count">{move || remaining_text(rows.get().iter().filter(|item| !item.checked).count())}</p>
                    <form class="shopping-add" method="post" action="/shopping/items">
                        <label class="shopping-visually-hidden" for="shopping-text">{t("shopping.add.label")}</label>
                        <input id="shopping-text" name="text" type="text" placeholder=t("shopping.add.placeholder") required/>
                        <button type="submit" aria-label=t("shopping.add.button")><svg aria-hidden="true" viewBox="0 0 24 24"><path d="M12 5v14M5 12h14"/></svg></button>
                    </form>
                    <Show when=move || load_failed.get()><p class="shopping-error" role="status">{t("shopping.error")}</p></Show>
                    <div class="shopping-layout"><div class="shopping-list">
                    <Show when=move || rows.get().is_empty()>
                        <p>{t("shopping.empty")}</p>
                    </Show>
                    <For each=move || grouped_rows(rows.get(), &sections.get()) key=|group| group.0.clone() let(group)>
                        <section class="shopping-group">
                            <h2>{group.0.clone()}</h2>
                            <For each=move || group.1.clone() key=|item| item.ids.first().copied().unwrap_or_default() let(item)>
                                <form class="shopping-row" class:checked=item.checked method="post" action="/shopping/checked">
                                    <input type="hidden" name="ids" value={item.ids.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")}/>
                                    <input type="hidden" name="on" value=if item.checked { "false" } else { "true" }/>
                                    <button class="shopping-item" aria-pressed=item.checked>
                                        <span class="shopping-check" aria-hidden="true">{if item.checked { "✓" } else { "" }}</span>
                                        <span class="shopping-name">{item.label.clone()}</span>
                                        <span class="shopping-quantity">{item.quantity.clone()}</span>
                                    </button>
                                </form>
                            </For>
                        </section>
                    </For>
                    <Show when=move || rows.get().iter().any(|item| item.checked)>
                        <form class="shopping-clear" method="post" action="/shopping/clear-checked"><button class="quiet">{t("shopping.clear")}</button></form>
                    </Show>
                    </div><aside class="shopping-review"><span class="shopping-eyebrow">{t("shopping.next")}</span><h2>{t("shopping.review.heading")}</h2><p>{t("shopping.review.copy")}</p><button type="button" disabled=true>{t("shopping.review.button")}<svg aria-hidden="true" viewBox="0 0 24 24"><path d="M5 12h14m-7-7 7 7-7 7"/></svg></button><p class="shopping-note">{t("shopping.review.note")}</p></aside></div>
                }
            })}
        </Suspense>
    }
}

#[derive(Clone)]
struct Group(String, Vec<ShoppingRow>);

#[derive(Deserialize)]
#[cfg(feature = "hydrate")]
struct ShoppingEventRow {
    id: i64,
    ingredient_id: Option<i64>,
    ingredient: Option<String>,
    text: Option<String>,
    amount: Option<f64>,
    unit: Option<String>,
    checked: bool,
    section_id: Option<i64>,
    section_position: Option<i64>,
}

#[cfg(feature = "hydrate")]
type ShoppingListener = wasm_bindgen::prelude::Closure<dyn FnMut(web_sys::Event)>;

#[cfg(feature = "hydrate")]
type ShoppingEventSource = (web_sys::EventSource, ShoppingListener, ShoppingListener);

#[cfg(feature = "hydrate")]
thread_local! {
    static SOURCE: std::cell::RefCell<Option<ShoppingEventSource>> = const { std::cell::RefCell::new(None) };
}

fn grouped_rows(mut rows: Vec<ShoppingRow>, sections: &[(i64, String)]) -> Vec<Group> {
    rows.sort_by_key(|item| {
        (
            item.checked,
            item.section_position.unwrap_or(i64::MAX),
            item.section_id.unwrap_or(i64::MAX),
            item.checked,
            item.ids.first().copied().unwrap_or_default(),
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

fn merge_rows(rows: Vec<ShoppingRow>) -> Vec<ShoppingRow> {
    row_groups(rows)
        .into_iter()
        .filter_map(|group| merge_group(&group))
        .collect()
}

fn row_groups(rows: Vec<ShoppingRow>) -> Vec<Vec<ShoppingRow>> {
    use crabbage_domain::quantity::Unit;
    use std::collections::HashMap;
    let mut groups = Vec::<Vec<ShoppingRow>>::new();
    let mut ingredient_groups = HashMap::<(i64, bool), usize>::new();
    for row in rows {
        let known = row
            .unit
            .as_deref()
            .is_none_or(|unit| Unit::from_symbol(unit).is_some());
        let Some(id) = row.ingredient_id.filter(|_| known) else {
            groups.push(vec![row]);
            continue;
        };
        let key = (id, row.checked);
        let index = *ingredient_groups.entry(key).or_insert_with(|| {
            groups.push(Vec::new());
            groups.len() - 1
        });
        if let Some(group) = groups.get_mut(index) {
            group.push(row);
        }
    }
    groups
}

fn merge_group(group: &[ShoppingRow]) -> Option<ShoppingRow> {
    use crabbage_domain::quantity::{Quantity, Unit};
    use crabbage_domain::shopping::{Item, merge};
    let first = group.first()?.clone();
    if first
        .unit
        .as_deref()
        .is_some_and(|unit| Unit::from_symbol(unit).is_none())
    {
        return Some(singleton_row(first));
    }
    let Some(id) = first.ingredient_id else {
        return Some(singleton_row(first));
    };
    let merged = merge(group.iter().map(|row| {
        let amount = row.amount.and_then(|amount| {
            let unit = row
                .unit
                .as_deref()
                .and_then(Unit::from_symbol)
                .unwrap_or(Unit::Piece);
            Quantity::new(amount, unit)
        });
        (Item::Ingredient(id), amount)
    }));
    let amounts = merged
        .first()
        .map(|line| {
            line.amounts
                .iter()
                .map(|amount| {
                    (
                        amount.amount(),
                        if amount.unit() == Unit::Piece {
                            String::new()
                        } else {
                            amount.unit().symbol().to_owned()
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    let ids = group
        .iter()
        .flat_map(|row| row.ids.iter().copied())
        .collect();
    let mut result = first;
    result.ids = ids;
    result.amounts = amounts;
    result.quantity = quantities_text(&result.amounts);
    result.label = result.ingredient.clone().unwrap_or_default();
    Some(result)
}

fn singleton_row(mut row: ShoppingRow) -> ShoppingRow {
    let name = row
        .ingredient
        .as_deref()
        .or(row.text.as_deref())
        .unwrap_or_default();
    row.amounts = row
        .amount
        .map(|amount| vec![(amount, row.unit.clone().unwrap_or_default())])
        .unwrap_or_default();
    row.quantity = quantities_text(&row.amounts);
    row.label = name.to_owned();
    row
}

fn quantities_text(amounts: &[(f64, String)]) -> String {
    amounts
        .iter()
        .map(|(amount, unit)| {
            let value = format!("{amount:.2}")
                .trim_end_matches('0')
                .trim_end_matches('.')
                .to_owned();
            if unit.is_empty() {
                value
            } else {
                format!("{value} {unit}")
            }
        })
        .collect::<Vec<_>>()
        .join(" + ")
}

fn remaining_text(count: usize) -> String {
    let key = if count == 1 {
        "shopping.remaining.one"
    } else {
        "shopping.remaining.many"
    };
    t_with(key, &[("count", &count.to_string())])
}

#[cfg(feature = "hydrate")]
fn connect_events(rows: RwSignal<Vec<ShoppingRow>>, error: RwSignal<bool>) {
    let Ok(source) = web_sys::EventSource::new("/shopping/events") else {
        return;
    };
    let list_callback = list_listener(rows, error);
    let error_callback = error_listener(error);
    listen_for_updates(&source, &list_callback, &error_callback);
    replace_event_source((source, list_callback, error_callback));
    on_cleanup(close_event_source);
}

#[cfg(feature = "hydrate")]
fn list_listener(rows: RwSignal<Vec<ShoppingRow>>, error: RwSignal<bool>) -> ShoppingListener {
    use wasm_bindgen::prelude::Closure;
    Closure::<dyn FnMut(web_sys::Event)>::new(move |event| update_from_event(event, rows, error))
}

#[cfg(feature = "hydrate")]
fn update_from_event(
    event: web_sys::Event,
    rows: RwSignal<Vec<ShoppingRow>>,
    error: RwSignal<bool>,
) {
    use wasm_bindgen::JsCast;
    let Ok(message) = event.dyn_into::<web_sys::MessageEvent>() else {
        return;
    };
    let Some(json) = message.data().as_string() else {
        return;
    };
    let Ok(items) = serde_json::from_str::<Vec<ShoppingEventRow>>(&json) else {
        return;
    };
    rows.set(merge_rows(items.into_iter().map(event_row).collect()));
    error.set(false);
}

#[cfg(feature = "hydrate")]
fn event_row(item: ShoppingEventRow) -> ShoppingRow {
    ShoppingRow {
        ids: vec![item.id],
        label: String::new(),
        quantity: String::new(),
        ingredient_id: item.ingredient_id,
        ingredient: item.ingredient,
        text: item.text,
        amount: item.amount,
        unit: item.unit,
        amounts: Vec::new(),
        checked: item.checked,
        section_id: item.section_id,
        section_position: item.section_position,
    }
}

#[cfg(feature = "hydrate")]
fn error_listener(error: RwSignal<bool>) -> ShoppingListener {
    use wasm_bindgen::prelude::Closure;
    Closure::<dyn FnMut(web_sys::Event)>::new(move |_event| error.set(true))
}

#[cfg(feature = "hydrate")]
fn listen_for_updates(
    source: &web_sys::EventSource,
    list: &ShoppingListener,
    error: &ShoppingListener,
) {
    use wasm_bindgen::JsCast;
    let _list_result =
        source.add_event_listener_with_callback("list", list.as_ref().unchecked_ref());
    let _error_result =
        source.add_event_listener_with_callback("error", error.as_ref().unchecked_ref());
}

#[cfg(feature = "hydrate")]
fn replace_event_source(source: ShoppingEventSource) {
    SOURCE.with(|current| {
        if let Some((previous, _, _)) = current.borrow_mut().replace(source) {
            previous.close();
        }
    });
}

#[cfg(feature = "hydrate")]
fn close_event_source() {
    SOURCE.with(|current| {
        if let Some((source, list, error)) = current.borrow_mut().take() {
            source.close();
            drop(list);
            drop(error);
        }
    });
}
