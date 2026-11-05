use std::path::Path;
use std::sync::Arc;

use mistralrs::{GgufModelBuilder, Model, RequestBuilder, TextMessageRole};

use crate::error::{AppError, AppResult};
use garden_core::llm::{ChatSource, MessageRole, ModelKind, RetrievedChunk};

/// A loaded model plus its registry metadata, kept in AppState.
pub struct LoadedModel {
    pub model: Arc<Model>,
    pub model_id: String,
    pub kind: ModelKind,
}

pub struct GeneratedText {
    pub content: String,
    pub prompt_tok_per_sec: f64,
    pub compl_tok_per_sec: f64,
}

pub async fn load_gguf(model_id: &str, gguf_path: &str, kind: ModelKind) -> AppResult<LoadedModel> {
    if !Path::new(gguf_path).is_file() {
        return Err(AppError::InvalidInput(format!("model file not found: {gguf_path}")));
    }
    let model = GgufModelBuilder::new(model_id.to_string(), vec![gguf_path.to_string()])
        .with_force_cpu()
        .build()
        .await
        .map_err(|e| AppError::Llm(e.to_string()))?;
    Ok(LoadedModel {
        model: Arc::new(model),
        model_id: model_id.to_string(),
        kind,
    })
}

pub async fn generate_text(
    model: &Model,
    system: &str,
    history: &[(MessageRole, String)],
    question: &str,
    max_tokens: usize,
) -> AppResult<GeneratedText> {
    let mut req = RequestBuilder::new()
        .add_message(TextMessageRole::System, system)
        .set_sampler_temperature(0.7)
        .set_sampler_max_len(max_tokens);
    for (role, content) in history {
        let r = match role {
            MessageRole::User => TextMessageRole::User,
            MessageRole::Assistant => TextMessageRole::Assistant,
        };
        req = req.add_message(r, content.clone());
    }
    req = req.add_message(TextMessageRole::User, question);

    let resp = model
        .send_chat_request(req)
        .await
        .map_err(|e| AppError::Llm(e.to_string()))?;
    let prompt_tok_per_sec = resp.usage.avg_prompt_tok_per_sec as f64;
    let compl_tok_per_sec = resp.usage.avg_compl_tok_per_sec as f64;
    let content = resp
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.content)
        .unwrap_or_default();
    Ok(GeneratedText {
        content,
        prompt_tok_per_sec,
        compl_tok_per_sec,
    })
}

pub async fn generate_with_image(
    model: &Model,
    system: &str,
    question: &str,
    image: image::DynamicImage,
    max_tokens: usize,
) -> AppResult<GeneratedText> {
    let req = RequestBuilder::new()
        .add_message(TextMessageRole::System, system)
        .set_sampler_temperature(0.7)
        .set_sampler_max_len(max_tokens)
        .add_image_message(TextMessageRole::User, question, vec![image]);
    let resp = model
        .send_chat_request(req)
        .await
        .map_err(|e| AppError::Llm(e.to_string()))?;
    let prompt_tok_per_sec = resp.usage.avg_prompt_tok_per_sec as f64;
    let compl_tok_per_sec = resp.usage.avg_compl_tok_per_sec as f64;
    let content = resp
        .choices
        .into_iter()
        .next()
        .and_then(|c| c.message.content)
        .unwrap_or_default();
    Ok(GeneratedText {
        content,
        prompt_tok_per_sec,
        compl_tok_per_sec,
    })
}

// ---- Pure context assembly (unit-tested, no model) ----

pub const MAX_RETRIEVED: usize = 6;
pub const MAX_ANSWER_TOKENS: usize = 512;

/// Build the system prompt from gathered context. Pure + deterministic.
pub fn build_system_prompt(
    garden: Option<&str>,
    zone: Option<&str>,
    tasks: Option<&str>,
    chunks: &[RetrievedChunk],
) -> String {
    let mut out = String::from(
        "You are the LovelyGarden assistant, a gardening coach. \
         Answer helpfully and concisely. When you use the provided library excerpts, \
         answer from them first; clearly separate what comes from the library versus your \
         own general knowledge. If the user asks for a plan, give a numbered step-by-step plan.",
    );
    if let Some(g) = garden {
        out.push_str("\n\n## My garden\n");
        out.push_str(g);
    }
    if let Some(z) = zone {
        out.push_str("\n\n## My location\n");
        out.push_str(z);
    }
    if let Some(t) = tasks {
        out.push_str("\n\n## Current sowing tasks\n");
        out.push_str(t);
    }
    if !chunks.is_empty() {
        out.push_str("\n\n## Library excerpts (from the bundled library, cite these)\n");
        for c in chunks {
            out.push_str(&format!("- [{}] {}: {}\n", c.kind, c.title, c.excerpt));
        }
    }
    out
}

/// Convert retrieved chunks into displayable source cards.
pub fn chunks_to_sources(chunks: &[RetrievedChunk]) -> Vec<ChatSource> {
    chunks
        .iter()
        .map(|c| ChatSource {
            id: c.id.clone(),
            title: c.title.clone(),
            kind: c.kind.clone(),
            url: c.url.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_system_prompt_includes_sections() {
        let chunks = vec![RetrievedChunk {
            id: "p1".into(),
            title: "Tomato".into(),
            kind: "plant".into(),
            excerpt: "Sow after frost.".into(),
            url: None,
        }];
        let s = build_system_prompt(Some("bed 1"), Some("north"), Some("water"), &chunks);
        assert!(s.contains("## My garden"));
        assert!(s.contains("bed 1"));
        assert!(s.contains("## My location"));
        assert!(s.contains("north"));
        assert!(s.contains("## Current sowing tasks"));
        assert!(s.contains("water"));
        assert!(s.contains("## Library excerpts"));
        assert!(s.contains("[plant] Tomato: Sow after frost."));
    }

    #[test]
    fn build_system_prompt_omits_empty() {
        let s = build_system_prompt(None, None, None, &[]);
        assert!(!s.contains("## My garden"));
        assert!(!s.contains("Library excerpts"));
    }

    #[test]
    fn chunks_to_sources_maps() {
        let chunks = vec![RetrievedChunk {
            id: "g1".into(),
            title: "Soil".into(),
            kind: "guide".into(),
            excerpt: "e".into(),
            url: Some("u".into()),
        }];
        let srcs = chunks_to_sources(&chunks);
        assert_eq!(srcs.len(), 1);
        assert_eq!(srcs[0].id, "g1");
        assert_eq!(srcs[0].url.as_deref(), Some("u"));
    }
}
