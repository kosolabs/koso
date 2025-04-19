use super::{
    YDocProxy,
    awareness::{AwarenessState, AwarenessUpdate},
    msg_sync::koso_awareness_state,
    notifications,
};
use crate::{
    api::{
        collab::{
            client::{
                CLOSE_ERROR, CLOSE_RESTART, ClientClosure, ClientReceiver, ClientSender, OVERLOADED,
            },
            client_messages::{ClientMessage, ClientMessageReceiver},
            doc_updates::{DocObserver, DocUpdate, GraphObserver},
            msg_sync::sync_request,
            notifications::KosoEvent,
            storage,
            txn_origin::YOrigin,
        },
        model::{ProjectId, User},
    },
    postgres::compact,
};
use anyhow::{Context as _, Result, anyhow};
use async_trait::async_trait;
use sqlx::PgPool;
use std::{
    collections::{HashMap, hash_map::Entry},
    fmt,
    sync::{
        Arc, Weak,
        atomic::{self, Ordering::Relaxed},
    },
};
use tokio::sync::{Mutex, MutexGuard, mpsc::Sender};
use tokio_util::sync::CancellationToken;
use tracing::Instrument;
use yrs::{ReadTxn as _, StateVector, Subscription, Update};

#[derive(Debug)]
enum ProjectInsertionError {
    InitDocError(anyhow::Error),
    Stopped(),
}

struct ProjectsMap {
    map: HashMap<ProjectId, Weak<ProjectState>>,
    stopped: bool,
}

pub(super) struct ProjectsState {
    projects: Mutex<ProjectsMap>,
    process_msg_tx: Sender<ClientMessage>,
    doc_update_tx: Sender<DocUpdate>,
    event_tx: Sender<KosoEvent>,
    pool: &'static PgPool,
    tracker: tokio_util::task::TaskTracker,
}

impl ProjectsState {
    pub(super) fn new(
        process_msg_tx: Sender<ClientMessage>,
        doc_update_tx: Sender<DocUpdate>,
        event_tx: Sender<KosoEvent>,
        pool: &'static PgPool,
        tracker: tokio_util::task::TaskTracker,
    ) -> Self {
        ProjectsState {
            projects: Mutex::new(ProjectsMap {
                map: HashMap::new(),
                stopped: false,
            }),
            process_msg_tx,
            doc_update_tx,
            event_tx,
            pool,
            tracker,
        }
    }

    pub(super) async fn add_and_init_local_client(
        &self,
        project_id: &ProjectId,
    ) -> Result<Arc<ProjectState>> {
        let project = match self.get_or_init(project_id).await {
            Ok((project, _)) => project,
            Err(err) => return Err(anyhow!("{err:?}")),
        };
        Ok(project)
    }

    pub(super) async fn add_and_init_client(
        &self,
        project_id: &ProjectId,
        mut sender: ClientSender,
        receiver: ClientReceiver,
    ) -> Result<()> {
        // Get of insert the project state.
        let (project, sv) = match self.get_or_init(project_id).await {
            Ok(r) => r,
            Err(ProjectInsertionError::InitDocError(err)) => {
                sender.close(CLOSE_ERROR, "Failed to init project.").await;
                return Err(err);
            }
            Err(ProjectInsertionError::Stopped()) => {
                sender
                    .close(
                        CLOSE_RESTART,
                        "Server is shutting down, cannot init project.",
                    )
                    .await;
                return Err(anyhow!("Server is shutting down, cannot init project."));
            }
        };

        // Store the sender side of the socket in the list of clients.
        match project.insert_client(sender).await {
            Err((mut sender, ClientInsertionError::TooManyClients(err))) => {
                sender.close(OVERLOADED, "Too many active clients.").await;
                return Err(anyhow!(err));
            }
            Err((mut sender, ClientInsertionError::DuplicateClient(err))) => {
                sender
                    .close(CLOSE_ERROR, "Unexpected duplicate connection.")
                    .await;
                return Err(anyhow!(err));
            }
            Err((mut sender, ClientInsertionError::Stopped())) => {
                sender
                    .close(CLOSE_RESTART, "Server is shutting down, cannot add client.")
                    .await;
                return Err(anyhow!("Server is shutting down, cannot add client."));
            }
            Ok(_) => (),
        };

        // Send the entire state vector to the client.
        tracing::debug!("Sending sync_request message to client");
        if let Err(e) = project
            .send_msg(&receiver.who, sync_request(&sv))
            .await
            .context("Failed to send sync request message to client")
        {
            project
                .remove_and_close_client(
                    &receiver.who,
                    ClientClosure {
                        code: CLOSE_ERROR,
                        reason: "Failed to send sync request message.",
                        details: format!("Failed to send sync request message: {e:#}"),
                        client_initiated: false,
                    },
                )
                .await;
            return Err(e);
        }

        // Listen for messages on the read side of the socket.
        let handler =
            ClientMessageReceiver::new(Arc::clone(&project), self.process_msg_tx.clone(), receiver);
        self.tracker.spawn(handler.receive_messages_from_client());

        Ok(())
    }

    async fn get_or_init(
        &self,
        project_id: &ProjectId,
    ) -> Result<(Arc<ProjectState>, StateVector), ProjectInsertionError> {
        let project = {
            let mut projects = self.projects.lock().await;
            if projects.stopped {
                return Err(ProjectInsertionError::Stopped());
            }
            match projects.map.entry(project_id.to_string()) {
                Entry::Occupied(mut o) => match o.get().upgrade() {
                    Some(p) => p,
                    None => {
                        let project = self.new_project(project_id);
                        o.insert(Arc::downgrade(&project));
                        project
                    }
                },
                Entry::Vacant(v) => {
                    let project = self.new_project(project_id);
                    v.insert(Arc::downgrade(&project));
                    project
                }
            }
        };

        // Init the doc_box, if necessary and grab the state vector.
        let sv = match ProjectState::init_doc_box(&project).await {
            Ok(sv) => sv,
            Err(err) => return Err(ProjectInsertionError::InitDocError(err)),
        };
        Ok((project, sv))
    }

    fn new_project(&self, project_id: &String) -> Arc<ProjectState> {
        Arc::new(ProjectState {
            project_id: project_id.to_string(),
            clients: Mutex::new(ClientsMap {
                map: HashMap::new(),
                stopped: false,
            }),
            awarenesses: Mutex::new(HashMap::new()),
            doc_box: Mutex::new(None),
            doc_update_tx: self.doc_update_tx.clone(),
            event_tx: self.event_tx.clone(),
            updates: atomic::AtomicUsize::new(0),
            pool: self.pool,
            tracker: self.tracker.clone(),
            stopped_token: CancellationToken::new(),
        })
    }

    pub(super) async fn stop(&self) {
        let res = {
            let mut projects = self.projects.lock().await;
            projects.stopped = true;

            let mut res = Vec::new();
            for project in projects.map.values() {
                if let Some(project) = project.upgrade() {
                    res.push(ProjectState::stop(project));
                }
            }
            res
        };
        futures::future::join_all(res).await;
    }
}

enum ClientInsertionError {
    TooManyClients(String),
    DuplicateClient(String),
    Stopped(),
}

pub(crate) struct ClientsMap {
    map: HashMap<String, ClientSender>,
    stopped: bool,
}

pub(crate) struct ProjectState {
    pub(crate) project_id: ProjectId,
    clients: Mutex<ClientsMap>,
    awarenesses: Mutex<HashMap<String, AwarenessState>>,
    pub(crate) doc_box: Mutex<Option<DocBox>>,
    updates: atomic::AtomicUsize,
    doc_update_tx: Sender<DocUpdate>,
    pub(super) event_tx: Sender<KosoEvent>,
    pool: &'static PgPool,
    tracker: tokio_util::task::TaskTracker,
    pub(super) stopped_token: CancellationToken,
}

impl ProjectState {
    async fn insert_client(
        &self,
        sender: ClientSender,
    ) -> Result<(), (ClientSender, ClientInsertionError)> {
        let mut clients = self.clients.lock().await;
        if clients.stopped {
            return Err((sender, ClientInsertionError::Stopped()));
        }

        const MAX_PROJECT_CLIENTS: usize = 100;
        if clients.map.len() >= MAX_PROJECT_CLIENTS {
            return Err((
                sender,
                ClientInsertionError::TooManyClients(format!(
                    "Too many ({}) active project clients.",
                    clients.map.len(),
                )),
            ));
        }
        match clients.map.entry(sender.who.clone()) {
            Entry::Occupied(entry) => {
                tracing::error!("Unexpectedly, client already exists: {entry:?}");
                return Err((
                    sender,
                    ClientInsertionError::DuplicateClient(format!(
                        "Unexpectedly, client already exists: {entry:?}",
                    )),
                ));
            }
            Entry::Vacant(entry) => entry.insert(sender),
        };
        Ok(())
    }

    async fn init_doc_box(project: &Arc<ProjectState>) -> Result<StateVector> {
        let mut doc_box = project.doc_box.lock().await;
        if let Some(doc_box) = doc_box.as_ref() {
            return Ok(doc_box.ydoc.transact().state_vector());
        }

        // Load the doc if it wasn't already loaded by another client.
        tracing::debug!("Initializing new YDoc");
        let (ydoc, update_count) = storage::load_doc(&project.project_id, project.pool).await?;
        tracing::debug!("Initialized new YDoc with {update_count} updates");
        project.updates.store(update_count, Relaxed);

        // Attach observers to the doc.
        let subs = vec![
            Self::create_doc_observer(project, &ydoc)?,
            Self::create_graph_observer(project, &ydoc),
            Self::create_deep_graph_observer(project, &ydoc),
        ];

        let db = DocBox { ydoc, subs };
        let sv = db.ydoc.transact().state_vector();
        *doc_box = Some(db);
        Ok(sv)
    }

    /// Persist and broadcast update events by subscribing to the callback.
    fn create_doc_observer(project: &Arc<ProjectState>, doc: &YDocProxy) -> Result<Subscription> {
        let observer = DocObserver::new(project.doc_update_tx.clone(), project.tracker.clone());
        let project = Arc::downgrade(project);
        doc.observe_update_v2(move |txn, update| {
            let Some(project) = project.upgrade() else {
                // This will never happen because the observer is invoked syncronously in
                // ProjectState.apply_update while holding a strong reference to the project.
                tracing::error!(
                    "handle_doc_update_v2_event but weak project reference was destroyed"
                );
                return;
            };

            project.updates.fetch_add(1, Relaxed);
            observer.handle_doc_update_v2_event(project, txn, update);
        })
        .context("Failed to create observer")
    }

    fn create_graph_observer(project: &Arc<ProjectState>, doc: &YDocProxy) -> Subscription {
        let observer: GraphObserver = GraphObserver::new(project.tracker.clone());
        let project = Arc::downgrade(project);
        doc.observe_graph(move |txn, event| {
            let Some(project) = project.upgrade() else {
                // This will never happen because the observer is invoked syncronously in
                // ProjectState.apply_update while holding a strong reference to the project.
                tracing::error!(
                    "handle_graph_update_event but weak project reference was destroyed"
                );
                return;
            };

            observer.handle_graph_update_event(project, txn, event)
        })
    }

    fn create_deep_graph_observer(project: &Arc<ProjectState>, doc: &YDocProxy) -> Subscription {
        let project = Arc::downgrade(project);
        doc.observe_deep_graph(move |txn, events| {
            let Some(project) = project.upgrade() else {
                // This will never happen because the observer is invoked syncronously in
                // ProjectState.apply_update while holding a strong reference to the project.
                tracing::error!(
                    "handle_deep_graph_update_events but weak project reference was destroyed"
                );
                return;
            };

            notifications::handle_deep_graph_update_events(txn, events, project);
        })
    }

    pub(super) async fn encode_state_as_update(&self, sv: &StateVector) -> Result<Vec<u8>> {
        let update = DocBox::doc_or_error(self.doc_box.lock().await.as_ref())?
            .ydoc
            .transact()
            .encode_state_as_update_v2(sv);
        Ok(update)
    }
    pub(super) async fn apply_doc_update(&self, origin: YOrigin, update: Update) -> Result<()> {
        DocBox::doc_or_error(self.doc_box.lock().await.as_ref())?
            .ydoc
            .transact_mut_with(origin.as_origin()?)
            .apply_update(update)
            .context("Failed to apply doc update")
    }

    pub(super) async fn broadcast_msg(&self, data: Vec<u8>, exclude_who: Option<&String>) {
        let mut clients = self.clients.lock().await;
        if clients.stopped {
            return;
        }

        tracing::debug!("Broadcasting to {} clients", clients.map.len());
        let mut results = Vec::new();
        for client in clients.map.values_mut() {
            match exclude_who {
                Some(exclude_who) if client.who == *exclude_who => {}
                _ => results.push(client.send(data.to_owned())),
            }
        }
        let res = futures::future::join_all(results).await;
        tracing::debug!("Finished broadcasting: {res:?}");
    }

    pub(super) async fn send_msg(&self, to_who: &String, data: Vec<u8>) -> Result<()> {
        let mut clients = self.clients.lock().await;
        let Some(client) = clients.map.get_mut(to_who) else {
            return Err(anyhow!("Unexpectedly found no client to send to"));
        };
        client.send(data).await.context("Failed to send to client")
    }

    pub(super) async fn remove_and_close_client(&self, who: &String, closure: ClientClosure) {
        let (client, remaining_clients) = {
            let clients = &mut self.clients.lock().await;
            if clients.stopped {
                tracing::debug!("Tryed to remove client during shutdown");
                return;
            }
            (clients.map.remove(who), clients.map.len())
        };
        tracing::debug!(
            "Removed client. {} clients remain. Reason: {}",
            remaining_clients,
            closure.details,
        );

        self.awarenesses.lock().await.remove(who);
        self.broadcast_awarenesses().await;

        match client {
            Some(mut client) => {
                if !closure.client_initiated {
                    client.close(closure.code, closure.reason).await;
                } else {
                    client.close_sender().await;
                }
            }
            None => tracing::warn!(
                "Tried to remove client ({who}) in project {}, but it was already gone.",
                self.project_id
            ),
        }
    }

    #[tracing::instrument()]
    async fn stop(project: Arc<ProjectState>) {
        let mut clients = project.clients.lock().await;
        clients.stopped = true;
        project.stopped_token.cancel();

        tracing::debug!("Closing {} project clients", clients.map.len());
        let mut res = Vec::new();
        for (_, mut client) in clients.map.drain() {
            res.push(async move {
                client
                    .close(CLOSE_RESTART, "The server is shutting down.")
                    .await;
            });
        }
        futures::future::join_all(res).await;
    }

    pub(super) async fn update_awareness(&self, who: &str, user: &User, update: AwarenessUpdate) {
        let state = update.into_state(user.clone());
        self.awarenesses.lock().await.insert(who.into(), state);
        self.broadcast_awarenesses().await;
    }

    async fn broadcast_awarenesses(&self) {
        let Ok(state) =
            serde_json::to_string(&self.awarenesses.lock().await.values().collect::<Vec<_>>())
        else {
            tracing::warn!("Failed to serialize awarenesses");
            return;
        };
        let msg = koso_awareness_state(&state);
        self.broadcast_msg(msg, None).await;
    }
}

impl fmt::Debug for ProjectState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.project_id)
    }
}

impl fmt::Display for ProjectState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.project_id)
    }
}

#[async_trait]
impl DocBoxProvider for ProjectState {
    async fn get_doc_box(&self) -> MutexGuard<'_, Option<DocBox>> {
        self.doc_box.lock().await
    }
}

impl Drop for ProjectState {
    fn drop(&mut self) {
        tracing::debug!(
            "Last client disconnected, destroying project state: {}",
            self.project_id
        );

        // Avoid compaction on shutdown to speed shutdown along
        // and avoid the burst of compacting many projects at once.
        if let Ok(clients) = self.clients.try_lock() {
            if clients.stopped {
                tracing::trace!("Skipping compacting, shutting down");
                return;
            }
        }

        let updates: usize = self.updates.load(Relaxed);
        if updates > 10 {
            self.tracker
                .spawn(compact(self.pool, self.project_id.clone()).in_current_span());
        } else {
            tracing::debug!("Skipping compacting, only {updates} updates exist")
        }
    }
}

pub(crate) struct DocBox {
    pub(crate) ydoc: YDocProxy,
    /// Subscription to observe changes to doc.
    #[allow(dead_code)]
    pub(crate) subs: Vec<Subscription>,
}

impl DocBox {
    pub(crate) fn doc_or_error(doc_box: Option<&DocBox>) -> Result<&DocBox> {
        match doc_box {
            Some(db) => Ok(db),
            None => Err(anyhow!("DocBox is absent")),
        }
    }
}

#[async_trait]
pub(crate) trait DocBoxProvider: Sync + Send {
    async fn get_doc_box(&self) -> MutexGuard<'_, Option<DocBox>>;
}
