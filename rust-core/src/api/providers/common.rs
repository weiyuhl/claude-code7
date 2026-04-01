use crate::session::ClaudeError;

pub fn parse_sse_line(line: &str) -> Option<String> {
    if line.starts_with("data: ") {
        let data = &line[6..];
        if data == "[DONE]" {
            None
        } else {
            Some(data.to_string())
        }
    } else {
        None
    }
}

pub fn estimate_token_count(text: &str) -> usize {
    text.chars().count() / 4
}

pub fn truncate_text(text: &str, max_tokens: usize) -> String {
    let max_chars = max_tokens * 4;
    if text.chars().count() > max_chars {
        text.chars().take(max_chars).collect()
    } else {
        text.to_string()
    }
}

pub fn retry_with_backoff<F, T, Fut>(mut f: F, max_retries: u32) -> impl std::future::Future<Output = Result<T, ClaudeError>>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, ClaudeError>>,
{
    async move {
        let mut attempts = 0;
        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempts += 1;
                    if attempts >= max_retries {
                        return Err(e);
                    }
                    let delay = std::time::Duration::from_millis(500 * 2_u64.pow(attempts));
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
}
