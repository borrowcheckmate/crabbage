use crabbage_store::recipes::{LineInput, RecipeInput, StepInput};

pub(crate) struct DemoRecipe {
    title_en: &'static str,
    title_nl: &'static str,
    ingredient_en: [&'static str; 3],
    ingredient_nl: [&'static str; 3],
}

impl DemoRecipe {
    pub(crate) const fn all() -> &'static [Self; 12] {
        &RECIPES
    }

    const fn new(
        title_en: &'static str,
        title_nl: &'static str,
        ingredient_en: [&'static str; 3],
        ingredient_nl: [&'static str; 3],
    ) -> Self {
        Self {
            title_en,
            title_nl,
            ingredient_en,
            ingredient_nl,
        }
    }

    pub(crate) fn input(&self, language: &str, category_id: Option<i64>) -> RecipeInput {
        let (title, ingredients) = if language == "nl" {
            (self.title_nl, &self.ingredient_nl)
        } else {
            (self.title_en, &self.ingredient_en)
        };
        let lines = ingredients
            .iter()
            .zip([250.0, 2.0, 1.0])
            .zip(["g", "piece", "piece"])
            .map(|((ingredient, amount), unit)| LineInput {
                amount: Some(amount),
                unit: Some(unit.to_owned()),
                ingredient: Some((*ingredient).to_owned()),
                note: None,
            })
            .collect();
        let steps = vec![
            StepInput {
                text: if language == "nl" {
                    "Bereid de ingrediënten voor en bak 5 minuten in een warme pan."
                } else {
                    "Prepare the ingredients and cook for 5 minutes in a warm pan."
                }
                .to_owned(),
                duration_seconds: Some(300),
                line_positions: vec![0, 1],
            },
            StepInput {
                text: if language == "nl" {
                    "Voeg alles samen, laat 10 minuten sudderen en serveer warm."
                } else {
                    "Combine everything, simmer for 10 minutes, and serve warm."
                }
                .to_owned(),
                duration_seconds: Some(600),
                line_positions: vec![0, 1, 2],
            },
        ];
        RecipeInput {
            title: title.to_owned(),
            servings: 4,
            lines,
            steps,
            category_ids: category_id.into_iter().collect(),
        }
    }
}

static RECIPES: [DemoRecipe; 12] = [
    DemoRecipe::new(
        "Lemon herb pasta",
        "Citroen-kruidenpasta",
        ["pasta", "lemon", "parsley"],
        ["pasta", "citroen", "peterselie"],
    ),
    DemoRecipe::new(
        "Tomato lentil soup",
        "Tomaten-linzensoep",
        ["lentils", "tomato", "onion"],
        ["linzen", "tomaat", "ui"],
    ),
    DemoRecipe::new(
        "Roasted vegetable couscous",
        "Couscous met geroosterde groenten",
        ["couscous", "courgette", "bell pepper"],
        ["couscous", "courgette", "paprika"],
    ),
    DemoRecipe::new(
        "Crispy potato frittata",
        "Krokante aardappelfrittata",
        ["potato", "egg", "spinach"],
        ["aardappel", "ei", "spinazie"],
    ),
    DemoRecipe::new(
        "Ginger soy salmon",
        "Zalm met gember en soja",
        ["salmon", "ginger", "rice"],
        ["zalm", "gember", "rijst"],
    ),
    DemoRecipe::new(
        "Creamy mushroom risotto",
        "Romige paddenstoelenrisotto",
        ["risotto rice", "mushroom", "Parmesan"],
        ["risottorijst", "champignon", "Parmezaanse kaas"],
    ),
    DemoRecipe::new(
        "Green garden salad",
        "Groene tuinsalade",
        ["lettuce", "cucumber", "tomato"],
        ["sla", "komkommer", "tomaat"],
    ),
    DemoRecipe::new(
        "Chickpea coconut curry",
        "Kikkererwtencurry met kokos",
        ["chickpeas", "coconut milk", "spinach"],
        ["kikkererwten", "kokosmelk", "spinazie"],
    ),
    DemoRecipe::new(
        "Apple cinnamon pancakes",
        "Appel-kaneelpannenkoeken",
        ["flour", "milk", "apple"],
        ["bloem", "melk", "appel"],
    ),
    DemoRecipe::new(
        "Baked chicken tray",
        "Ovenschotel met kip",
        ["chicken breast", "potato", "carrot"],
        ["kipfilet", "aardappel", "wortel"],
    ),
    DemoRecipe::new(
        "Quick sesame noodles",
        "Snelle noedels met sesam",
        ["egg noodles", "carrot", "soy sauce"],
        ["mie", "wortel", "sojasaus"],
    ),
    DemoRecipe::new(
        "Berry yogurt crumble",
        "Kruimeldessert met bessen en yoghurt",
        ["oats", "apple", "Greek yoghurt"],
        ["havermout", "appel", "Griekse yoghurt"],
    ),
];
