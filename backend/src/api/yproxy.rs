use std::collections::HashMap;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Result;
use yrs::types::ToJson;
use yrs::Out;
use yrs::ReadTxn;
use yrs::{Any, ArrayPrelim, In, Map, MapPrelim, MapRef, WriteTxn};

pub(crate) struct Task<'a> {
    id: &'a str,
    num: &'a str,
    name: &'a str,
    children: Vec<&'a str>,
    assignee: Option<&'a str>,
    reporter: Option<&'a str>,
    status: Option<&'a str>,
    status_time: Option<i64>,
}

pub(crate) struct YGraphProxy {
    graph: MapRef,
}

impl YGraphProxy {
    pub fn new(txn: &mut yrs::TransactionMut) -> Self {
        YGraphProxy {
            graph: txn.get_or_insert_map("graph"),
        }
    }

    pub fn size<T: ReadTxn>(&self, txn: &T) -> u32 {
        self.graph.len(txn)
    }

    pub fn set(&self, txn: &mut yrs::TransactionMut, task: &Task) {
        let mut y_task: HashMap<String, In> = HashMap::new();

        y_task.insert("id".into(), task.id.into());
        y_task.insert("num".into(), task.num.into());
        y_task.insert("name".into(), task.name.into());
        y_task.insert(
            "children".into(),
            In::Array(ArrayPrelim::from(task.children.clone())),
        );
        y_task.insert("assignee".into(), or_else_null(task.assignee));
        y_task.insert("reporter".into(), or_else_null(task.reporter));
        y_task.insert("status".into(), or_else_null(task.status));
        y_task.insert("statusTime".into(), or_else_null(task.status_time));

        self.graph
            .insert(txn, task.id, MapPrelim::from_iter(y_task));
    }

    pub fn has<T: ReadTxn>(&self, txn: &T, id: &str) -> bool {
        let result = self.graph.get(txn, id);
        result.is_some()
    }

    pub fn get<T: ReadTxn>(&self, txn: &T, id: &str) -> Result<YTaskProxy> {
        let Some(y_task) = self.graph.get(txn, id) else {
            return Err(anyhow!("task is missing: {id}"));
        };
        let y_task = match y_task {
            Out::YMap(map_ref) => map_ref,
            _ => return Err(anyhow!("task {id} is not a map")),
        };
        Ok(YTaskProxy::new(y_task))
    }

    pub fn json<T: ReadTxn>(&self, txn: &T) -> Any {
        self.graph.to_json(txn)
    }
}

pub(crate) struct YTaskProxy {
    y_task: MapRef,
}

impl YTaskProxy {
    pub fn new(y_task: MapRef) -> Self {
        YTaskProxy { y_task }
    }

    fn get_optional_string<T: ReadTxn>(&self, txn: &T, field: &str) -> Result<Option<Arc<str>>> {
        let Some(result) = self.y_task.get(txn, field) else {
            return Ok(None);
        };
        let Out::Any(Any::String(result)) = result else {
            return Err(anyhow!("invalid field: {field}: {result}"));
        };
        Ok(Some(result))
    }

    fn get_string<T: ReadTxn>(&self, txn: &T, field: &str) -> Result<Arc<str>> {
        let Some(result) = self.get_optional_string(txn, field)? else {
            return Err(anyhow!("field is missing: {field}"));
        };
        Ok(result)
    }

    pub fn get_id<T: ReadTxn>(&self, txn: &T) -> Result<Arc<str>> {
        self.get_string(txn, "id")
    }

    pub fn get_num<T: ReadTxn>(&self, txn: &T) -> Result<Arc<str>> {
        self.get_string(txn, "num")
    }

    pub fn get_name<T: ReadTxn>(&self, txn: &T) -> Result<Arc<str>> {
        self.get_string(txn, "name")
    }

    pub fn get_assignee<T: ReadTxn>(&self, txn: &T) -> Result<Option<Arc<str>>> {
        self.get_optional_string(txn, "assignee")
    }

    pub fn get_reporter<T: ReadTxn>(&self, txn: &T) -> Result<Option<Arc<str>>> {
        self.get_optional_string(txn, "reporter")
    }

    pub fn get_status<T: ReadTxn>(&self, txn: &T) -> Result<Option<Arc<str>>> {
        self.get_optional_string(txn, "status")
    }

    pub fn get_status_time<T: ReadTxn>(&self, txn: &T, field: &str) -> Result<Option<f64>> {
        let Some(result) = self.y_task.get(txn, field) else {
            return Ok(None);
        };
        let Out::Any(Any::Number(result)) = result else {
            return Err(anyhow!("invalid field: {field}: {result}"));
        };
        Ok(Some(result))
    }
}

fn or_else_null<T>(v: Option<T>) -> In
where
    T: Into<In>,
{
    v.map(|v| v.into()).unwrap_or(Any::Null.into())
}

#[cfg(test)]
mod tests {
    use yrs::{Doc, Transact};

    use super::*;

    #[test]
    fn it_works() {
        let doc = Doc::new();

        let mut txn = doc.transact_mut();
        let y_graph = YGraphProxy::new(&mut txn);
        y_graph.set(
            &mut txn,
            &Task {
                id: "1",
                num: "1",
                name: "Task 1",
                children: vec!["2"],
                assignee: Some("assigneed@gmail.com"),
                reporter: Some("reporter@gmail.com"),
                status: Some("Done"),
                status_time: Some(23),
            },
        );

        let graph = y_graph.json(&txn);
        println!("{:?}", graph);

        let y_task = match y_graph.get(&txn, "1") {
            Ok(y_task) => y_task,
            Err(e) => {
                panic!("Error getting task: {e}");
            }
        };
        println!("{:?}", y_task.get_id(&txn));
    }
}
