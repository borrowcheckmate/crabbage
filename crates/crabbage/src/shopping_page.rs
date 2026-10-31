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
    ids: Vec<i64>,
    label: String,
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
                    rows.set(merge_rows(items));
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
                            <For each=move || group.1.clone() key=|item| item.ids.first().copied().unwrap_or_default() let(item)>
                                <form class="shopping-row" class:checked=item.checked method="post" action="/shopping/checked">
                                    <input type="hidden" name="ids" value={item.ids.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")}/>
                                    <input type="hidden" name="on" value=if item.checked { "false" } else { "true" }/>
                                    <button class="shopping-item" aria-pressed=item.checked>
                                        <span class="shopping-check" aria-hidden="true">{if item.checked { "✓" } else { "" }}</span>
                                        <span>{item.label.clone()}</span>
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
    result.label = format_line(
        result.ingredient.as_deref().unwrap_or_default(),
        &result.amounts,
    );
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
    row.label = format_line(name, &row.amounts);
    row
}

fn format_line(name: &str, amounts: &[(f64, String)]) -> String {
    let quantities = amounts
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
        .join(" + ");
    if quantities.is_empty() {
        name.to_owned()
    } else {
        format!("{name}: {quantities}")
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
            && let Ok(items) = serde_json::from_str::<Vec<ShoppingEventRow>>(&json)
        {
            rows.set(merge_rows(
                items
                    .into_iter()
                    .map(|item| ShoppingRow {
                        ids: vec![item.id],
                        label: String::new(),
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
                    .collect(),
            ));
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
