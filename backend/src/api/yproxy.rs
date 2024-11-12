use std::collections::HashMap;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Result;
use yrs::Out;
use yrs::{Any, ArrayPrelim, Doc, In, Map, MapPrelim, MapRef, ReadTxn, Transact, WriteTxn};

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

pub(crate) struct YDocProxy<'a> {
    doc: &'a Doc,
}

impl<'a> YDocProxy<'a> {
    pub fn transact(&self) -> YGraphProxy<'a> {
        let txn: yrs::TransactionMut<'a> = self.doc.transact_mut();
        YGraphProxy { txn }
    }
}

pub(crate) struct YGraphProxy<'a> {
    txn: yrs::TransactionMut<'a>,
}

impl<'a> YGraphProxy<'a> {
    fn graph(&mut self) -> MapRef {
        self.txn.get_or_insert_map("graph")
    }

    pub fn size(&mut self) -> u32 {
        self.graph().len(&self.txn)
    }

    pub fn set(&mut self, task: &Task) {
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

        self.graph()
            .insert(&mut self.txn, task.id, MapPrelim::from_iter(y_task));
    }

    pub fn has(&mut self, id: &str) -> bool {
        let graph = self.txn.get_or_insert_map("graph");
        let result = graph.get(&self.txn, id);
        result.is_some()
    }

    pub fn get(&'a mut self, id: &str) -> Result<YTaskProxy<'a>> {
        let Some(y_task) = self.graph().get(&self.txn, id) else {
            return Err(anyhow!("task is missing: {id}"));
        };
        let y_task = match y_task {
            Out::YMap(map_ref) => map_ref,
            _ => return Err(anyhow!("task {id} is not a map")),
        };
        Ok(YTaskProxy {
            txn: &self.txn,
            y_task,
        })
    }
}

pub(crate) struct YTaskProxy<'a> {
    txn: &'a yrs::TransactionMut<'a>,
    y_task: MapRef,
}

impl<'a> YTaskProxy<'a> {
    fn get_optional_string(self, field: &str) -> Result<Option<Arc<str>>> {
        let Some(result) = self.y_task.get(self.txn, field) else {
            return Ok(None);
        };
        let Out::Any(Any::String(result)) = result else {
            return Err(anyhow!("invalid field: {field}: {result}"));
        };
        Ok(Some(result))
    }

    fn get_string(self, field: &str) -> Result<Arc<str>> {
        let Some(result) = self.get_optional_string(field)? else {
            return Err(anyhow!("field is missing: {field}"));
        };
        Ok(result)
    }

    pub fn get_id(self) -> Result<Arc<str>> {
        self.get_string("id")
    }

    pub fn get_num(self) -> Result<Arc<str>> {
        self.get_string("num")
    }

    pub fn get_name(self) -> Result<Arc<str>> {
        self.get_string("name")
    }

    pub fn get_assignee(self) -> Result<Option<Arc<str>>> {
        self.get_optional_string("assignee")
    }

    pub fn get_reporter(self) -> Result<Option<Arc<str>>> {
        self.get_optional_string("reporter")
    }

    pub fn get_status(self) -> Result<Option<Arc<str>>> {
        self.get_optional_string("status")
    }

    pub fn get_status_time(self, field: &str) -> Result<Option<f64>> {
        let Some(result) = self.y_task.get(self.txn, field) else {
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
    use yrs::{types::ToJson, Transact};

    use super::*;

    #[test]
    fn it_works() {
        let doc = Doc::new();

        {
            let y_doc = YDocProxy { doc: &doc };
            let mut y_graph = y_doc.transact();
            y_graph.set(&Task {
                id: "1",
                num: "1",
                name: "Task 1",
                children: vec!["2"],
                assignee: Some("assigneed@gmail.com"),
                reporter: Some("reporter@gmail.com"),
                status: Some("Done"),
                status_time: Some(23),
            });
        }

        let graph = doc
            .get_or_insert_map("graph")
            .get(&doc.transact(), "1")
            .unwrap()
            .to_json(&doc.transact());
        println!("{:?}", graph);

        {
            let y_doc = YDocProxy { doc: &doc };
            let mut y_graph = y_doc.transact();
            let y_task = match y_graph.get("1") {
                Ok(y_task) => y_task,
                Err(e) => {
                    panic!("Error getting task: {e}");
                }
            };
            println!("{:?}", y_task.get_id());
        }
    }
}
