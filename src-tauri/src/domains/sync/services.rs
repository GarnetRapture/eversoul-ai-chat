use rusqlite::Connection;
use crate::infrastructure::http::HttpManager;
use crate::domains::persona::repositories::PersonaRepository;
use crate::domains::knowledge::repositories::KnowledgeRepository;
use super::types::{SyncResult, RemoteDataPack, SyncError};
use super::repositories::SyncRepository;

pub struct SyncService<'a> {
    conn: &'a Connection,
    http: &'a HttpManager,
}

impl<'a> SyncService<'a> {
    pub fn new(conn: &'a Connection, http: &'a HttpManager) -> Self {
        Self { conn, http }
    }

    /// 원격 API 서버로부터 페르소나 및 지식팩 데이터를 동기화하여 SQLite 로컬 DB에 적재한다.
    pub async fn run_synchronization(&self) -> Result<SyncResult, SyncError> {
        let fetch_path = "/sync/fetch";

        // 1. 원격 데이터팩 다운로드
        let pack: RemoteDataPack = self.http.get::<RemoteDataPack>(fetch_path).await
            .map_err(|e| SyncError::Network(e.to_string()))?;

        let mut synced_count = 0;

        // 2. 페르소나 데이터 로컬 저장소 적재
        for persona in &pack.personas {
            PersonaRepository::save_persona(self.conn, persona)
                .map_err(|e| SyncError::Database(e.to_string()))?;
            synced_count += 1;
        }

        // 3. 지식팩 청크 로컬 저장소 적재
        for chunk in &pack.knowledges {
            KnowledgeRepository::insert_chunk(self.conn, chunk)
                .map_err(|e| SyncError::Database(e.to_string()))?;
            synced_count += 1;
        }

        // 4. 동기화 성공 상태 메타데이터 기록
        SyncRepository::set_metadata(self.conn, "last_sync_status", "success")
            .map_err(|e| SyncError::Database(e.to_string()))?;

        Ok(SyncResult {
            success: true,
            synced_items: synced_count,
            error_message: None,
        })
    }
}
