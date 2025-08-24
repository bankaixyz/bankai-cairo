use crate::fetcher::recursive_epoch_input::RecursiveEpochOutput;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteConnectOptions, FromRow, Pool, Sqlite};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum Status {
    Fetching,
    TraceGen,
    Proving,
    Done,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub struct EpochUpdate {
    pub uuid: String,
    pub epoch_number: i64,
    pub slot_number: i64,
    pub outputs: Option<RecursiveEpochOutput>,
    pub atlantic_id: Option<String>,
    pub proof_id: Option<i64>,
    pub status: String,
    pub error_reason: Option<String>,
}

#[derive(Debug, FromRow)]
struct EpochUpdateRow {
    pub uuid: String,
    pub epoch_number: i64,
    pub slot_number: i64,
    pub outputs: Option<String>,
    pub atlantic_id: Option<String>,
    pub proof_id: Option<i64>,
    pub status: String,
    pub error_reason: Option<String>,
}

impl From<EpochUpdateRow> for EpochUpdate {
    fn from(row: EpochUpdateRow) -> Self {
        let outputs = row
            .outputs
            .as_ref()
            .and_then(|json| serde_json::from_str(json).ok());

        EpochUpdate {
            uuid: row.uuid,
            epoch_number: row.epoch_number,
            slot_number: row.slot_number,
            outputs,
            atlantic_id: row.atlantic_id,
            proof_id: row.proof_id,
            status: row.status,
            error_reason: row.error_reason,
        }
    }
}

#[derive(Debug, FromRow)]
pub struct Proof {
    pub id: i64,
    pub proof: String,
}

#[derive(Debug)]
pub struct Database {
    pool: Pool<Sqlite>,
}

impl Database {
    pub async fn new(url: &str) -> Result<Self, sqlx::Error> {
        let options = SqliteConnectOptions::from_str(url)?.create_if_missing(true);

        let pool = Pool::connect_with(options).await?;
        sqlx::migrate!("../../../migrations").run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn create_epoch_update(
        &self,
        epoch_number: u64,
        slot_number: u64,
        outputs: RecursiveEpochOutput,
    ) -> Result<String, sqlx::Error> {
        let uuid = Uuid::new_v4().to_string();
        let outputs_json = serde_json::to_string(&outputs).unwrap();
        let epoch_number_i64 = epoch_number as i64;
        let slot_number_i64 = slot_number as i64;

        sqlx::query!(
            "INSERT INTO epoch_updates (uuid, epoch_number, slot_number, outputs, status) VALUES (?, ?, ?, ?, ?)",
            uuid,
            epoch_number_i64,
            slot_number_i64,
            outputs_json,
            "fetching"
        )
        .execute(&self.pool)
        .await?;

        Ok(uuid)
    }

    pub async fn update_outputs(
        &self,
        uuid: &str,
        outputs: &RecursiveEpochOutput,
    ) -> Result<(), sqlx::Error> {
        let outputs_json = serde_json::to_string(outputs).unwrap();

        sqlx::query!(
            "UPDATE epoch_updates SET outputs = ? WHERE uuid = ?",
            outputs_json,
            uuid
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn add_atlantic_id(&self, uuid: &str, atlantic_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE epoch_updates SET atlantic_id = ? WHERE uuid = ?",
            atlantic_id,
            uuid
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn add_proof(&self, proof_json: &str) -> Result<i64, sqlx::Error> {
        let result = sqlx::query!(
            "INSERT INTO proofs (proof) VALUES (?) RETURNING id",
            proof_json
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(result.id)
    }

    pub async fn update_proof_id(&self, uuid: &str, proof_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE epoch_updates SET proof_id = ? WHERE uuid = ?",
            proof_id,
            uuid
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_status(&self, uuid: &str, status: Status) -> Result<(), sqlx::Error> {
        let status_str = match status {
            Status::Fetching => "fetching",
            Status::TraceGen => "trace_gen",
            Status::Proving => "proving",
            Status::Done => "done",
            Status::Error => "error",
        };

        sqlx::query!(
            "UPDATE epoch_updates SET status = ? WHERE uuid = ?",
            status_str,
            uuid
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_error(&self, uuid: &str, error_reason: &str) -> Result<(), sqlx::Error> {
        sqlx::query!(
            "UPDATE epoch_updates SET status = 'error', error_reason = ? WHERE uuid = ?",
            error_reason,
            uuid
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_latest_epoch_update(&self) -> Result<Option<EpochUpdate>, sqlx::Error> {
        let row = sqlx::query_as::<_, EpochUpdateRow>(
            "SELECT uuid, epoch_number, slot_number, outputs, atlantic_id, proof_id, status, error_reason 
             FROM epoch_updates 
             WHERE status != 'error'
             ORDER BY slot_number DESC 
             LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    pub async fn get_epoch_update_by_uuid(
        &self,
        uuid: &str,
    ) -> Result<Option<EpochUpdate>, sqlx::Error> {
        let row = sqlx::query_as::<_, EpochUpdateRow>(
            "SELECT uuid, epoch_number, slot_number, outputs, atlantic_id, proof_id, status, error_reason 
             FROM epoch_updates 
             WHERE uuid = ?"
        )
        .bind(uuid)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(Into::into))
    }

    pub async fn get_proof(&self, proof_id: i64) -> Result<Option<Proof>, sqlx::Error> {
        let proof = sqlx::query_as!(Proof, "SELECT id, proof FROM proofs WHERE id = ?", proof_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(proof)
    }

    pub async fn get_all_epoch_updates(&self) -> Result<Vec<EpochUpdate>, sqlx::Error> {
        let rows = sqlx::query_as::<_, EpochUpdateRow>(
            "SELECT uuid, epoch_number, slot_number, outputs, atlantic_id, proof_id, status, error_reason 
             FROM epoch_updates 
             ORDER BY slot_number DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let updates = rows.into_iter().map(EpochUpdate::from).collect();
        Ok(updates)
    }

    pub async fn get_proof_by_beacon_height(
        &self,
        height: u64,
    ) -> Result<Option<Proof>, sqlx::Error> {
        let height_i64 = height as i64;
        let proof = sqlx::query_as(
            "SELECT p.id, p.proof
             FROM proofs p
             JOIN epoch_updates eu ON p.id = eu.proof_id
             WHERE json_extract(eu.outputs, '$.beacon_height') = ?",
        )
        .bind(height_i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(proof)
    }

    pub async fn get_proof_by_execution_height(
        &self,
        height: u64,
    ) -> Result<Option<Proof>, sqlx::Error> {
        let height_i64 = height as i64;
        let proof = sqlx::query_as(
            "SELECT p.id, p.proof
             FROM proofs p
             JOIN epoch_updates eu ON p.id = eu.proof_id
             WHERE json_extract(eu.outputs, '$.execution_header_height') = ?",
        )
        .bind(height_i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(proof)
    }
}
