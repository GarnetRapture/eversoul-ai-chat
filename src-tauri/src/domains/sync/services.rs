use super::repositories::SyncRepository;
use super::types::{LocalStatusSnapshot, RemoteDataPack, SyncError, SyncResult};
use crate::domains::knowledge::repositories::KnowledgeRepository;
use crate::domains::persona::repositories::PersonaRepository;
use crate::domains::style::repositories::StyleRepository;
use crate::infrastructure::http::HttpManager;
use rusqlite::Connection;

pub struct SyncService<'a> {
    http: &'a HttpManager,
}

impl<'a> SyncService<'a> {
    pub fn new(http: &'a HttpManager) -> Self {
        Self { http }
    }

    pub async fn fetch_remote_pack(&self) -> Result<RemoteDataPack, SyncError> {
        self.http
            .get::<RemoteDataPack>("/sync/fetch")
            .await
            .map_err(|e| SyncError::Network(e.to_string()))
    }

    pub fn persist_pack(conn: &Connection, pack: &RemoteDataPack) -> Result<SyncResult, SyncError> {
        let mut synced_count = 0;

        for persona in &pack.personas {
            PersonaRepository::save_persona(conn, persona)
                .map_err(|e| SyncError::Database(e.to_string()))?;
            synced_count += 1;
        }

        for chunk in &pack.knowledges {
            KnowledgeRepository::insert_chunk(conn, chunk)
                .map_err(|e| SyncError::Database(e.to_string()))?;
            synced_count += 1;
        }

        for style in &pack.styles {
            StyleRepository::save_style(conn, style)
                .map_err(|e| SyncError::Database(e.to_string()))?;
            synced_count += 1;
        }

        SyncRepository::set_metadata(conn, "last_sync_status", "success")
            .map_err(|e| SyncError::Database(e.to_string()))?;

        Ok(SyncResult {
            success: true,
            synced_items: synced_count,
            error_message: None,
        })
    }

    pub fn record_failure(conn: &Connection, error_message: &str) -> Result<(), SyncError> {
        SyncRepository::set_metadata(conn, "last_sync_status", "failure")
            .map_err(|e| SyncError::Database(e.to_string()))?;
        SyncRepository::set_metadata(conn, "last_sync_error", error_message)
            .map_err(|e| SyncError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn get_local_status(conn: &Connection) -> Result<LocalStatusSnapshot, SyncError> {
        Ok(LocalStatusSnapshot {
            persona_count: SyncRepository::count_table(conn, "persona_profile")
                .map_err(|e| SyncError::Database(e.to_string()))?,
            chat_room_count: SyncRepository::count_table(conn, "chat_room")
                .map_err(|e| SyncError::Database(e.to_string()))?,
            chat_message_count: SyncRepository::count_table(conn, "chat_message")
                .map_err(|e| SyncError::Database(e.to_string()))?,
            style_count: SyncRepository::count_table(conn, "style_profile")
                .map_err(|e| SyncError::Database(e.to_string()))?,
            knowledge_chunk_count: SyncRepository::count_table(conn, "knowledge_chunk")
                .map_err(|e| SyncError::Database(e.to_string()))?,
            memory_count: SyncRepository::count_table(conn, "persona_memory")
                .map_err(|e| SyncError::Database(e.to_string()))?,
            last_sync_status: SyncRepository::get_metadata(conn, "last_sync_status")
                .map_err(|e| SyncError::Database(e.to_string()))?,
            last_sync_error: SyncRepository::get_metadata(conn, "last_sync_error")
                .map_err(|e| SyncError::Database(e.to_string()))?,
        })
    }
}
