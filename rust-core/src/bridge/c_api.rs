use std::ffi::{c_char, c_int, c_void, CStr, CString};
use std::ptr;
use std::sync::Arc;

use crate::api::providers::{DeepSeekProvider, OpenRouterProvider, SiliconFlowProvider};
use crate::message::Message;
use crate::session::SessionManager;
use crate::session::{Session, SessionConfig, StreamCallback};

fn get_session_manager() -> &'static SessionManager {
    static SESSION_MANAGER: once_cell::sync::Lazy<SessionManager> =
        once_cell::sync::Lazy::new(SessionManager::new);
    &SESSION_MANAGER
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
        return ptr::null_mut();
    }

    let config_cstr = unsafe { CStr::from_ptr(config_str) };
    let config_json = match config_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let config: SessionConfig = match serde_json::from_str(config_json) {
        Ok(c) => c,
        Err(_) => return ptr::null_mut(),
    };

    match get_session_manager().create_session(config) {
        Ok(session) => Arc::into_raw(session) as *mut c_void,
        Err(_) => ptr::null_mut(),
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

    let session_arc = unsafe { Arc::from_raw(session_ptr as *const Session) };
    let response = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(crate::execute_query(
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

    {
        let mut messages = session.messages.write();
        messages.push(assistant_message);
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
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(crate::execute_streaming_query(
            &session_arc,
            &messages_clone,
            &mut send_callback,
        ));
    std::mem::forget(session_arc);

    match result {
        Ok(response) => {
            let assistant_message = Message::assistant(&response);
            let mut messages = session.messages.write();
            messages.push(assistant_message);
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

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            return CString::new(make_error_json(&format!("runtime error: {}", e)))
                .unwrap()
                .into_raw();
        }
    };

    let result = session.list_models();
    let response_str = match rt.block_on(result) {
        Ok(models) => serde_json::to_string(&models).unwrap_or_else(|_| "[]".to_string()),
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

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            return CString::new(make_error_json(&format!("runtime error: {}", e)))
                .unwrap()
                .into_raw();
        }
    };

    let result = session.get_balance();
    let response_str = match rt.block_on(result) {
        Ok(balance) => serde_json::to_string(&balance).unwrap_or_else(|_| "{}".to_string()),
        Err(e) => make_error_json(&format!("{}", e)),
    };

    CString::new(response_str).unwrap().into_raw()
}
