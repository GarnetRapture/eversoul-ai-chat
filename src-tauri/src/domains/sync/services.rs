use super::repositories::SyncRepository;
use super::types::{RemoteDataPack, SyncError, SyncResult};
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

    /// 원격 API 서버로부터 페르소나 및 지식팩 데이터를 다운로드한다.
    ///
    /// DB 커넥션을 들고 있지 않은 순수 비동기 구간으로 분리하여,
    /// 이 `.await` 동안 SQLite `MutexGuard`(non-Send)가 스레드 경계를 넘지 않도록 한다.
    pub async fn fetch_remote_pack(&self) -> Result<RemoteDataPack, SyncError> {
        self.http
            .get::<RemoteDataPack>("/sync/fetch")
            .await
            .map_err(|e| SyncError::Network(e.to_string()))
    }

    /// 다운로드된 데이터팩을 로컬 SQLite 데이터베이스에 적재한다 (동기).
    pub fn persist_pack(conn: &Connection, pack: &RemoteDataPack) -> Result<SyncResult, SyncError> {
        let mut synced_count = 0;

        // 1. 페르소나 데이터 로컬 저장소 적재
        for persona in &pack.personas {
            PersonaRepository::save_persona(conn, persona)
                .map_err(|e| SyncError::Database(e.to_string()))?;
            synced_count += 1;
        }

        // 2. 지식팩 청크 로컬 저장소 적재
        for chunk in &pack.knowledges {
            KnowledgeRepository::insert_chunk(conn, chunk)
                .map_err(|e| SyncError::Database(e.to_string()))?;
            synced_count += 1;
        }

        // 3. 스타일팩(말투 프로필) 로컬 저장소 적재
        for style in &pack.styles {
            StyleRepository::save_style(conn, style)
                .map_err(|e| SyncError::Database(e.to_string()))?;
            synced_count += 1;
        }

        // 4. 동기화 성공 상태 메타데이터 기록
        SyncRepository::set_metadata(conn, "last_sync_status", "success")
            .map_err(|e| SyncError::Database(e.to_string()))?;

        Ok(SyncResult {
            success: true,
            synced_items: synced_count,
            error_message: None,
        })
    }
}
