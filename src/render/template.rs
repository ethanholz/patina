use serde_json::{Value, json};

pub fn basic_template() -> Result<String, anyhow::Error> {
    let user_template = r#"<div class="title_bar">
        <span class="title">{{ message }}</span>
        </div>"#;
    let user_data = json!({
        "message": "hello world"
    });
    render_user_template_embedded(user_template, user_data)
}

/// Renders a user template with the provided data.
pub fn render_user_template(
    user_template: &str,
    user_data: Value,
) -> Result<String, anyhow::Error> {
    let parser = liquid::ParserBuilder::with_stdlib().build()?;
    let template = parser.parse(user_template)?;
    let obj = liquid::to_object(&user_data)?;

    let out = template.render(&obj)?;

    Ok(out)
}

pub fn render_user_template_embedded(
    user_template: &str,
    user_data: Value,
) -> Result<String, anyhow::Error> {
    let user_defined = render_user_template(user_template, user_data)?;

    let parser = liquid::ParserBuilder::with_stdlib().build()?;
    let base = parser.parse_file("./templates/base.liquid")?;
    let data = json!({
        "embed": user_defined
    });
    let obj = liquid::to_object(&data)?;

    let out = base.render(&obj)?;

    Ok(out)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_render_user_template() {
        let user_template = r#"<pi>{{ message }}</p>"#;

        let user_data = json!({
            "message": "hello world"
        });

        let rendered = render_user_template(user_template, user_data).unwrap();
        assert_eq!(rendered, "<pi>hello world</p>");
    }
}
