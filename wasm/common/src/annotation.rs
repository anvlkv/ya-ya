use super::error::YaYaError;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
pub struct Annotation {
    pub annotation: String,
    pub id: usize,
}

pub async fn annotate_word(
    word: String,
    context: String,
    previous: Option<String>,
) -> Result<Annotation, YaYaError> {
    let client = reqwest::Client::new();

    let body = json::object! {
        word: word,
        context: context,
        previous: previous
    };

    let res = client
        .post(
            format!(
                "{}/translate-word",
                crate::env::EXTENSION_PUBLIC_TRANSLATE_URL
            )
            .as_str(),
        )
        .body(json::stringify(body))
        .send()
        .await?;

    // TODO: why doesnt it work with `res.json()`... ?
    let text = res.text().await?;

    Ok(serde_json::from_str(text.as_str())?)
}

pub async fn annotate_text(
    text: String,
    origin: String,
    previous: Option<String>,
) -> Result<Annotation, YaYaError> {
    let client = reqwest::Client::new();

    let body = json::object! {
        text: text,
        origin: origin,
        previous: previous
    };

    let res = client
        .post(
            format!(
                "{}/translate-text",
                crate::env::EXTENSION_PUBLIC_TRANSLATE_URL
            )
            .as_str(),
        )
        .body(json::stringify(body))
        .send()
        .await?;

    // TODO: why doesnt it work with `res.json()`... ?
    let text = res.text().await?;

    Ok(serde_json::from_str(text.as_str())?)
}

pub async fn success_record(id: usize, result: bool) -> Result<(), YaYaError> {
    let client = reqwest::Client::new();

    let body = json::object! {
        id: id,
        result: result
    };

    client
        .post(
            format!(
                "{}/success-record",
                crate::env::EXTENSION_PUBLIC_TRANSLATE_URL
            )
            .as_str(),
        )
        .body(json::stringify(body))
        .send()
        .await?;

    Ok(())
}
