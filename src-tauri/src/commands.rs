//! Tauri commands: the only bridge between the webview and the Rust core. No secrets
//! cross this boundary (SPEC.md §2); every query here is read-only.

use serde::Serialize;
use tauri::State;

use mifi_core::domain::{budget, overview};

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

// KATEGORIEN

#[derive(Debug, Serialize)]
pub struct CategoryDto {
    pub id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub kind: String,
}

#[tauri::command]
pub fn list_categories(state: State<AppState>) -> Result<Vec<CategoryDto>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT id, parent_id, name, kind FROM category ORDER BY parent_id IS NOT NULL, name")
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(CategoryDto { id: row.get(0)?, parent_id: row.get(1)?, name: row.get(2)?, kind: row.get(3)? })
        })
        .map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[derive(Debug, Serialize)]
pub struct SubcategoryDto {
    pub id: i64,
    pub name: String,
    pub spent_cents: i64,
}

#[derive(Debug, Serialize)]
pub struct ContractDto {
    pub id: i64,
    pub normalized_counterparty: String,
    pub direction: String,
    pub expected_amount_cents: i64,
    pub interval: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct CategoryDetailDto {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub month: String,
    pub spent_cents: i64,
    pub subcategories: Vec<SubcategoryDto>,
    pub contracts: Vec<ContractDto>,
}

#[tauri::command]
pub fn get_category_detail(
    state: State<AppState>,
    category_id: i64,
    month: Option<String>,
) -> Result<CategoryDetailDto, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let month = month.unwrap_or_else(|| chrono::Local::now().format("%Y-%m").to_string());

    let (name, kind): (String, String) = conn
        .query_row("SELECT name, kind FROM category WHERE id = ?1", [category_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .map_err(|e| e.to_string())?;
    let spent_cents = budget::spent_cents(&conn, category_id, &month).map_err(|e| e.to_string())?;

    let mut sub_stmt = conn
        .prepare("SELECT id, name FROM category WHERE parent_id = ?1 ORDER BY name")
        .map_err(|e| e.to_string())?;
    let subcategories = sub_stmt
        .query_map([category_id], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?
        .map(|r| {
            let (id, name) = r.map_err(|e: rusqlite::Error| e.to_string())?;
            let spent = budget::spent_cents(&conn, id, &month).map_err(|e| e.to_string())?;
            Ok(SubcategoryDto { id, name, spent_cents: spent })
        })
        .collect::<Result<Vec<_>, String>>()?;

    let mut contract_stmt = conn
        .prepare(
            "SELECT id, normalized_counterparty, direction, expected_amount_cents, interval, status
             FROM contract WHERE category_id = ?1 ORDER BY normalized_counterparty",
        )
        .map_err(|e| e.to_string())?;
    let contracts = contract_stmt
        .query_map([category_id], |row| {
            Ok(ContractDto {
                id: row.get(0)?,
                normalized_counterparty: row.get(1)?,
                direction: row.get(2)?,
                expected_amount_cents: row.get(3)?,
                interval: row.get(4)?,
                status: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(CategoryDetailDto { id: category_id, name, kind, month, spent_cents, subcategories, contracts })
}

// BUDGET

#[derive(Debug, Serialize)]
pub struct BudgetRowDto {
    pub category_id: i64,
    pub category_name: String,
    pub parent_id: Option<i64>,
    pub target_cents: i64,
    pub spent_cents: i64,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct BudgetOverviewDto {
    pub month: String,
    pub rows: Vec<BudgetRowDto>,
    pub unbudgeted_expense_cents: i64,
}

fn budget_state_str(state: budget::BudgetState) -> &'static str {
    match state {
        budget::BudgetState::OnTrack => "on_track",
        budget::BudgetState::Warning => "warning",
        budget::BudgetState::Over => "over",
    }
}

#[tauri::command]
pub fn get_budget_overview(state: State<AppState>, month: Option<String>) -> Result<BudgetOverviewDto, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let month = month.unwrap_or_else(|| chrono::Local::now().format("%Y-%m").to_string());

    let mut stmt = conn
        .prepare("SELECT id, parent_id, name FROM category WHERE kind = 'expense' ORDER BY parent_id IS NOT NULL, name")
        .map_err(|e| e.to_string())?;
    let categories = stmt
        .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Option<i64>>(1)?, row.get::<_, String>(2)?)))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut rows = Vec::new();
    for (id, parent_id, name) in categories {
        if let Some(target_cents) = budget::effective_target_cents(&conn, id, &month).map_err(|e| e.to_string())? {
            let spent_cents = budget::spent_cents(&conn, id, &month).map_err(|e| e.to_string())?;
            let budget_state = budget::state(target_cents, spent_cents);
            rows.push(BudgetRowDto {
                category_id: id,
                category_name: name,
                parent_id,
                target_cents,
                spent_cents,
                state: budget_state_str(budget_state).to_string(),
            });
        }
    }

    let unbudgeted_expense_cents = budget::unbudgeted_expense_cents(&conn, &month).map_err(|e| e.to_string())?;

    Ok(BudgetOverviewDto { month, rows, unbudgeted_expense_cents })
}

#[tauri::command]
pub fn set_budget_target(
    state: State<AppState>,
    category_id: i64,
    amount_cents: Option<i64>,
    effective_from_month: String,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO budget_target (category_id, amount_cents, effective_from_month) VALUES (?1, ?2, ?3)",
        (category_id, amount_cents, effective_from_month),
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
