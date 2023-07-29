use sea_orm::sea_query::{ArrayType, ColumnType, ValueType, ValueTypeErr};
use sea_orm::{
    ColIdx, DbErr, IntoActiveValue, JsonValue, QueryResult, TryGetError, TryGetable, Value,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupPermissions {
    pub modify_accounts: bool,
    pub manage_system: bool,
}

impl Default for GroupPermissions {
    fn default() -> Self {
        Self {
            modify_accounts: false,
            manage_system: false,
        }
    }
}
impl GroupPermissions {
    #[inline]
    pub fn new_admin() -> Self {
        Self {
            modify_accounts: true,
            manage_system: true,
        }
    }
}
impl From<GroupPermissions> for JsonValue {
    fn from(value: GroupPermissions) -> Self {
        serde_json::to_value(value).unwrap()
    }
}
impl From<GroupPermissions> for Value {
    fn from(value: GroupPermissions) -> Self {
        Value::Json(Some(serde_json::to_value(value).unwrap().into()))
    }
}
impl TryGetable for GroupPermissions {
    fn try_get_by<I: ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        let val: JsonValue = res.try_get_by(index).map_err(TryGetError::DbErr)?;
        serde_json::from_value(val).map_err(|e| TryGetError::DbErr(DbErr::Json(e.to_string())))
    }
}
impl ValueType for GroupPermissions {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        match v {
            Value::Json(Some(x)) => {
                let auth_properties: GroupPermissions =
                    serde_json::from_value(*x).map_err(|_error| ValueTypeErr)?;
                Ok(auth_properties)
            }
            _ => Err(ValueTypeErr),
        }
    }

    fn type_name() -> String {
        stringify!(GroupPermissions).to_owned()
    }

    fn array_type() -> ArrayType {
        ArrayType::Json
    }

    fn column_type() -> ColumnType {
        ColumnType::Json
    }
}
impl IntoActiveValue<GroupPermissions> for GroupPermissions {
    fn into_active_value(self) -> sea_orm::ActiveValue<GroupPermissions> {
        sea_orm::ActiveValue::Set(self)
    }
}
