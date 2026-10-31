//! The servings a recipe page shows: the count the address asks for, and the amounts scaled to it.

use crabbage_domain::quantity::{Quantity, Unit};
use crabbage_domain::scaling::{Servings, scale};

use super::types::LineRow;

/// The servings the page shows: the `servings` value of the address when it is from 1 to 99, otherwise
/// the recipe's own count.
pub(crate) fn shown_servings(requested: Option<&str>, original: i64) -> i64 {
    requested
        .and_then(|text| text.parse::<i64>().ok())
        .filter(|count| (1..=99).contains(count))
        .unwrap_or(original)
}

/// The lines with their amounts scaled from `from` to `to` servings. A line without an amount, or
/// whose unit is not a measure such as `to taste`, keeps its text. An amount without a unit counts
/// in pieces.
pub(crate) fn scale_lines(lines: &[LineRow], from: i64, to: i64) -> Vec<LineRow> {
    let (Some(from), Some(to)) = (servings(from), servings(to)) else {
        return lines.to_vec();
    };
    lines
        .iter()
        .map(|line| scale_line(line, from, to))
        .collect()
}

fn servings(count: i64) -> Option<Servings> {
    u32::try_from(count).ok().and_then(Servings::new)
}

fn scale_line(line: &LineRow, from: Servings, to: Servings) -> LineRow {
    let unit = match line.unit.as_deref() {
        None | Some("") => Some(Unit::Piece),
        Some(symbol) => Unit::from_symbol(symbol),
    };
    let quantity = line
        .amount
        .zip(unit)
        .and_then(|(amount, unit)| Quantity::new(amount, unit));
    quantity.map_or_else(
        || line.clone(),
        |quantity| {
            let scaled = scale(quantity, from, to);
            LineRow {
                amount: Some(scaled.amount()),
                unit: Some(scaled.unit().symbol().to_owned()),
                ..line.clone()
            }
        },
    )
}
