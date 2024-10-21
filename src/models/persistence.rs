// persistence.rs

use std::fs::{File, self};
use std::io::{self, Read, Write};
use serde_json;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::models::user::UserPool;
use crate::DAGs::transaction_dag::DAG;

pub fn save_user_pool_state(user_pool: &UserPool) -> io::Result<()> {
    let serialized_user_pool = serde_json::to_string(user_pool)?;
    let mut file = File::create("user_pool_state.json")?;
    file.write_all(serialized_user_pool.as_bytes())?;
    Ok(())
}

pub fn load_user_pool_state() -> Option<UserPool> {
    if let Ok(mut file) = File::open("user_pool_state.json") {
        let mut serialized_user_pool = String::new();
        file.read_to_string(&mut serialized_user_pool).ok()?;
        serde_json::from_str(&serialized_user_pool).ok()
    } else {
        None
    }
}

pub fn save_dag_state(dag: &DAG) -> io::Result<()> {
    let serialized_dag = serde_json::to_string(dag)?;
    let mut file = File::create("dag_state.json")?;
    file.write_all(serialized_dag.as_bytes())?;
    Ok(())
}

pub fn load_dag_state(user_pool: Arc<RwLock<UserPool>>) -> Option<DAG> {
    if let Ok(mut file) = File::open("dag_state.json") {
        let mut serialized_dag = String::new();
        file.read_to_string(&mut serialized_dag).ok()?;
        if let Ok(mut dag) = serde_json::from_str::<DAG>(&serialized_dag) {
            dag.user_pool = user_pool; // Reattach user_pool
            Some(dag)
        } else {
            None
        }
    } else {
        None
    }
}
