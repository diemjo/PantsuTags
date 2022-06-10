pub(crate) mod select_transactions;
pub use select_transactions::*;

pub(crate) mod update_transactions;
pub use update_transactions::*;

pub(crate) mod insert_transactions;
pub use insert_transactions::*;

pub(crate) mod delete_transactions;
pub use delete_transactions::*;

/*pub trait PantsuTransaction<T> {
    fn execute(self) -> Result<T>;
}*/