use crate::llm::{
    provider::{
        anthropic::AnthropicService,
        apple::AppleService,
        config::{AnthropicConfig, CustomConfig, GoogleConfig, LMStudioConfig, OpenAIConfig},
        google::GoogleService,
        lmstudio::LMStudioService,
        openai::OpenAiService,
        openai_compatible::OpenAiCompatibleService,
    },
    LlmModel, LlmServiceManager,
};
use std::sync::Arc;
use std::time::Duration;
use tauri::State;
use tokio::sync::RwLock;

#[tauri::command]
pub async fn add_llm_service(
    state: State<'_, Arc<RwLock<LlmServiceManager>>>,
    provider_name: String,
    provider_conf: serde_json::Value,
) -> Result<(), String> {
    let mut manager = state.write().await;
    let timeout = Duration::from_secs(60); // Default timeout

    let model_id = provider_conf
        .get("model")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    match provider_name.as_str() {
        "apple" => {
            // Apple service might not check config? Or assume default?
            // The prompt says "provider_name에 맞는 llm provider service를 ... 해당 provider service를 생성"
            // AppleService::new() takes no args in current impl.
            // But we should check if conf is valid or needed? user said "provider_conf는 serde_json::Value"
            // We can try to deserialize to empty or ignore.
            let service = AppleService::new();
            manager.add_model(
                "apple".to_string(), // Or use model name from somewhere?
                LlmModel {
                    model: "Apple Foundation Model".to_string(), // Apple service has hardcoded model usually? or queries it.
                    service: Box::new(service),
                },
            );
        }
        "anthropic" => {
            let model_id = model_id.ok_or_else(|| "Missing model in config".to_string())?;
            let config: AnthropicConfig = serde_json::from_value(provider_conf)
                .map_err(|e| format!("Invalid config for Anthropic: {}", e))?;
            let service = AnthropicService::new(config.api_key.clone(), timeout);
            manager.add_model(
                model_id.clone(),
                LlmModel {
                    model: model_id,
                    service: Box::new(service),
                },
            );
        }
        "google" => {
            let model_id = model_id.ok_or_else(|| "Missing model in config".to_string())?;
            let config: GoogleConfig = serde_json::from_value(provider_conf)
                .map_err(|e| format!("Invalid config for Google: {}", e))?;
            let service = GoogleService::new(config.api_key.clone(), timeout);
            manager.add_model(
                model_id.clone(),
                LlmModel {
                    model: model_id,
                    service: Box::new(service),
                },
            );
        }
        "lmstudio" => {
            let model_id = model_id.ok_or_else(|| "Missing model in config".to_string())?;
            let config: LMStudioConfig = serde_json::from_value(provider_conf)
                .map_err(|e| format!("Invalid config for LMStudio: {}", e))?;
            let service = LMStudioService::new(config.base_url, config.api_key, timeout);
            manager.add_model(
                model_id.clone(),
                LlmModel {
                    model: model_id,
                    service: Box::new(service),
                },
            );
        }
        "openai" => {
            let model_id = model_id.ok_or_else(|| "Missing model in config".to_string())?;
            let config: OpenAIConfig = serde_json::from_value(provider_conf)
                .map_err(|e| format!("Invalid config for OpenAI: {}", e))?;
            let service = OpenAiService::new(config.api_key.clone(), timeout);
            manager.add_model(
                model_id.clone(),
                LlmModel {
                    model: model_id,
                    service: Box::new(service),
                },
            );
        }
        "custom" => {
            let model_id = model_id.ok_or_else(|| "Missing model in config".to_string())?;
            let config: CustomConfig = serde_json::from_value(provider_conf)
                .map_err(|e| format!("Invalid config for Custom: {}", e))?;
            let service = OpenAiCompatibleService::new(config.base_url, config.api_key, timeout);
            manager.add_model(
                config.name.clone(),
                LlmModel {
                    model: model_id,
                    service: Box::new(service),
                },
            );
            // Note: User might want 'name' for the key in manager?
            // "name" in CustomConfig might be the display name or ID in UI.
            // Manager uses HashMap<String, LlmModel>. The key is usually the model ID the user selects.
            // Let's use config.name if available as ID?
            // CustomConfig has `pub name: String`.
            // Wait, `add_model` takes `name: String`.
            // Ideally we should use `config.name` as the key in the manager, or `config.model`?
            // If I have multiple custom services, I might name them "Local LLM 1", "My Server".
            // Let's use `config.name` for the manager key (UI selection), and `config.model` for the API request.
            // But for other providers like "openai", we used `config.model` as key.
            // If user adds multiple openai models? user might configure "gpt-4" and "gpt-3.5".
            // The `provider_conf` likely comes from a form where user selects specific model.
            // So `config.model` is appropriate as ID usually.
            // EXCEPT for custom where `name` might be "My Local Server".
            // Re-reading `CustomConfig`: `pub name: String`, `pub model: String`.
            // `name` is likely the display label.
            // Let's check `LlmServiceManager::add_model`.
            // `pub fn add_model(&mut self, name: String, service: LlmModel)`
            // I'll stick to using `config.model` as the key for standard providers, and `config.name` or `config.model` for custom?
            // If I look at the `invoke_chat_command`, it takes `model: String` and calls `chat_stream(&model, ...)`.
            // So the `model` param passed from frontend to `invoke_chat_command` MUST match the key in `LlmServiceManager`.
            // So the key should be what the UI sends as `model`.
            // In UI, for OpenAI, user selects "gpt-4". So key="gpt-4".
            // For Custom, user defines a name "MyLocal". And model "llama-2".
            // If UI constructs a dropdown of *available* services, it might iterate keys of `LlmServiceManager`.
            // So for Custom, the key should probably be `config.name`?
            // But wait, `invoke_chat_command` passes `model` string.
            // If I have `key="MyCustom"`, `model="llama-7b"`.
            // `invoke_chat_command` receives `model="MyCustom"`.
            // It calls `manager.chat_stream("MyCustom", ...)`.
            // `manager` gets `LlmModel` for "MyCustom".
            // `LlmModel.model` is "llama-7b".
            // `LlmModel.service.chat_stream("llama-7b", ...)`.
            // This looks correct! So for Custom, key should be `config.name`.
            // For others, key should be `config.model`.
        }
        // Ollama case? user didn't mention ollama in the list: "apple, anthropic, google, lmstudio, openai, custom".
        // But `config.rs` has `OllamaConfig`.
        // The user request said: "provider_name은 ... apple, anthropic, google, lmstudio, openai, custom 이어야 해."
        // It explicitly lists allowed strings. Ollama is NOT in the allowed list of `provider_name` strings, but logic might be similar?
        // I will follow the accepted list strictly.
        _ => return Err(format!("Unknown provider: {}", provider_name)),
    }

    Ok(())
}
