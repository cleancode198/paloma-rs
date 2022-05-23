use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use cosmwasm_std::{CosmosMsg, Empty};

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct InstantiateMsg {
    pub admins: Vec<String>,
    pub mutable: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    /// Execute contract requests to re-dispatch messages with the
    /// contract's address as a sender. Every implementation has its own logic
    Execute { msgs: Vec<CosmosMsg<T>> },
    /// Freeze will make a mutable contract immutable, must be called by an admin
    Freeze {},
    /// UpdateAdmins will change the admin set of the contract, must be called by an existing admin,
    /// and only works if the contract is mutable
    UpdateAdmins { admins: Vec<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    /// Shows all admins and whether or not the contract is mutable
    AdminList {},
    /// Checks caller permissions on this proxy.
    /// If CanExecute returns true, then a call to `Execute` with the same message,
    /// before any further state changes, should also succeed.
    CanExecute { sender: String, msg: CosmosMsg<T> },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct AdminListResponse {
    pub admins: Vec<String>,
    pub mutable: bool,
}

#[cfg(any(test, feature = "test-utils"))]
impl AdminListResponse {
    /// Utility function for converting a message to its canonical form, so two messages with
    /// different representations but same semantical meaning can be easly compared.
    ///
    /// It could be encapsulated in a custom `PartialEq` implementation, but `PartialEq` is expected
    /// to be quick, so it is reasonable to keep it as a representation-equality, and
    /// canonicalize a message only when it is needed
    ///
    /// Example:
    ///
    /// ```
    /// # use cw1_whitelist::msg::AdminListResponse;
    ///
    /// let resp1 = AdminListResponse {
    ///   admins: vec!["admin1".to_owned(), "admin2".to_owned()],
    ///   mutable: true,
    /// };
    ///
    /// let resp2 = AdminListResponse {
    ///   admins: vec!["admin2".to_owned(), "admin1".to_owned(), "admin2".to_owned()],
    ///   mutable: true,
    /// };
    ///
    /// assert_eq!(resp1.canonical(), resp2.canonical());
    /// ```
    pub fn canonical(mut self) -> Self {
        self.admins.sort();
        self.admins.dedup();
        self
    }
}
