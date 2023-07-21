pub trait Permissions {
    fn can_manage_users(&self) -> bool;

    fn can_manage_system(&self) -> bool;
}
