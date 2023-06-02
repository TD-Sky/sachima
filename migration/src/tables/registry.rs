use sea_orm_migration::prelude::*;

/// User registry
#[derive(Iden)]
pub enum Registry {
    Table,

    /// 账号ID
    Id,

    /// 用户名
    Username,

    /// 密码
    Password,
}

impl Registry {
    pub fn table() -> TableCreateStatement {
        Table::create()
            .table(Self::Table)
            .if_not_exists()
            .col(
                ColumnDef::new(Self::Id)
                    .integer()
                    .primary_key()
                    .auto_increment(),
            )
            .col(
                ColumnDef::new(Self::Username)
                    .string()
                    .not_null()
                    .unique_key(),
            )
            .col(ColumnDef::new(Self::Password).string().not_null())
            .to_owned()
    }

    pub fn drop() -> TableDropStatement {
        Table::drop().table(Self::Table).to_owned()
    }
}
