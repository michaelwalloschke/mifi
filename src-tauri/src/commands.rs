//! Tauri commands: the only bridge between the webview and the Rust core. No secrets
//! cross this boundary (SPEC.md §2); every query here is read-only.

use serde::Serialize;
use tauri::State;

use mifi_core::domain::overview;

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct AccountDto {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct SplitDto {
    pub category_name: Option<String>,
    pub amount_cents: i64,
    pub category_source: String,
}

#[derive(Debug, Serialize)]
pub struct TransactionDto {
    pub id: i64,
    pub booking_date: String,
    pub account_name: String,
    pub counterparty: String,
    pub purpose: String,
    pub amount_cents: i64,
    pub is_transfer: bool,
    pub splits: Vec<SplitDto>,
}

#[tauri::command]
pub fn list_accounts(state: State<AppState>) -> Result<Vec<AccountDto>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, name FROM account ORDER BY id")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| Ok(AccountDto { id: row.get(0)?, name: row.get(1)? }))
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_transactions(
    state: State<AppState>,
    account_id: Option<i64>,
    search: Option<String>,
) -> Result<Vec<TransactionDto>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.booking_date, a.name, t.counterparty_raw, t.purpose_raw, t.amount_cents,
                    EXISTS(SELECT 1 FROM transfer WHERE leg_a_txn_id = t.id OR leg_b_txn_id = t.id) AS is_transfer,
                    s.amount_cents, s.category_source, c.name
             FROM \"transaction\" t
             JOIN account a ON a.id = t.account_id
             LEFT JOIN split s ON s.transaction_id = t.id
             LEFT JOIN category c ON c.id = s.category_id
             WHERE (?1 IS NULL OR t.account_id = ?1)
               AND (?2 IS NULL OR t.counterparty_raw LIKE '%' || ?2 || '%' OR t.purpose_raw LIKE '%' || ?2 || '%')
             ORDER BY t.booking_date DESC, t.id DESC, s.id ASC",
        )
        .map_err(|e| e.to_string())?;

    let mut transactions: Vec<TransactionDto> = Vec::new();
    let mut rows = stmt.query((account_id, search)).map_err(|e| e.to_string())?;
    while let Some(row) = rows.next().map_err(|e| e.to_string())? {
        let id: i64 = row.get(0).map_err(|e| e.to_string())?;
        let split = SplitDto {
            amount_cents: row.get(7).map_err(|e| e.to_string())?,
            category_source: row.get(8).map_err(|e| e.to_string())?,
            category_name: row.get(9).map_err(|e| e.to_string())?,
        };

        if transactions.last().is_some_and(|t| t.id == id) {
            transactions.last_mut().unwrap().splits.push(split);
        } else {
            transactions.push(TransactionDto {
                id,
                booking_date: row.get(1).map_err(|e| e.to_string())?,
                account_name: row.get(2).map_err(|e| e.to_string())?,
                counterparty: row.get(3).map_err(|e| e.to_string())?,
                purpose: row.get(4).map_err(|e| e.to_string())?,
                amount_cents: row.get(5).map_err(|e| e.to_string())?,
                is_transfer: row.get(6).map_err(|e| e.to_string())?,
                splits: vec![split],
            });
        }
    }
    Ok(transactions)
}

#[derive(Debug, Serialize)]
pub struct MonthlyOverviewDto {
    pub month: String,
    pub einnahmen_cents: i64,
    pub ausgaben_cents: i64,
    pub sparquote_percent: f64,
    pub puffer_cents: i64,
}

impl From<overview::MonthlyOverview> for MonthlyOverviewDto {
    fn from(o: overview::MonthlyOverview) -> Self {
        Self {
            month: o.month,
            einnahmen_cents: o.einnahmen_cents,
            ausgaben_cents: o.ausgaben_cents,
            sparquote_percent: o.sparquote_percent,
            puffer_cents: o.puffer_cents,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OverviewDto {
    pub current: MonthlyOverviewDto,
    pub previous: MonthlyOverviewDto,
    /// Trailing 12 months (oldest first, current month last) — feeds each tile's sparkline.
    pub sparkline: Vec<MonthlyOverviewDto>,
}

#[tauri::command]
pub fn get_overview(state: State<AppState>, month: Option<String>) -> Result<OverviewDto, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let month = month.unwrap_or_else(|| chrono::Local::now().format("%Y-%m").to_string());
    let previous_month = overview::months_before(&month, 1);

    let current = overview::compute_month(&conn, &month).map_err(|e| e.to_string())?;
    let previous = overview::compute_month(&conn, &previous_month).map_err(|e| e.to_string())?;
    let sparkline = overview::compute_series(&conn, &month, 12).map_err(|e| e.to_string())?;

    Ok(OverviewDto {
        current: current.into(),
        previous: previous.into(),
        sparkline: sparkline.into_iter().map(Into::into).collect(),
    })
}
