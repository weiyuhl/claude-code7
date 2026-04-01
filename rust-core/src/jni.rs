use jni::JNIEnv;
use jni::objects::{JClass, JString};
use jni::sys::{jlong, jstring};

use crate::session::{Session, SessionConfig};
use crate::message::Message;
use crate::message::MessageList;
use crate::execute_query;
use crate::execute_streaming_query;
use crate::get_session_manager;
use std::sync::Arc;

fn config_from_jstring(env: &mut JNIEnv, config: JString) -> Option<SessionConfig> {
    let config_str: String = env.get_string(&config).ok()?.into();
    serde_json::from_str(&config_str).ok()
}

fn jstring_from_string(env: &mut JNIEnv, s: String) -> jstring {
    env.new_string(s).unwrap_or_else(|_| env.new_string("").unwrap()).into_raw()
}

#[no_mangle]
pub extern "system" fn Java_com_example_claude_ClaudeCore_nativeCreateSession(
    mut env: JNIEnv,
    _class: JClass,
    config: JString,
) -> jlong {
    let config = match config_from_jstring(&mut env, config) {
        Some(c) => c,
        None => return 0,
    };

    let session = match get_session_manager().create_session(config) {
        Ok(s) => s,
        Err(_) => return 0,
    };

    Arc::into_raw(session) as jlong
}

#[no_mangle]
pub extern "system" fn Java_com_example_claude_ClaudeCore_nativeSendMessage(
    mut env: JNIEnv,
    _class: JClass,
    session_ptr: jlong,
    content: JString,
) -> jstring {
    if session_ptr == 0 {
        return jstring_from_string(&mut env, r#"{"error":"invalid session"}"#.to_string());
    }

    let session_arc = unsafe { Arc::from_raw(session_ptr as *const Session) };
    let session_ref = Arc::clone(&session_arc);
    drop(session_arc);

    let content_str: String = match env.get_string(&content) {
        Ok(s) => s.into(),
        Err(_) => {
            return jstring_from_string(&mut env, r#"{"error":"invalid content"}"#.to_string());
        }
    };

    let user_message = Message::user(&content_str);

    {
        let mut messages = session_ref.messages.write();
        messages.push(user_message);
    }

    let messages_clone: Vec<Message> = {
        let messages = session_ref.messages.read();
        messages.clone()
    };

    let response = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(execute_query(&session_ref, &messages_clone));

    let response_str = match response {
        Ok(resp) => resp,
        Err(e) => serde_json::to_string(&e).unwrap_or_else(|_| r#"{"error":"unknown"}"#.to_string()),
    };

    let assistant_message = Message::assistant(&response_str);

    {
        let mut messages = session_ref.messages.write();
        messages.push(assistant_message);
    }

    std::mem::forget(session_ref);
    jstring_from_string(&mut env, response_str)
}

#[no_mangle]
pub extern "system" fn Java_com_example_claude_ClaudeCore_nativeDestroySession(
    _env: JNIEnv,
    _class: JClass,
    session_ptr: jlong,
) {
    if session_ptr == 0 {
        return;
    }

    let session = unsafe { Arc::from_raw(session_ptr as *const Session) };
    let id = session.id.clone();
    drop(session);

    get_session_manager().remove_session(&id);
}

#[no_mangle]
pub extern "system" fn Java_com_example_claude_ClaudeCore_nativeGetMessages(
    mut env: JNIEnv,
    _class: JClass,
    session_ptr: jlong,
) -> jstring {
    if session_ptr == 0 {
        return jstring_from_string(&mut env, r#"{"messages":[]}"#.to_string());
    }

    let session_arc = unsafe { Arc::from_raw(session_ptr as *const Session) };
    let session = &*session_arc;
    let messages = session.messages.read();

    let message_list = MessageList {
        messages: messages.clone(),
    };

    drop(messages);
    drop(session_arc);

    let json = serde_json::to_string(&message_list).unwrap_or_else(|_| r#"{"messages":[]}"#.to_string());

    jstring_from_string(&mut env, json)
}

pub type JniStreamCallback = extern "system" fn(JString, jlong);

#[no_mangle]
pub extern "system" fn Java_com_example_claude_ClaudeCore_nativeStreamMessage(
    mut env: JNIEnv,
    _class: JClass,
    session_ptr: jlong,
    content: JString,
    callback: JniStreamCallback,
    user_data: jlong,
) -> jstring {
    if session_ptr == 0 {
        return jstring_from_string(&mut env, r#"{"error":"invalid session"}"#.to_string());
    }

    let session_arc = unsafe { Arc::from_raw(session_ptr as *const Session) };
    let session_ref = Arc::clone(&session_arc);
    drop(session_arc);

    let content_str: String = match env.get_string(&content) {
        Ok(s) => s.into(),
        Err(_) => {
            return jstring_from_string(&mut env, r#"{"error":"invalid content"}"#.to_string());
        }
    };

    let user_message = Message::user(&content_str);

    {
        let mut messages = session_ref.messages.write();
        messages.push(user_message);
    }

    let messages_clone: Vec<Message> = {
        let messages = session_ref.messages.read();
        messages.clone()
    };

    struct CallbackWrapper {
        env_ptr: usize,
        callback: extern "system" fn(JString, jlong),
        user_data: jlong,
    }

    unsafe impl Send for CallbackWrapper {}

    let callback_wrapper = CallbackWrapper { 
        env_ptr: env.get_raw() as usize, 
        callback, 
        user_data,
    };
    
    let mut send_callback = move |chunk: String| {
        let env = unsafe { JNIEnv::from_raw(callback_wrapper.env_ptr as *mut _) }.unwrap();
        let chunk_jstring = env.new_string(chunk).unwrap();
        (callback_wrapper.callback)(chunk_jstring, callback_wrapper.user_data);
        std::mem::forget(env);
    };

    let response = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(execute_streaming_query(&session_ref, &messages_clone, &mut send_callback));

    let response_str = match response {
        Ok(resp) => resp,
        Err(e) => serde_json::to_string(&e).unwrap_or_else(|_| r#"{"error":"unknown"}"#.to_string()),
    };

    let assistant_message = Message::assistant(&response_str);

    {
        let mut messages = session_ref.messages.write();
        messages.push(assistant_message);
    }

    std::mem::forget(session_ref);
    jstring_from_string(&mut env, response_str)
}
