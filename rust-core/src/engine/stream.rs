use crate::session::ClaudeError;
use futures::io::AsyncBufReadExt;
use std::pin::Pin;
use futures::io::AsyncRead;

pub struct StreamChunk {
    pub content: String,
    pub is_complete: bool,
    pub tool_call: Option<ToolCallDelta>,
}

pub struct ToolCallDelta {
    pub id: String,
    pub name: String,
    pub input_delta: String,
}

impl StreamChunk {
    pub fn new_text(content: &str, is_complete: bool) -> Self {
        Self {
            content: content.to_string(),
            is_complete,
            tool_call: None,
        }
    }

    pub fn new_tool_call(id: &str, name: &str, input_delta: &str, is_complete: bool) -> Self {
        Self {
            content: String::new(),
            is_complete,
            tool_call: Some(ToolCallDelta {
                id: id.to_string(),
                name: name.to_string(),
                input_delta: input_delta.to_string(),
            }),
        }
    }
}

pub struct SSEParser {
    event_type: Option<String>,
    data_buffer: String,
}

impl SSEParser {
    pub fn new() -> Self {
        Self {
            event_type: None,
            data_buffer: String::new(),
        }
    }

    pub fn parse_line(&mut self, line: &str) -> Option<StreamChunk> {
        if line.is_empty() {
            let chunk = self.flush_event();
            self.event_type = None;
            self.data_buffer.clear();
            return chunk;
        }

        if let Some(data) = line.strip_prefix("data: ") {
            self.data_buffer.push_str(data);
        } else if let Some(event) = line.strip_prefix("event: ") {
            self.event_type = Some(event.to_string());
        } else if line.starts_with(":") {
            // Comment line, ignore
        } else if let Some(data) = line.strip_prefix("data:") {
            self.data_buffer.push_str(data);
        }

        None
    }

    fn flush_event(&mut self) -> Option<StreamChunk> {
        if self.data_buffer.is_empty() {
            return None;
        }

        let data = &self.data_buffer;
        
        if data == "[DONE]" {
            return Some(StreamChunk::new_text("", true));
        }

        if let Ok(json) = serde_json::from_str::<serde_json::Value>(data) {
            if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                if let Some(first_choice) = choices.first() {
                    if let Some(delta) = first_choice.get("delta") {
                        if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                            return Some(StreamChunk::new_text(content, false));
                        }
                    }
                    
                    if let Some(text) = first_choice.get("text").and_then(|t| t.as_str()) {
                        return Some(StreamChunk::new_text(text, false));
                    }
                }
            }
            
            if let Some(content) = json.get("content").and_then(|c| c.as_str()) {
                return Some(StreamChunk::new_text(content, false));
            }
        }

        None
    }

    pub fn parse_stream<R: AsyncRead + Unpin + Send + 'static>(
        mut reader: R,
        mut callback: impl FnMut(String) + Send + 'static,
    ) -> Pin<Box<dyn futures::Future<Output = Result<String, ClaudeError>> + Send>> {
        Box::pin(async move {
            let mut buf_reader = futures::io::BufReader::new(&mut reader);
            let mut line = String::new();
            let mut full_response = String::new();
            let mut parser = SSEParser::new();

            loop {
                line.clear();
                let bytes_read = match buf_reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(e) => return Err(ClaudeError::IoError {
                        path: String::new(),
                        message: e.to_string(),
                    }),
                };

                if bytes_read == 0 {
                    break;
                }

                let line_trimmed = line.trim_end_matches(|c| c == '\r' || c == '\n');
                if let Some(chunk) = parser.parse_line(line_trimmed) {
                    if !chunk.content.is_empty() {
                        full_response.push_str(&chunk.content);
                        callback(chunk.content);
                    }
                    if chunk.is_complete {
                        break;
                    }
                }
            }

            Ok(full_response)
        })
    }
}

impl Default for SSEParser {
    fn default() -> Self {
        Self::new()
    }
}

pub fn parse_sse_stream<F>(reader: impl AsyncRead + Unpin + Send + 'static, callback: F) -> Pin<Box<dyn futures::Future<Output = Result<String, ClaudeError>> + Send>>
where
    F: FnMut(String) + Send + 'static,
{
    SSEParser::parse_stream(reader, callback)
}
