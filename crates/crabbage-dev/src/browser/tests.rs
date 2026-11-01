use super::run::ROUTES;

#[test]
fn route_table_covers_every_application_route() {
    let router = include_str!("../../../crabbage/src/app.rs");
    for (_, route) in ROUTES {
        if *route == "/browser-test-not-found" {
            assert!(router.contains("fallback=NotFound"));
            continue;
        }
        let path = if route.chars().any(char::is_numeric) {
            route
                .split('/')
                .map(|part| {
                    if !part.is_empty() && part.chars().all(char::is_numeric) {
                        ":id"
                    } else {
                        part
                    }
                })
                .collect::<Vec<_>>()
                .join("/")
        } else {
            (*route).to_owned()
        };
        assert!(
            router.contains(&format!("path!(\"{path}\")")),
            "missing route {path}"
        );
    }
    assert_eq!(ROUTES.len(), 18);
}

#[test]
fn contrast_ratio_uses_wcag_relative_luminance() {
    assert!(contrast_ratio([0, 0, 0], [255, 255, 255]) > 20.9);
    assert!((contrast_ratio([255, 255, 255], [255, 255, 255]) - 1.0).abs() < f64::EPSILON);
}

fn contrast_ratio(foreground: [u8; 3], background: [u8; 3]) -> f64 {
    let luminance = |rgb: [u8; 3]| {
        let values = rgb.map(|channel| f64::from(channel) / 255.0);
        let linear = values.map(|value| {
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        });
        linear[0].mul_add(0.2126, linear[1].mul_add(0.7152, 0.0722 * linear[2]))
    };
    let first = luminance(foreground);
    let second = luminance(background);
    (first.max(second) + 0.05) / (first.min(second) + 0.05)
}
