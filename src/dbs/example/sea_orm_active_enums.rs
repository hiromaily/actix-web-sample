//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "todo_status")]
pub enum TodoStatus {
    #[sea_orm(string_value = "canceled")]
    Canceled,
    #[sea_orm(string_value = "doing")]
    Doing,
    #[sea_orm(string_value = "done")]
    Done,
    #[sea_orm(string_value = "pending")]
    Pending,
}
