use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::ptr;
use std::sync::Arc;

use crate::api::providers::{DeepSeekProvider, OpenRouterProvider, SiliconFlowProvider};
use crate::db;
use crate::engine::conversation::ConversationEngine;
use crate::message::Message;
use crate::session::{PersistentSessionManager, SessionManager};
use crate::session::{Session, SessionConfig, StreamCallback};

fn get_session_manager() -> &'static SessionManager {
    static SESSION_MANAGER: once_cell::sync::Lazy<SessionManager> =
        once_cell::sync::Lazy::new(SessionManager::new);
    &SESSION_MANAGER
}

/// Persistent session manager backed by file storage.
/// Sessions survive app restarts and are automatically loaded.
fn get_persistent_session_manager() -> &'static PersistentSessionManager {
    static PERSISTENT_MANAGER: once_cell::sync::Lazy<PersistentSessionManager> =
        once_cell::sync::Lazy::new(|| {
            // On Android, use the app's internal storage directory
            let storage_dir = if cfg!(target_os = "android") {
                // Android: use /data/data/<package_name>/files/claude-core/sessions
                // This directory is accessible without root
                std::path::PathBuf::from("/data/data/com.example.flutter_app/files/claude-core/sessions")
            } else {
                dirs::config_dir()
                    .map(|d| d.join("claude-core").join("sessions"))
                    .unwrap_or_else(|| std::path::PathBuf::from("./sessions"))
            };
            
            eprintln!("🔵 [Rust] PersistentSessionManager storage_dir: {:?}", storage_dir);
            
            PersistentSessionManager::new(storage_dir.to_str().unwrap_or("./sessions"))
        });
    &PERSISTENT_MANAGER
}

/// Shared tokio runtime for all async C API calls.
/// Creating a new Runtime per call wastes thread pools and can exhaust resources.
fn get_runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: once_cell::sync::Lazy<tokio::runtime::Runtime> =
        once_cell::sync::Lazy::new(|| {
            tokio::runtime::Runtime::new().expect("Failed to create tokio runtime")
        });
    &RUNTIME
}

#[no_mangle]
pub extern "C" fn init_database(db_path: *const c_char) -> c_int {
    if db_path.is_null() {
        return -1;
    }

    let db_path_cstr = unsafe { CStr::from_ptr(db_path) };
    let db_path_str = match db_path_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    match db::init_db(db_path_str) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

fn make_error_json(msg: &str) -> String {
    format!(
        "{{\"error\":\"{}\"}}",
        msg.replace('\\', "\\\\").replace('"', "\\\"")
    )
}

#[no_mangle]
pub extern "C" fn create_session(config_str: *const c_char) -> *mut c_void {
    if config_str.is_null() {
        eprintln!("❌ [Rust] create_session: config_str is null");
        return ptr::null_mut();
    }

    let config_cstr = unsafe { CStr::from_ptr(config_str) };
    let config_json = match config_cstr.to_str() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("❌ [Rust] create_session: config_cstr.to_str() failed: {}", e);
            return ptr::null_mut();
        }
    };

    let config: SessionConfig = match serde_json::from_str(config_json) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("❌ [Rust] create_session: serde_json::from_str failed: {}", e);
            return ptr::null_mut();
        }
    };

    // Use persistent session manager so sessions survive app restarts
    match get_persistent_session_manager().create_session(config.clone()) {
        Ok(session) => {
            // Also register in the in-memory manager for backward compatibility
            let _ = get_session_manager().create_session(config);
            eprintln!("✅ [Rust] create_session: success");
            Arc::into_raw(session) as *mut c_void
        }
        Err(e) => {
            eprintln!("❌ [Rust] create_session: create_session failed: {}", e);
            ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn send_message(session_ptr: *mut c_void, content: *const c_char) -> *mut c_char {
    if session_ptr.is_null() || content.is_null() {
        return CString::new(make_error_json("null pointer"))
            .unwrap()
            .into_raw();
    }

    let session = unsafe { &*(session_ptr as *const Session) };
    let content_cstr = unsafe { CStr::from_ptr(content) };
    let content_str = match content_cstr.to_str() {
        Ok(s) => s,
        Err(_) => {
            return CString::new(make_error_json("invalid utf8"))
                .unwrap()
                .into_raw()
        }
    };

    let user_message = Message::user(content_str);

    {
        let mut messages = session.messages.write();
        messages.push(user_message);
    }

    // Persist user message to DB
    if let Ok(conversation_engine) = std::panic::catch_unwind(|| ConversationEngine::new()) {
        let session_arc = unsafe { Arc::from_raw(session_ptr as *const Session) };
        let _ = get_runtime().block_on(async {
            let _ = conversation_engine.submit_message(&session_arc, content_str, None).await;
        });
        std::mem::forget(session_arc);
    }

    let session_arc = unsafe { Arc::from_raw(session_ptr as *const Session) };
    let response = get_runtime().block_on(crate::execute_query(
        &session_arc,
        &session_arc.messages.read(),
    ));
    std::mem::forget(session_arc);

    let response_str = match response {
        Ok(resp) => resp,
        Err(e) => serde_json::to_string(&e).unwrap_or_else(|_| make_error_json("unknown error")),
    };

    let assistant_message =
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&response_str) {
            let content = val
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or(&response_str)
                .to_string();
            let thinking = val
                .get("thinking")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let mut msg = Message::assistant(content);
            msg.thinking = thinking;
            msg
        } else {
            Message::assistant(&response_str)
        };

    let assistant_content = assistant_message.content.clone();
    let assistant_thinking = assistant_message.thinking.clone();

    {
        let mut messages = session.messages.write();
        messages.push(assistant_message);
    }

    // Persist assistant message to DB
    if let Ok(conversation_engine) = std::panic::catch_unwind(|| ConversationEngine::new()) {
        let session_arc = unsafe { Arc::from_raw(session_ptr as *const Session) };
        let _ = get_runtime().block_on(async {
            let _ = conversation_engine.process_assistant_response(
                &session_arc,
                &assistant_content,
                assistant_thinking.as_deref(),
            ).await;
        });
        std::mem::forget(session_arc);
    }

    match CString::new(response_str) {
        Ok(s) => s.into_raw(),
        Err(_) => CString::new(make_error_json("null byte in response"))
            .unwrap()
            .into_raw(),
    }
}

#[no_mangle]
pub extern "C" fn stream_message(
    session_ptr: *mut c_void,
    content: *const c_char,
    callback: StreamCallback,
    user_data: *mut c_void,
) -> c_int {
    if session_ptr.is_null() || content.is_null() {
        return -1;
    }

    let session = unsafe { &*(session_ptr as *const Session) };
    let content_cstr = unsafe { CStr::from_ptr(content) };
    let content_str = match content_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let user_message = Message::user(content_str);

    {
        let mut messages = session.messages.write();
        messages.push(user_message);
    }

    // Persist user message to DB
    if let Ok(conversation_engine) = std::panic::catch_unwind(|| ConversationEngine::new()) {
        let session_arc = unsafe { Arc::from_raw(session_ptr as *const Session) };
        let _ = get_runtime().block_on(async {
            let _ = conversation_engine.submit_message(&session_arc, content_str, None).await;
        });
        std::mem::forget(session_arc);
    }

    struct CallbackWrapper {
        callback: StreamCallback,
        user_data: usize,
    }

    unsafe impl Send for CallbackWrapper {}
    unsafe impl Sync for CallbackWrapper {}

    let callback_wrapper = CallbackWrapper {
        callback,
        user_data: user_data as usize,
    };

    let mut send_callback = move |chunk: String| {
        let c_chunk = CString::new(chunk).unwrap();
        (callback_wrapper.callback)(c_chunk.as_ptr(), callback_wrapper.user_data as *mut c_void);
    };

    let messages_clone: Vec<Message> = {
        let messages = session.messages.read();
        messages.clone()
    };

    let session_arc = unsafe { Arc::from_raw(session_ptr as *const Session) };
    let result = get_runtime().block_on(crate::execute_streaming_query(
        &session_arc,
        &messages_clone,
        &mut send_callback,
    ));
    std::mem::forget(session_arc);

    match result {
        Ok(response) => {
            let assistant_message = Message::assistant(&response);
            let assistant_content = assistant_message.content.clone();
            let assistant_thinking = assistant_message.thinking.clone();
            {
                let mut messages = session.messages.write();
                messages.push(assistant_message);
            }
            // Persist assistant message to DB
            if let Ok(conversation_engine) = std::panic::catch_unwind(|| ConversationEngine::new()) {
                let session_arc = unsafe { Arc::from_raw(session_ptr as *const Session) };
                let _ = get_runtime().block_on(async {
                    let _ = conversation_engine.process_assistant_response(
                        &session_arc,
                        &assistant_content,
                        assistant_thinking.as_deref(),
                    ).await;
                });
                std::mem::forget(session_arc);
            }
            0
        }
        Err(_) => -1,
    }
}

#[no_mangle]
pub extern "C" fn destroy_session(session_ptr: *mut c_void) {
    if !session_ptr.is_null() {
        let session = unsafe { Arc::from_raw(session_ptr as *const Session) };
        let id = session.id.clone();
        drop(session);
        get_session_manager().remove_session(&id);
        // Also remove from persistent storage
        let _ = get_persistent_session_manager().remove_session(&id);
    }
}

#[no_mangle]
pub extern "C" fn get_messages(session_ptr: *mut c_void) -> *mut c_char {
    if session_ptr.is_null() {
        return CString::new(make_error_json("null pointer"))
            .unwrap()
            .into_raw();
    }

    let session = unsafe { &*(session_ptr as *const Session) };
    let messages = session.messages.read();

    let message_list = crate::message::MessageList {
        messages: messages.clone(),
    };

    let json =
        serde_json::to_string(&message_list).unwrap_or_else(|_| r#"{"messages":[]}"#.to_string());

    match CString::new(json) {
        Ok(s) => s.into_raw(),
        Err(_) => CString::new(make_error_json("null byte in messages"))
            .unwrap()
            .into_raw(),
    }
}

#[no_mangle]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)) };
    }
}

#[no_mangle]
pub extern "C" fn set_provider(
    session_ptr: *mut c_void,
    provider_name: *const c_char,
    api_key: *const c_char,
) -> bool {
    if session_ptr.is_null() || provider_name.is_null() || api_key.is_null() {
        return false;
    }

    let session = unsafe { &*(session_ptr as *const Session) };
    let provider_cstr = unsafe { CStr::from_ptr(provider_name) };
    let api_key_cstr = unsafe { CStr::from_ptr(api_key) };

    let provider_name_str = match provider_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let api_key_str = match api_key_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return false,
    };

    let provider: Arc<dyn crate::api::providers::Provider> = match provider_name_str {
        "openrouter" => Arc::new(OpenRouterProvider::new(api_key_str, None)),
        "deepseek" => Arc::new(DeepSeekProvider::new(api_key_str, None)),
        "siliconflow" => Arc::new(SiliconFlowProvider::new(api_key_str, None)),
        _ => return false,
    };

    let mut session_provider = session.provider.write();
    *session_provider = Some(provider);
    true
}

#[no_mangle]
pub extern "C" fn list_models(session_ptr: *mut c_void) -> *mut c_char {
    if session_ptr.is_null() {
        return CString::new(make_error_json("null session pointer"))
            .unwrap()
            .into_raw();
    }

    let session = unsafe { &*(session_ptr as *const Session) };

    let result = session.list_models();
    let response_str = match get_runtime().block_on(result) {
        Ok(models) => serde_json::to_string(&models).unwrap_or_else(|_| "[]".to_string()),
        Err(e) => make_error_json(&format!("{}", e)),
    };

    CString::new(response_str).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn set_api_key(provider: *const c_char, api_key: *const c_char) -> c_int {
    if provider.is_null() || api_key.is_null() {
        eprintln!("[set_api_key] null pointer");
        return -1;
    }

    let provider_cstr = unsafe { CStr::from_ptr(provider) };
    let api_key_cstr = unsafe { CStr::from_ptr(api_key) };

    let provider_str = match provider_cstr.to_str() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[set_api_key] invalid provider string: {}", e);
            return -1;
        }
    };

    let api_key_str = match api_key_cstr.to_str() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[set_api_key] invalid api_key string: {}", e);
            return -1;
        }
    };

    let rt = get_runtime();
    let config_manager = crate::session::ConfigManager::new(None);
    let result = rt.block_on(config_manager.set_api_key(provider_str, api_key_str));
    
    match result {
        Ok(_) => {
            eprintln!("[set_api_key] saved key for provider: {}", provider_str);
            0
        }
        Err(e) => {
            eprintln!("[set_api_key] failed to save key for {}: {}", provider_str, e);
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn get_api_key(provider: *const c_char) -> *mut c_char {
    if provider.is_null() {
        return CString::new("").unwrap().into_raw();
    }

    let provider_cstr = unsafe { CStr::from_ptr(provider) };
    let provider_str = match provider_cstr.to_str() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[get_api_key] invalid provider string: {}", e);
            return CString::new("").unwrap().into_raw();
        }
    };

    let rt = get_runtime();

    let config_manager = crate::session::ConfigManager::new(None);
    let result = rt.block_on(config_manager.get_api_key(provider_str));

    match result {
        Ok(Some(key)) => {
            eprintln!("[get_api_key] found key for provider: {}", provider_str);
            CString::new(key).unwrap().into_raw()
        }
        Ok(None) => {
            eprintln!("[get_api_key] no key found for provider: {}", provider_str);
            CString::new("").unwrap().into_raw()
        }
        Err(e) => {
            eprintln!(
                "[get_api_key] error reading key for {}: {}",
                provider_str, e
            );
            CString::new("").unwrap().into_raw()
        }
    }
}

#[no_mangle]
pub extern "C" fn compact_session(
    session_ptr: *mut c_void,
    summary: *const c_char,
    boundary_msg_id: *const c_char,
) -> c_int {
    if session_ptr.is_null() || summary.is_null() || boundary_msg_id.is_null() {
        return -1;
    }

    let _session = unsafe { &*(session_ptr as *const Session) };
    let summary_cstr = unsafe { CStr::from_ptr(summary) };
    let boundary_cstr = unsafe { CStr::from_ptr(boundary_msg_id) };

    let summary_str = match summary_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let boundary_str = match boundary_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let session_arc = unsafe { Arc::from_raw(session_ptr as *const Session) };

    let context_manager = crate::engine::ContextManager::new();
    let result = get_runtime().block_on(context_manager.manual_compact(&session_arc, summary_str, boundary_str));

    std::mem::forget(session_arc);

    if result.is_ok() {
        0
    } else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn get_conversation_history(session_ptr: *mut c_void) -> *mut c_char {
    if session_ptr.is_null() {
        return CString::new(make_error_json("null pointer"))
            .unwrap()
            .into_raw();
    }

    let session = unsafe { &*(session_ptr as *const Session) };
    let session_id = session.id.clone();

    let conversation_engine = crate::engine::ConversationEngine::new();
    let result = get_runtime().block_on(conversation_engine.get_conversation_history(&session_id));

    let response_str = match result {
        Ok(messages) => {
            let message_list = crate::message::MessageList { messages };
            serde_json::to_string(&message_list)
                .unwrap_or_else(|_| r#"{"messages":[]}"#.to_string())
        }
        Err(e) => make_error_json(&format!("{}", e)),
    };

    CString::new(response_str).unwrap().into_raw()
}

#[no_mangle]
pub extern "C" fn get_balance(session_ptr: *mut c_void) -> *mut c_char {
    if session_ptr.is_null() {
        return CString::new(make_error_json("null session pointer"))
            .unwrap()
            .into_raw();
    }

    let session = unsafe { &*(session_ptr as *const Session) };

    let result = session.get_balance();
    let response_str = match get_runtime().block_on(result) {
        Ok(balance) => serde_json::to_string(&balance).unwrap_or_else(|_| "{}".to_string()),
        Err(e) => make_error_json(&format!("{}", e)),
    };

    CString::new(response_str).unwrap().into_raw()
}
