use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

const SCHEMA: &str = "Answer with one JSON object and nothing else. The object has: \
\"title\" (text); \"servings\", \"prep_minutes\" and \"cook_minutes\" (whole numbers or null); \
\"lines\" (the ingredient lines in order, each with \"amount\" (a number or null), \"unit\" (one of \
g, kg, ml, l, tsp, tbsp, piece, pinch, \"to taste\", or null), \"name\" (text), \"note\" (text or \
null) and \"ingredient_id\" (a number or null)); \"steps\" (in order, each with \"text\" and \"lines\", \
the zero-based indexes of the ingredient lines the step uses).";

/// What the model reads: the instructions, the recipe text and its photos.
#[derive(Debug)]
pub struct Prompt {
    /// What the model must do.
    pub instructions: String,
    /// The recipe as text; left out of the request when empty.
    pub text: String,
    /// The recipe's photos, in order.
    pub images: Vec<Image>,
}

/// A photo sent to the model.
#[derive(Debug)]
pub struct Image {
    /// The media type, such as `image/png`.
    pub media_type: String,
    /// The image file's bytes.
    pub bytes: Vec<u8>,
}

pub(crate) fn body(model: &str, prompt: &Prompt) -> Value {
    let mut content = Vec::new();
    if !prompt.text.is_empty() {
        content.push(json!({"type": "text", "text": prompt.text}));
    }
    content.extend(prompt.images.iter().map(image_part));
    json!({
        "model": model,
        "response_format": {"type": "json_object"},
        "messages": [
            {"role": "system", "content": format!("{}\n\n{SCHEMA}", prompt.instructions)},
            {"role": "user", "content": content},
        ],
    })
}

fn image_part(image: &Image) -> Value {
    let encoded = STANDARD.encode(&image.bytes);
    let url = format!("data:{};base64,{encoded}", image.media_type);
    json!({"type": "image_url", "image_url": {"url": url}})
}
