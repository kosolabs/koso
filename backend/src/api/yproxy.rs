use super::model::Graph;
use crate::api::model::Task;
use anyhow::anyhow;
use anyhow::Result;
use similar::capture_diff_slices;
use similar::Algorithm;
use std::collections::HashMap;
use std::sync::Arc;
use yrs::{Any, Array, ArrayRef, Map, MapRef, Out, ReadTxn, TransactionMut, WriteTxn};

pub(crate) struct YGraphProxy {
    graph: MapRef,
}

impl YGraphProxy {
    pub fn new(txn: &mut yrs::TransactionMut) -> Self {
        YGraphProxy {
            graph: txn.get_or_insert_map("graph"),
        }
    }

    pub fn get_graph<T: ReadTxn>(&self, txn: &T) -> Result<Graph> {
        let mut graph: Graph = HashMap::new();
        for id in self.graph.keys(txn) {
            graph.insert(id.to_string(), self.get(txn, id)?.get_task(txn)?);
        }
        Ok(graph)
    }

    pub fn set(&self, txn: &mut yrs::TransactionMut, task: &Task) {
        let y_task: MapRef = self.graph.get_or_init(txn, task.id.as_ref());
        let y_task = YTaskProxy::new(y_task);
        y_task.set_id(txn, &task.id);
        y_task.set_num(txn, &task.num);
        y_task.set_name(txn, &task.name);
        y_task.set_children(txn, &task.children);
        y_task.set_assignee(txn, task.assignee.as_deref());
        y_task.set_reporter(txn, task.reporter.as_deref());
        y_task.set_status(txn, task.status.as_deref());
        y_task.set_status_time(txn, task.status_time);
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
}

pub(crate) struct YTaskProxy {
    y_task: MapRef,
}

impl YTaskProxy {
    pub fn new(y_task: MapRef) -> Self {
        YTaskProxy { y_task }
    }

    pub fn get_task<T: ReadTxn>(&self, txn: &T) -> Result<Task> {
        Ok(Task {
            id: self.get_id(txn)?.to_string(),
            num: self.get_num(txn)?.to_string(),
            name: self.get_name(txn)?.to_string(),
            children: self.get_children(txn)?,
            assignee: self.get_assignee(txn)?.map(|s| s.to_string()),
            reporter: self.get_reporter(txn)?.map(|s| s.to_string()),
            status: self.get_status(txn)?.map(|s| s.to_string()),
            status_time: self.get_status_time(txn)?,
        })
    }

    fn get_optional_number<T: ReadTxn>(&self, txn: &T, field: &str) -> Result<Option<i64>> {
        let Some(result) = self.y_task.get(txn, field) else {
            return Ok(None);
        };
        match result {
            Out::Any(Any::Number(result)) => Ok(Some(result as i64)),
            Out::Any(Any::Null) => Ok(None),
            _ => Err(anyhow!("invalid field: {field}: {result:?}")),
        }
    }

    fn get_optional_string<T: ReadTxn>(&self, txn: &T, field: &str) -> Result<Option<Arc<str>>> {
        let Some(result) = self.y_task.get(txn, field) else {
            return Ok(None);
        };
        match result {
            Out::Any(Any::String(result)) => Ok(Some(result)),
            Out::Any(Any::Null) => Ok(None),
            _ => Err(anyhow!("invalid field: {field}: {result:?}")),
        }
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

    fn set_id(&self, txn: &mut TransactionMut, id: &str) {
        self.y_task.try_update(txn, "id", id);
    }

    pub fn get_num<T: ReadTxn>(&self, txn: &T) -> Result<Arc<str>> {
        self.get_string(txn, "num")
    }

    pub fn set_num(&self, txn: &mut TransactionMut, num: &str) {
        self.y_task.try_update(txn, "num", num);
    }

    pub fn get_name<T: ReadTxn>(&self, txn: &T) -> Result<Arc<str>> {
        self.get_string(txn, "name")
    }

    pub fn set_name(&self, txn: &mut TransactionMut, name: &str) {
        self.y_task.try_update(txn, "name", name);
    }

    pub fn get_children<T: ReadTxn>(&self, txn: &T) -> Result<Vec<String>> {
        let Some(y_children) = self.y_task.get(txn, "children") else {
            return Ok(Vec::new());
        };
        let Out::YArray(y_children) = y_children else {
            return Err(anyhow!("invalid field: children: {y_children}"));
        };
        y_children
            .iter(txn)
            .map(|item| match item {
                Out::Any(Any::String(s)) => Ok(s.to_string()),
                e => Err(anyhow!("invalid child: {e}")),
            })
            .collect()
    }

    pub fn set_children(&self, txn: &mut TransactionMut, new_children: &[String]) {
        let y_children: ArrayRef = self.y_task.get_or_init(txn, "children");

        let old_children = match self.get_children(txn) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("invalid children: {e}, clobbering children");
                y_children.remove_range(txn, 0, y_children.len(txn));
                y_children.insert_range(txn, 0, new_children.to_vec());
                return;
            }
        };

        if old_children != *new_children {
            let ops = capture_diff_slices(Algorithm::Myers, &old_children, new_children)
                .into_iter()
                .rev()
                .collect::<Vec<_>>();

            for ops in ops {
                match ops {
                    similar::DiffOp::Delete {
                        old_index, old_len, ..
                    } => {
                        y_children.remove_range(txn, old_index as u32, old_len as u32);
                    }
                    similar::DiffOp::Insert {
                        old_index,
                        new_index,
                        new_len,
                    } => {
                        y_children.insert_range(
                            txn,
                            old_index as u32,
                            new_children[new_index..(new_index + new_len)].to_vec(),
                        );
                    }
                    similar::DiffOp::Replace {
                        old_index,
                        old_len,
                        new_index,
                        new_len,
                    } => {
                        y_children.remove_range(txn, old_index as u32, old_len as u32);
                        y_children.insert_range(
                            txn,
                            old_index as u32,
                            new_children[new_index..(new_index + new_len)].to_vec(),
                        );
                    }
                    _ => (),
                }
            }
        }
    }

    pub fn get_assignee<T: ReadTxn>(&self, txn: &T) -> Result<Option<Arc<str>>> {
        self.get_optional_string(txn, "assignee")
    }

    pub fn set_assignee(&self, txn: &mut TransactionMut, assignee: Option<&str>) {
        self.y_task.try_update(txn, "assignee", assignee);
    }

    pub fn get_reporter<T: ReadTxn>(&self, txn: &T) -> Result<Option<Arc<str>>> {
        self.get_optional_string(txn, "reporter")
    }

    pub fn set_reporter(&self, txn: &mut TransactionMut, reporter: Option<&str>) {
        self.y_task.try_update(txn, "reporter", reporter);
    }

    pub fn get_status<T: ReadTxn>(&self, txn: &T) -> Result<Option<Arc<str>>> {
        self.get_optional_string(txn, "status")
    }

    pub fn set_status(&self, txn: &mut TransactionMut, status: Option<&str>) {
        self.y_task.try_update(txn, "status", status);
    }

    pub fn get_status_time<T: ReadTxn>(&self, txn: &T) -> Result<Option<i64>> {
        self.get_optional_number(txn, "statusTime")
    }

    pub fn set_status_time(&self, txn: &mut TransactionMut, status_time: Option<i64>) {
        self.y_task.try_update(txn, "statusTime", status_time);
    }
}

#[cfg(test)]
mod tests {
    use yrs::types::ToJson;
    use yrs::{updates::decoder::Decode, Doc, StateVector, Transact, Update};

    use super::*;

    #[test]
    fn it_works() {
        let doc = Doc::new();
        let y_graph = {
            let mut txn = doc.transact_mut();
            YGraphProxy::new(&mut txn)
        };

        {
            let mut txn = doc.transact_mut();
            y_graph.set(
                &mut txn,
                &Task {
                    id: "1".into(),
                    num: "1".into(),
                    name: "Task 1".into(),
                    children: vec!["2".into()],
                    assignee: Some("assigneed@gmail.com".into()),
                    reporter: Some("reporter@gmail.com".into()),
                    status: Some("Done".into()),
                    status_time: Some(23),
                },
            );

            // let graph = y_graph.json(&txn);
            // println!("{:?}", graph);
        }

        let sv: StateVector = doc.transact().state_vector();
        let update = Update::decode_v2(
            &doc.transact()
                .encode_state_as_update_v2(&StateVector::default()),
        );
        println!("1st update all: {update:?}");

        {
            let mut txn = doc.transact_mut();
            y_graph.set(
                &mut txn,
                &Task {
                    id: "1".into(),
                    num: "1".into(),
                    name: "Task 1-edited".into(),
                    children: vec!["2".into(), "3".into()],
                    assignee: Some("assigneed@gmail.com".into()),
                    reporter: Some("reporter@gmail.com".into()),
                    status: Some("Done".into()),
                    status_time: Some(23),
                },
            );

            // let graph = y_graph.json(&txn);
            // println!("{:?}", graph);
        }

        // {
        //     let mut txn = doc.transact_mut();
        //     let y_task = y_graph.get(&txn, "1").unwrap();
        //     y_task.set_name(&mut txn, "Task 1-edited");
        // }
        {
            let txn = doc.transact();
            let update = Update::decode_v2(&txn.encode_state_as_update_v2(&StateVector::default()));
            println!("2nd update all: {update:?}");
            let update = Update::decode_v2(&txn.encode_state_as_update_v2(&sv));
            println!("2nd update incremental: {update:?}");
        }

        let txn = doc.transact();
        let y_task = match y_graph.get(&txn, "1") {
            Ok(y_task) => y_task,
            Err(e) => {
                panic!("Error getting task: {e}");
            }
        };
        println!("{:?}", y_task.y_task.to_json(&txn));
    }
}
