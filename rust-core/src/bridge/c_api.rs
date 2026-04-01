use std::ffi::{c_char, c_void, CStr};
use std::ptr;
use std::sync::Arc;

use crate::session::{Session, SessionConfig};
use crate::message::Message;
use crate::api::providers::{OpenRouterProvider, DeepSeekProvider, SiliconFlowProvider};
use crate::session::SessionManager;

fn get_session_manager() -> &'static SessionManager {
    static SESSION_MANAGER: once_cell::sync::Lazy<SessionManager> =
        once_cell::sync::Lazy::new(SessionManager::new);
    &SESSION_MANAGER
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
pub extern "C" fn send_message(
    session_ptr: *mut c_void,
    content: *const c_char,
) -> *mut c_char {
    if session_ptr.is_null() || content.is_null() {
        return ptr::null_mut();
    }

    let session = unsafe { Arc::from_raw(session_ptr as *mut Session) };
    let content_cstr = unsafe { CStr::from_ptr(content) };
    let content_str = match content_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let message = Message::user(content_str.to_string());
    
    match session.add_message(message) {
        Ok(_) => {
            let response = r#"{"status":"success"}"#;
            let c_str = std::ffi::CString::new(response).unwrap();
            c_str.into_raw() as *mut c_char
        }
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn stream_message(
    session_ptr: *mut c_void,
    content: *const c_char,
    _callback: Option<unsafe extern "C" fn(*const c_char, *mut c_void)>,
    _user_data: *mut c_void,
) -> *mut c_char {
    if session_ptr.is_null() || content.is_null() {
        return ptr::null_mut();
    }

    let session = unsafe { Arc::from_raw(session_ptr as *mut Session) };
    let content_cstr = unsafe { CStr::from_ptr(content) };
    let content_str = match content_cstr.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let message = Message::user(content_str.to_string());
    
    match session.add_message(message) {
        Ok(_) => {
            let response = r#"{"status":"streaming"}"#;
            let c_str = std::ffi::CString::new(response).unwrap();
            c_str.into_raw() as *mut c_char
        }
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn destroy_session(session_ptr: *mut c_void) {
    if !session_ptr.is_null() {
        unsafe {
            let _ = Arc::from_raw(session_ptr as *mut Session);
        }
    }
}

#[no_mangle]
pub extern "C" fn get_messages(session_ptr: *mut c_void) -> *mut c_char {
    if session_ptr.is_null() {
        return ptr::null_mut();
    }

    let session = unsafe { &*(session_ptr as *mut Session) };
    let messages = session.messages.read();
    
    match serde_json::to_string(&*messages) {
        Ok(json) => {
            let c_str = std::ffi::CString::new(json).unwrap();
            c_str.into_raw() as *mut c_char
        }
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = std::ffi::CString::from_raw(ptr);
        }
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

    let session = unsafe { &*(session_ptr as *mut Session) };
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
        return ptr::null_mut();
    }

    let session = unsafe { &*(session_ptr as *mut Session) };
    let provider_guard = session.provider.read();
    
    let provider = match &*provider_guard {
        Some(p) => p,
        None => return ptr::null_mut(),
    };
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    match rt.block_on(provider.list_models()) {
        Ok(models) => {
            let json = serde_json::to_string(&models).unwrap();
            let c_str = std::ffi::CString::new(json).unwrap();
            c_str.into_raw() as *mut c_char
        }
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn get_balance(session_ptr: *mut c_void) -> *mut c_char {
    if session_ptr.is_null() {
        return ptr::null_mut();
    }

    let session = unsafe { &*(session_ptr as *mut Session) };
    let provider_guard = session.provider.read();
    
    let provider = match &*provider_guard {
        Some(p) => p,
        None => return ptr::null_mut(),
    };
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    match rt.block_on(provider.get_balance()) {
        Ok(balance) => {
            let json = serde_json::to_string(&balance).unwrap();
            let c_str = std::ffi::CString::new(json).unwrap();
            c_str.into_raw() as *mut c_char
        }
        Err(_) => ptr::null_mut(),
    }
}
