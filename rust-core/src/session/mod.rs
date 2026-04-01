mod error;
mod config;
mod storage;

pub use error::*;
pub use config::*;
pub use storage::{SessionStorage, PersistentSessionManager};

use std::ffi::{c_char, c_int, c_void};
use std::ptr;
use std::sync::Arc;
use parking_lot::RwLock;

use crate::message::{Message, MessageList};

pub struct Session {
    pub id: String,
    pub messages: RwLock<Vec<Message>>,
    pub config: SessionConfig,
    pub provider: RwLock<Option<Arc<dyn crate::api::providers::Provider>>>,
}

unsafe impl Send for Session {}
unsafe impl Sync for Session {}

impl Session {
    pub fn add_message(&self, message: Message) -> Result<(), ClaudeError> {
        let mut messages = self.messages.write();
        messages.push(message);
        Ok(())
    }
}

pub struct SessionManager {
    sessions: RwLock<Vec<Arc<Session>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(Vec::new()),
        }
    }

    pub fn create_session(&self, config: SessionConfig) -> Result<Arc<Session>, ClaudeError> {
        let id = uuid::Uuid::new_v4().to_string();
        let session = Arc::new(Session {
            id,
            messages: RwLock::new(Vec::new()),
            config,
            provider: RwLock::new(None),
        });

        let mut sessions = self.sessions.write();
        sessions.push(Arc::clone(&session));

        Ok(session)
    }

    pub fn get_session(&self, id: &str) -> Option<Arc<Session>> {
        let sessions = self.sessions.read();
        sessions.iter().find(|s| s.id == id).map(Arc::clone)
    }

    pub fn remove_session(&self, id: &str) -> bool {
        let mut sessions = self.sessions.write();
        if let Some(pos) = sessions.iter().position(|s| s.id == id) {
            sessions.remove(pos);
            true
        } else {
            false
        }
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

static SESSION_MANAGER: once_cell::sync::Lazy<SessionManager> =
    once_cell::sync::Lazy::new(SessionManager::new);

pub fn get_session_manager() -> &'static SessionManager {
    &SESSION_MANAGER
}

#[no_mangle]
pub extern "C" fn claude_create_session(config_json: *const c_char) -> *mut c_void {
    if config_json.is_null() {
        return ptr::null_mut();
    }

    let config_str = match unsafe { std::ffi::CStr::from_ptr(config_json) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let config: SessionConfig = match serde_json::from_str(config_str) {
        Ok(c) => c,
        Err(_) => return ptr::null_mut(),
    };

    let session = match get_session_manager().create_session(config) {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    Arc::into_raw(session) as *mut c_void
}

#[no_mangle]
pub extern "C" fn claude_send_message(
    session: *mut c_void,
    content: *const c_char,
) -> *mut c_char {
    if session.is_null() || content.is_null() {
        return ptr::null_mut();
    }

    let session = unsafe { &*(session as *const Arc<Session>) };

    let content_str = match unsafe { std::ffi::CStr::from_ptr(content) }.to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };

    let user_message = Message::user(content_str);

    {
        let mut messages = session.messages.write();
        messages.push(user_message);
    }

    let response = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(crate::execute_query(session, &session.messages.read()));

    let response_str = match response {
        Ok(resp) => resp,
        Err(e) => serde_json::to_string(&e).unwrap_or_else(|_| r#"{"error":"unknown"}"#.to_string()),
    };

    let assistant_message = Message::assistant(&response_str);

    {
        let mut messages = session.messages.write();
        messages.push(assistant_message);
    }

    match CString::new(response_str) {
        Ok(s) => s.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

pub type StreamCallback = extern "C" fn(*const c_char, *mut c_void);

#[no_mangle]
pub extern "C" fn claude_stream_message(
    session: *mut c_void,
    content: *const c_char,
    callback: StreamCallback,
    user_data: *mut c_void,
) -> c_int {
    if session.is_null() || content.is_null() {
        return -1;
    }

    let session = unsafe { &*(session as *const Arc<Session>) };

    let content_str = match unsafe { std::ffi::CStr::from_ptr(content) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let user_message = Message::user(content_str);

    {
        let mut messages = session.messages.write();
        messages.push(user_message);
    }

    struct CallbackWrapper {
        callback: extern "C" fn(*const std::os::raw::c_char, *mut std::os::raw::c_void),
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
        (callback_wrapper.callback)(c_chunk.as_ptr(), callback_wrapper.user_data as *mut std::os::raw::c_void);
    };

    let messages_clone: Vec<Message> = {
        let messages = session.messages.read();
        messages.clone()
    };
    
    match tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(crate::execute_streaming_query(&session, &messages_clone, &mut send_callback))
    {
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
pub extern "C" fn claude_destroy_session(session: *mut c_void) {
    if session.is_null() {
        return;
    }

    let session = unsafe { Arc::from_raw(session as *const Session) };
    let id = session.id.clone();
    drop(session);

    get_session_manager().remove_session(&id);
}

#[no_mangle]
pub extern "C" fn claude_get_messages(session: *mut c_void) -> *mut c_char {
    if session.is_null() {
        return ptr::null_mut();
    }

    let session = unsafe { &*(session as *const Arc<Session>) };
    let messages = session.messages.read();

    let message_list = MessageList {
        messages: messages.clone(),
    };

    let json = serde_json::to_string(&message_list).unwrap_or_else(|_| r#"{"messages":[]}"#.to_string());

    match CString::new(json) {
        Ok(s) => s.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn claude_free_string(s: *mut c_char) {
    if !s.is_null() {
        unsafe { drop(CString::from_raw(s)) };
    }
}

use std::ffi::CString;
