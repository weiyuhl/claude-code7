use rusqlite::{Connection, Result, params, OptionalExtension};
use chrono::Utc;
use super::schema::MessageEntity;
use super::connection::DbConnection;

pub struct MessageDao {
    conn: DbConnection,
}

impl MessageDao {
    pub fn new(conn: DbConnection) -> Self {
        Self { conn }
    }

    pub fn insert(&self, message: &MessageEntity) -> Result<()> {
        self.conn.with_connection_mut(|conn| {
            conn.execute(
                "INSERT INTO messages (id, session_id, parent_uuid, role, content, name, tool_call_id, 
                 tool_name, tool_input, thinking, thinking_signature, timestamp, 
                 is_compact_summary, compact_boundary, order_index)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                params![
                    message.id,
                    message.session_id,
                    message.parent_uuid,
                    message.role,
                    message.content,
                    message.name,
                    message.tool_call_id,
                    message.tool_name,
                    message.tool_input,
                    message.thinking,
                    message.thinking_signature,
                    message.timestamp.to_rfc3339(),
                    if message.is_compact_summary { 1 } else { 0 },
                    message.compact_boundary,
                    message.order_index,
                ],
            )?;
            Ok(())
        })
    }

    pub fn insert_batch(&self, messages: &[MessageEntity]) -> Result<()> {
        self.conn.with_connection_mut(|conn| {
            let tx = conn.transaction()?;
            for msg in messages {
                tx.execute(
                    "INSERT INTO messages (id, session_id, parent_uuid, role, content, name, tool_call_id, 
                     tool_name, tool_input, thinking, thinking_signature, timestamp, 
                     is_compact_summary, compact_boundary, order_index)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
                    params![
                        msg.id,
                        msg.session_id,
                        msg.parent_uuid,
                        msg.role,
                        msg.content,
                        msg.name,
                        msg.tool_call_id,
                        msg.tool_name,
                        msg.tool_input,
                        msg.thinking,
                        msg.thinking_signature,
                        msg.timestamp.to_rfc3339(),
                        if msg.is_compact_summary { 1 } else { 0 },
                        msg.compact_boundary,
                        msg.order_index,
                    ],
                )?;
            }
            tx.commit()?;
            Ok(())
        })
    }

    pub fn get_by_session(&self, session_id: &str) -> Result<Vec<MessageEntity>> {
        self.conn.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, parent_uuid, role, content, name, tool_call_id, 
                        tool_name, tool_input, thinking, thinking_signature, timestamp, 
                        is_compact_summary, compact_boundary, order_index
                 FROM messages 
                 WHERE session_id = ?1 
                 ORDER BY order_index ASC"
            )?;
            
            let messages = stmt.query_map(params![session_id], |row| {
                Ok(MessageEntity {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    parent_uuid: row.get(2)?,
                    role: row.get(3)?,
                    content: row.get(4)?,
                    name: row.get(5)?,
                    tool_call_id: row.get(6)?,
                    tool_name: row.get(7)?,
                    tool_input: row.get(8)?,
                    thinking: row.get(9)?,
                    thinking_signature: row.get(10)?,
                    timestamp: row.get::<_, String>(11)?.parse().unwrap_or_else(|_| Utc::now()),
                    is_compact_summary: row.get::<_, i32>(12)? == 1,
                    compact_boundary: row.get(13)?,
                    order_index: row.get(14)?,
                })
            })?;
            
            Ok(messages.filter_map(|r| r.ok()).collect::<Vec<_>>())
        })
    }

    pub fn get_by_session_with_limit(&self, session_id: &str, limit: i32) -> Result<Vec<MessageEntity>> {
        self.conn.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, parent_uuid, role, content, name, tool_call_id, 
                        tool_name, tool_input, thinking, thinking_signature, timestamp, 
                        is_compact_summary, compact_boundary, order_index
                 FROM messages 
                 WHERE session_id = ?1 
                 ORDER BY order_index DESC 
                 LIMIT ?2"
            )?;
            
            let messages = stmt.query_map(params![session_id, limit], |row| {
                Ok(MessageEntity {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    parent_uuid: row.get(2)?,
                    role: row.get(3)?,
                    content: row.get(4)?,
                    name: row.get(5)?,
                    tool_call_id: row.get(6)?,
                    tool_name: row.get(7)?,
                    tool_input: row.get(8)?,
                    thinking: row.get(9)?,
                    thinking_signature: row.get(10)?,
                    timestamp: row.get::<_, String>(11)?.parse().unwrap_or_else(|_| Utc::now()),
                    is_compact_summary: row.get::<_, i32>(12)? == 1,
                    compact_boundary: row.get(13)?,
                    order_index: row.get(14)?,
                })
            })?;
            
            let mut result: Vec<MessageEntity> = messages.filter_map(|r| r.ok()).collect();
            result.reverse();
            Ok(result)
        })
    }

    pub fn get_by_id(&self, message_id: &str) -> Result<Option<MessageEntity>> {
        self.conn.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, parent_uuid, role, content, name, tool_call_id, 
                        tool_name, tool_input, thinking, thinking_signature, timestamp, 
                        is_compact_summary, compact_boundary, order_index
                 FROM messages WHERE id = ?1"
            )?;
            
            let message = stmt.query_row(params![message_id], |row| {
                Ok(MessageEntity {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    parent_uuid: row.get(2)?,
                    role: row.get(3)?,
                    content: row.get(4)?,
                    name: row.get(5)?,
                    tool_call_id: row.get(6)?,
                    tool_name: row.get(7)?,
                    tool_input: row.get(8)?,
                    thinking: row.get(9)?,
                    thinking_signature: row.get(10)?,
                    timestamp: row.get::<_, String>(11)?.parse().unwrap_or_else(|_| Utc::now()),
                    is_compact_summary: row.get::<_, i32>(12)? == 1,
                    compact_boundary: row.get(13)?,
                    order_index: row.get(14)?,
                })
            }).optional()?;
            
            Ok(message)
        })
    }

    pub fn get_children(&self, parent_uuid: &str) -> Result<Vec<MessageEntity>> {
        self.conn.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, parent_uuid, role, content, name, tool_call_id, 
                        tool_name, tool_input, thinking, thinking_signature, timestamp, 
                        is_compact_summary, compact_boundary, order_index
                 FROM messages 
                 WHERE parent_uuid = ?1 
                 ORDER BY order_index ASC"
            )?;
            
            let messages = stmt.query_map(params![parent_uuid], |row| {
                Ok(MessageEntity {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    parent_uuid: row.get(2)?,
                    role: row.get(3)?,
                    content: row.get(4)?,
                    name: row.get(5)?,
                    tool_call_id: row.get(6)?,
                    tool_name: row.get(7)?,
                    tool_input: row.get(8)?,
                    thinking: row.get(9)?,
                    thinking_signature: row.get(10)?,
                    timestamp: row.get::<_, String>(11)?.parse().unwrap_or_else(|_| Utc::now()),
                    is_compact_summary: row.get::<_, i32>(12)? == 1,
                    compact_boundary: row.get(13)?,
                    order_index: row.get(14)?,
                })
            })?;
            
            Ok(messages.filter_map(|r| r.ok()).collect::<Vec<_>>())
        })
    }

    pub fn get_latest_order_index(&self, session_id: &str) -> Result<i32> {
        self.conn.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT COALESCE(MAX(order_index), -1) FROM messages WHERE session_id = ?1"
            )?;
            
            let index: i32 = stmt.query_row(params![session_id], |row| row.get(0))?;
            Ok(index)
        })
    }

    pub fn update_content(&self, message_id: &str, content: &str) -> Result<()> {
        self.conn.with_connection_mut(|conn| {
            conn.execute(
                "UPDATE messages SET content = ?1 WHERE id = ?2",
                params![content, message_id],
            )?;
            Ok(())
        })
    }

    pub fn delete(&self, message_id: &str) -> Result<()> {
        self.conn.with_connection_mut(|conn| {
            conn.execute(
                "DELETE FROM messages WHERE id = ?1",
                params![message_id],
            )?;
            Ok(())
        })
    }

    pub fn delete_by_session(&self, session_id: &str) -> Result<()> {
        self.conn.with_connection_mut(|conn| {
            conn.execute(
                "DELETE FROM messages WHERE session_id = ?1",
                params![session_id],
            )?;
            Ok(())
        })
    }

    pub fn delete_before_index(&self, session_id: &str, before_index: i32) -> Result<i32> {
        self.conn.with_connection_mut(|conn| {
            let tx = conn.transaction()?;
            
            let count: i32 = tx.query_row(
                "SELECT COUNT(*) FROM messages WHERE session_id = ?1 AND order_index < ?2",
                params![session_id, before_index],
                |row| row.get(0),
            )?;
            
            tx.execute(
                "DELETE FROM messages WHERE session_id = ?1 AND order_index < ?2",
                params![session_id, before_index],
            )?;
            
            tx.commit()?;
            Ok(count)
        })
    }

    pub fn mark_as_compact_summary(&self, message_id: &str, boundary_id: &str) -> Result<()> {
        self.conn.with_connection_mut(|conn| {
            conn.execute(
                "UPDATE messages SET is_compact_summary = 1, compact_boundary = ?1 WHERE id = ?2",
                params![boundary_id, message_id],
            )?;
            Ok(())
        })
    }

    pub fn get_compact_summaries(&self, session_id: &str) -> Result<Vec<MessageEntity>> {
        self.conn.with_connection(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, session_id, parent_uuid, role, content, name, tool_call_id, 
                        tool_name, tool_input, thinking, thinking_signature, timestamp, 
                        is_compact_summary, compact_boundary, order_index
                 FROM messages 
                 WHERE session_id = ?1 AND is_compact_summary = 1
                 ORDER BY order_index ASC"
            )?;
            
            let messages = stmt.query_map(params![session_id], |row| {
                Ok(MessageEntity {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    parent_uuid: row.get(2)?,
                    role: row.get(3)?,
                    content: row.get(4)?,
                    name: row.get(5)?,
                    tool_call_id: row.get(6)?,
                    tool_name: row.get(7)?,
                    tool_input: row.get(8)?,
                    thinking: row.get(9)?,
                    thinking_signature: row.get(10)?,
                    timestamp: row.get::<_, String>(11)?.parse().unwrap_or_else(|_| Utc::now()),
                    is_compact_summary: row.get::<_, i32>(12)? == 1,
                    compact_boundary: row.get(13)?,
                    order_index: row.get(14)?,
                })
            })?;
            
            Ok(messages.filter_map(|r| r.ok()).collect::<Vec<_>>())
        })
    }
}
