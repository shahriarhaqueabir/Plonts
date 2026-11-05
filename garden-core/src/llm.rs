use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ModelKind {
    #[default]
    Text,
    Vision,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ContextFlags {
    pub garden: bool,
    pub zone: bool,
    pub tasks: bool,
}

impl Default for ContextFlags {
    fn default() -> Self {
        Self {
            garden: true,
            zone: true,
            tasks: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ChatSource {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ChatAttachment {
    pub id: String,
    pub message_id: String,
    pub path: String,
    pub mime: Option<String>,
    pub url: String,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: MessageRole,
    pub content: String,
    pub sources: Vec<ChatSource>,
    pub grounded: bool,
    pub flagged: bool,
    pub flag_note: Option<String>,
    pub attachments: Vec<ChatAttachment>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ChatConversation {
    pub id: String,
    pub title: String,
    pub context_flags: ContextFlags,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub path: String,
    pub size_bytes: i64,
    pub kind: ModelKind,
    pub created_at: String,
    pub last_used_at: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    #[default]
    Open,
    Done,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub body: Option<String>,
    pub status: TaskStatus,
    pub due_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RetrievedChunk {
    pub id: String,
    pub title: String,
    pub kind: String,
    pub excerpt: String,
    pub url: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ChatResponse {
    pub message: ChatMessage,
    pub sources: Vec<ChatSource>,
    pub grounded: bool,
    pub prompt_tok_per_sec: f64,
    pub compl_tok_per_sec: f64,
}

/// Split a free-form plan answer into task titles (one per bullet/numbered line).
/// Pure fn (testable). Strips bullet/number prefixes and skips blanks.
pub fn split_plan_into_tasks(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|l| l.trim_start_matches(['-', '*', '+']).trim())
        .filter(|l| !l.is_empty())
        .map(|l| {
            // strip leading "1." / "1)" numbering
            let bytes = l.as_bytes();
            let mut i = 0;
            while i < bytes.len()
                && (bytes[i].is_ascii_digit() || bytes[i] == b'.' || bytes[i] == b')' || bytes[i] == b' ')
            {
                i += 1;
            }
            l[i..].trim().to_string()
        })
        .filter(|l| !l.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_bullets_and_numbers() {
        let t = split_plan_into_tasks("- Sow tomatoes\n2. Water daily\n  prune weekly");
        assert_eq!(t, vec!["Sow tomatoes", "Water daily", "prune weekly"]);
    }

    #[test]
    fn empty_returns_empty() {
        assert!(split_plan_into_tasks("  \n ").is_empty());
    }

    #[test]
    fn defaults() {
        let c = ContextFlags::default();
        assert!(c.garden && c.zone && c.tasks);
        assert_eq!(ModelKind::default(), ModelKind::Text);
        assert_eq!(TaskStatus::default(), TaskStatus::Open);
    }
}