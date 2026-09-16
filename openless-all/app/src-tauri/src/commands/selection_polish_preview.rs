use super::*;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectionPolishPreviewPayload {
    pub text: String,
    pub source_text: String,
}

fn map_selection_preview(
    snapshot: openless_core::SelectionSnapshot,
) -> Option<SelectionPolishPreviewPayload> {
    match (snapshot.preview_text, snapshot.source_text) {
        (Some(text), Some(source_text)) => {
            Some(SelectionPolishPreviewPayload { text, source_text })
        }
        _ => None,
    }
}

#[tauri::command]
pub async fn get_selection_polish_preview(
    core: CoreState<'_>,
) -> Result<Option<SelectionPolishPreviewPayload>, String> {
    let snapshot = core
        .services()
        .selection
        .snapshot()
        .await
        .map_err(|error| error.message)?;
    Ok(map_selection_preview(snapshot))
}

#[tauri::command]
pub async fn confirm_selection_polish_preview(
    core: CoreState<'_>,
    text: String,
) -> Result<(), String> {
    let snapshot = core
        .services()
        .selection
        .snapshot()
        .await
        .map_err(|error| error.message)?;
    let session_id = snapshot
        .session_id
        .ok_or_else(|| "selection preview is not active".to_string())?;
    // instrumentation: confirm 成功/失敗此前全無 log，無法區分「點擊被吞」與
    // 「到達後端但 apply 失敗」；先記錄入口與結果，定位後再撤。
    // 單位注意：chars()（字數），不是 bytes——繁中 1 字 3 bytes，混用會誤判截斷。
    let source_len = snapshot
        .source_text
        .as_deref()
        .map_or(0, |s| s.chars().count());
    log::info!(
        "[selection-polish] confirm: entry text_chars={} source_chars={}",
        text.chars().count(),
        source_len
    );
    let result = core
        .services()
        .selection
        .confirm(session_id, Some(text))
        .await
        .map_err(|error| error.message);
    match &result {
        Ok(()) => {
            log::info!("[selection-polish] confirm: ok")
        }
        Err(message) => {
            log::warn!("[selection-polish] confirm: err message={}", message)
        }
    }
    result
}

#[tauri::command]
pub async fn cancel_selection_polish_preview(core: CoreState<'_>) -> Result<(), String> {
    core.services()
        .selection
        .cancel(None)
        .await
        .map_err(|error| error.message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_preview_wire_uses_legacy_camel_case_fields() {
        let payload = map_selection_preview(openless_core::SelectionSnapshot {
            phase: openless_core::SelectionPhase::Preview,
            session_id: Some(openless_core::SessionId::new()),
            source_text: Some("source".to_string()),
            preview_text: Some("preview".to_string()),
            instruction: None,
            insert_outcome: None,
            revert_outcome: None,
        })
        .expect("preview snapshot should map to the legacy payload");

        assert_eq!(
            serde_json::to_value(payload).unwrap(),
            serde_json::json!({ "text": "preview", "sourceText": "source" })
        );
    }
}
