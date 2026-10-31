//! Shopping list lines: items for the same ingredient merge into one line with combined amounts.
use crate::quantity::Quantity;

/// What a shopping item is: a catalogue ingredient or text the user typed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Item {
    /// A catalogue ingredient, by id.
    Ingredient(i64),
    /// Free text; never merged.
    Text(String),
}

/// One line of the list: an item and its amounts, at most one amount per dimension or unitless unit.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// The item this line is about.
    pub item: Item,
    /// The combined amounts. Amounts that cannot be added (another dimension, another unitless unit) stay apart.
    pub amounts: Vec<Quantity>,
}

/// Merges items into lines in first-seen order.
///
/// Items with the same [`Item::Ingredient`] id share one line, placed where the first of them
/// appeared. [`Item::Text`] items never merge, not even with identical text. Each amount is
/// added to the first amount on the line it can be summed with (see [`Quantity::checked_add`]),
/// or appended when none can. An item without an amount still creates or joins its line.
#[must_use]
pub fn merge(items: impl IntoIterator<Item = (Item, Option<Quantity>)>) -> Vec<Line> {
    let mut lines: Vec<Line> = Vec::new();
    for (item, amount) in items {
        let same_line = |line: &&mut Line| matches!(item, Item::Ingredient(_)) && line.item == item;
        if let Some(line) = lines.iter_mut().find(same_line) {
            add_to(line, amount);
        } else {
            let mut line = Line {
                item,
                amounts: Vec::new(),
            };
            add_to(&mut line, amount);
            lines.push(line);
        }
    }
    lines
}

fn add_to(line: &mut Line, amount: Option<Quantity>) {
    if let Some(amount) = amount {
        add_amount(&mut line.amounts, amount);
    }
}

fn add_amount(amounts: &mut Vec<Quantity>, amount: Quantity) {
    for existing in amounts.iter_mut() {
        if let Some(sum) = existing.checked_add(amount) {
            *existing = sum;
            return;
        }
    }
    amounts.push(amount);
}
