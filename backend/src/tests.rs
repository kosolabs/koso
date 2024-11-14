use crate::{
    api::{
        collab::msg_sync::{self, MSG_SYNC, MSG_SYNC_REQUEST, MSG_SYNC_RESPONSE, MSG_SYNC_UPDATE},
        google::test_utils::{encode_token, testonly_key_set, Claims, KID_1, PEM_1},
        model::{CreateProject, Project, ProjectExport, Task},
        yproxy::YGraphProxy,
    },
    server::{self, Config},
};
use axum::http::HeaderValue;
use futures::{stream::FusedStream, SinkExt, StreamExt};
use reqwest::{Client, StatusCode};
use serde_json::Value;
use sqlx::PgPool;
use tokio::{net::TcpStream, sync::oneshot::channel};
use tokio_tungstenite::{
    tungstenite::{
        protocol::{frame::coding::CloseCode, CloseFrame},
        Message,
    },
    MaybeTlsStream, WebSocketStream,
};
use tungstenite::client::IntoClientRequest;
use yrs::{
    encoding::read::Read as _,
    types::ToJson,
    updates::{
        decoder::{Decode as _, DecoderV1},
        encoder::Encode,
    },
    Doc, Map, MapPrelim, ReadTxn, StateVector, Transact, Update, WriteTxn,
};

#[test_log::test(sqlx::test)]
async fn basic_test(pool: PgPool) -> sqlx::Result<()> {
    let projects: Vec<(String,)> = sqlx::query_as("SELECT project_id FROM projects")
        .fetch_all(&pool)
        .await
        .unwrap();
    tracing::info!("test");
    assert_eq!(projects.len(), 1);
    assert_eq!(projects.first().unwrap().0, "koso-staging");
    Ok(())
}

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[test_log::test(sqlx::test)]
async fn api_test(pool: PgPool) -> sqlx::Result<()> {
    let (closer, close_signal) = channel::<()>();
    let (addr, serve) = server::start_main_server(Config {
        pool: Some(Box::leak(Box::new(pool))),
        port: Some(0),
        shutdown_signal: Some(close_signal),
        key_set: Some(testonly_key_set().await.unwrap()),
    })
    .await;
    let client = Client::default();

    // Health check
    {
        let res = client
            .get(format!("http://{addr}/healthz"))
            .send()
            .await
            .expect("Failed to check healthz.");
        assert_eq!(res.status(), StatusCode::OK);
    }

    let token: String = encode_token(&Claims::default(), KID_1, PEM_1).unwrap();
    // Log in
    {
        let res = client
            .post(format!("http://{addr}/api/auth/login"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::OK);
    }

    // Log in a second user
    const OTHER_USER_EMAIL: &str = "other-user@koso.app";
    {
        let claims = Claims {
            email: OTHER_USER_EMAIL.to_string(),
            ..Claims::default()
        };

        let token: String = encode_token(&claims, KID_1, PEM_1).unwrap();
        let res = client
            .post(format!("http://{addr}/api/auth/login"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::OK);
    }

    // Try a request without any credentials attached.
    {
        let res = client
            .get(format!("http://{addr}/api/projects"))
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    // Create a project
    let project_name = "Test Project1";
    let project_id = {
        let res = client
            .post(format!("http://{addr}/api/projects"))
            .bearer_auth(&token)
            .header("Content-Type", "application/json")
            .body(format!("{{\"name\":\"{project_name}\"}}"))
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::OK);
        let project: Value = serde_json::from_str(res.text().await.unwrap().as_str()).unwrap();
        let project = project.as_object().unwrap();
        assert_eq!(project.get("name").unwrap().as_str().unwrap(), project_name);
        project
            .get("projectId")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string()
    };

    // Get the project
    {
        let res = client
            .get(format!("http://{addr}/api/projects/{project_id}"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::OK);
        let project: Value = serde_json::from_str(res.text().await.unwrap().as_str()).unwrap();
        let project = project.as_object().unwrap();
        assert_eq!(
            project.get("projectId").unwrap().as_str().unwrap(),
            project_id
        );
        assert_eq!(project.get("name").unwrap().as_str().unwrap(), project_name);
    }

    // Update a project
    let project_name = {
        let project_name = "Updated test name 1";
        let res = client
            .patch(format!("http://{addr}/api/projects/{project_id}"))
            .bearer_auth(&token)
            .header("Content-Type", "application/json")
            .body(format!(
                "{{\"name\":\"{project_name}\", \"projectId\":\"{project_id}\"}}"
            ))
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::OK);
        let project: Value = serde_json::from_str(res.text().await.unwrap().as_str()).unwrap();
        let project = project.as_object().unwrap();
        assert_eq!(project.get("name").unwrap().as_str().unwrap(), project_name);
        assert_eq!(
            project.get("projectId").unwrap().as_str().unwrap(),
            project_id
        );
        project_name
    };

    // Update project permissions
    {
        let res = client
            .patch(format!(
                "http://{addr}/api/projects/{project_id}/users"
            ))
            .bearer_auth(&token)
            .header("Content-Type", "application/json")
            .body(format!(
                "{{\"projectId\":\"{project_id}\", \"addEmails\":[\"{OTHER_USER_EMAIL}\"], \"removeEmails\":[\"does-not-exist@koso.app\"]}}"
            ))
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::OK);
    };

    // List the projects.
    {
        let res = client
            .get(format!("http://{addr}/api/projects"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::OK);
        let projects: Value = serde_json::from_str(res.text().await.unwrap().as_str()).unwrap();
        let projects = projects.as_array().unwrap();
        assert_eq!(projects.len(), 1);
        let project = projects.first().unwrap().as_object().unwrap();
        assert_eq!(
            project.get("projectId").unwrap().as_str().unwrap(),
            project_id
        );
        assert_eq!(project.get("name").unwrap().as_str().unwrap(), project_name);
    }

    // List the project's users.
    {
        let res = client
            .get(format!("http://{addr}/api/projects/{project_id}/users"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::OK);
        let users: Value = serde_json::from_str(res.text().await.unwrap().as_str()).unwrap();
        let users = users.as_array().unwrap();
        assert_eq!(users.len(), 2);
    }

    // List the  users.
    {
        let res = client
            .get(format!("http://{addr}/api/users"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::OK);
        let users: Value = serde_json::from_str(res.text().await.unwrap().as_str()).unwrap();
        let users = users.as_array().unwrap();
        assert_eq!(users.len(), 2);
    }

    // Initated server shutdown.
    tracing::info!("Sending server shutdown signal...");
    closer.send(()).unwrap();
    // Finally, the server should gracefully stop.
    serve.await.unwrap();

    Ok(())
}

#[test_log::test(sqlx::test)]
async fn ws_test(pool: PgPool) -> sqlx::Result<()> {
    let (closer, close_signal) = channel::<()>();
    let (addr, serve) = server::start_main_server(Config {
        pool: Some(Box::leak(Box::new(pool))),
        port: Some(0),
        shutdown_signal: Some(close_signal),
        key_set: Some(testonly_key_set().await.unwrap()),
    })
    .await;

    let project_id = "koso-staging";
    let token: String = encode_token(
        &Claims {
            email: "leonhard.kyle@gmail.com".to_string(),
            ..Default::default()
        },
        KID_1,
        PEM_1,
    )
    .unwrap();

    // Test that unauthenticated users are rejected.
    {
        let req = format!("ws://{addr}/api/ws/projects/koso-staging")
            .into_client_request()
            .unwrap();
        let err = tokio_tungstenite::connect_async(req).await.unwrap_err();
        let tokio_tungstenite::tungstenite::error::Error::Http(response) = err else {
            panic!("Expected http error, got: {err:?}");
        };
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // Test that authenticated but unauthorized users are rejected.
    {
        let mut req = format!("ws://{addr}/api/ws/projects/{project_id}")
            .into_client_request()
            .unwrap();
        let claims = Claims {
            email: "unauthorized-users@koso.app".to_string(),
            ..Default::default()
        };
        let token = encode_token(&claims, KID_1, PEM_1).unwrap();
        req.headers_mut().insert(
            "authorization",
            HeaderValue::from_str(format!("Bearer {token}").as_str()).unwrap(),
        );
        let (mut socket, response) = tokio_tungstenite::connect_async(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
        let close = socket.next().await.unwrap().unwrap();
        let Message::Close(Some(close)) = close else {
            panic!("Expected close frame, got: {close:?}");
        };
        assert_eq!(close.code, CloseCode::Iana(3000));
        assert_eq!(close.reason, "Unauthorized.");
        futures::SinkExt::close(&mut socket).await.unwrap();
        assert!(socket.next().await.is_none());
        assert!(socket.is_terminated());
    }

    // Test opening and closing sockets.
    {
        let mut req = format!("ws://{addr}/api/ws/projects/{project_id}")
            .into_client_request()
            .unwrap();
        req.headers_mut().insert(
            "authorization",
            HeaderValue::from_str(format!("Bearer {token}").as_str()).unwrap(),
        );

        let (mut socket, response) = tokio_tungstenite::connect_async(req.clone()).await.unwrap();
        let socket = &mut socket;
        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
        assert_eq!(read_sync_request(socket).await, StateVector::default());
        close_socket(socket).await;

        // Abruptly close the socket, this'll trigger the error handling in ClientMessageReceiver
        let (_socket, response) = tokio_tungstenite::connect_async(req.clone()).await.unwrap();
        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    }

    // Finally, run through a valid websocket interaction.
    let mut req = format!("ws://{addr}/api/ws/projects/{project_id}")
        .into_client_request()
        .unwrap();
    req.headers_mut().insert(
        "authorization",
        HeaderValue::from_str(format!("Bearer {token}").as_str()).unwrap(),
    );

    let (mut socket_1, response) = tokio_tungstenite::connect_async(req.clone()).await.unwrap();
    let socket_1 = &mut socket_1;
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    let doc_1: Doc = Doc::new();
    {
        let mut txn_1 = doc_1.transact_mut();
        YGraphProxy::new(&mut txn_1);
    }

    let (mut socket_2, response) = tokio_tungstenite::connect_async(req.clone()).await.unwrap();
    let socket_2 = &mut socket_2;
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    let doc_2: Doc = Doc::new();
    {
        let mut txn_2 = doc_2.transact_mut();
        let y_graph_2 = YGraphProxy::new(&mut txn_2);
        y_graph_2.set(
            &mut txn_2,
            &Task {
                id: "id1".to_string(),
                num: "1".to_string(),
                name: "Task 1".to_string(),
                children: vec!["id2".to_string()],
                assignee: Some("a@koso.app".to_string()),
                reporter: Some("r@koso.app".to_string()),
                status: None,
                status_time: None,
            },
        );
        y_graph_2.set(
            &mut txn_2,
            &Task {
                id: "id2".to_string(),
                num: "2".to_string(),
                name: "Task 2".to_string(),
                children: vec![],
                assignee: Some("a@koso.app".to_string()),
                reporter: Some("r@koso.app".to_string()),
                status: None,
                status_time: None,
            },
        );
    }

    // Read the initial sync_request.
    assert_eq!(read_sync_request(socket_1).await, StateVector::default());
    // Send our own sync request
    socket_1
        .send(Message::Binary(msg_sync::sync_request(
            &doc_1.transact().state_vector(),
        )))
        .await
        .unwrap();
    // Read the sync_response.
    assert_eq!(read_sync_response(socket_1).await, Update::default());
    // Send the sync_response.
    socket_1
        .send(Message::Binary(msg_sync::sync_response(
            &Update::default().encode_v2(),
        )))
        .await
        .unwrap();

    // Read the initial sync_request.
    assert_eq!(read_sync_request(socket_2).await, StateVector::default());
    // Send a sync_response.
    socket_2
        .send(Message::Binary(msg_sync::sync_response(
            &doc_2
                .transact()
                .encode_state_as_update_v2(&StateVector::default()),
        )))
        .await
        .unwrap();

    // Read the broadcast update applied by socket_2.
    let sync_update = read_sync_update(socket_1).await;
    doc_1.transact_mut().apply_update(sync_update).unwrap();
    assert_eq!(
        doc_1.to_json(&doc_1.transact()),
        doc_2.to_json(&doc_2.transact())
    );

    // Open a third socket and verify the sync
    let (mut socket_3, response) = tokio_tungstenite::connect_async(req.clone()).await.unwrap();
    let socket_3 = &mut socket_3;
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    let doc_3: Doc = Doc::new();
    {
        let mut txn_3 = doc_3.transact_mut();
        YGraphProxy::new(&mut txn_3);
    }
    assert_eq!(
        read_sync_request(socket_3).await,
        doc_1.transact().state_vector()
    );
    // Send our own sync request
    socket_3
        .send(Message::Binary(msg_sync::sync_request(
            &doc_3.transact().state_vector(),
        )))
        .await
        .unwrap();
    let sync_response = read_sync_response(socket_3).await;
    doc_3.transact_mut().apply_update(sync_response).unwrap();
    assert_eq!(
        doc_1.to_json(&doc_1.transact()),
        doc_3.to_json(&doc_3.transact())
    );

    // Everything is up to date, subsequent syncs should yield empty updates.
    socket_3
        .send(Message::Binary(msg_sync::sync_request(
            &doc_3.transact().state_vector(),
        )))
        .await
        .unwrap();
    assert_eq!(read_sync_response(socket_3).await, Update::default());

    // Export the project.
    {
        let client = Client::default();
        let res = client
            .get(format!("http://{addr}/api/projects/{project_id}/export"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::OK);
        let export: ProjectExport =
            serde_json::from_str(res.text().await.unwrap().as_str()).unwrap();
        assert_eq!(export.project_id, project_id);

        let create_req = CreateProject {
            name: "Imported project".to_string(),
            project_export: Some(export),
        };
        let res = client
            .post(format!("http://{addr}/api/projects"))
            .bearer_auth(&token)
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&create_req).unwrap())
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::OK);
        let project: Project = serde_json::from_str(res.text().await.unwrap().as_str()).unwrap();
        assert_eq!(project.name, "Imported project");
        assert!(!project.project_id.is_empty());
    }

    // Apply enough updates to trigger compaction on shutdown.
    for i in 0..10 {
        let mut txn = doc_1.transact_mut();
        let graph = txn.get_or_insert_map("graph");
        graph.insert(
            &mut txn,
            format!("new_entry_{i}"),
            MapPrelim::from([("inner", "value1")]),
        );
        let update = txn.encode_update_v2();
        socket_1
            .send(Message::Binary(msg_sync::sync_update(&update)))
            .await
            .unwrap();
    }
    for _ in 0..10 {
        doc_2
            .transact_mut()
            .apply_update(read_sync_update(socket_2).await)
            .unwrap();
        doc_3
            .transact_mut()
            .apply_update(read_sync_update(socket_3).await)
            .unwrap();
    }
    assert_eq!(
        doc_1.to_json(&doc_1.transact()),
        doc_3.to_json(&doc_3.transact())
    );
    assert_eq!(
        doc_1.to_json(&doc_1.transact()),
        doc_2.to_json(&doc_2.transact())
    );

    // Test other valid and invalid message types.
    {
        let (mut socket, response) = tokio_tungstenite::connect_async(req.clone()).await.unwrap();
        let socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>> = &mut socket;
        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
        assert_eq!(
            read_sync_request(socket).await,
            doc_1.transact().state_vector()
        );

        // Send a ping.
        socket.send(Message::Text("".to_string())).await.unwrap();
        // Send some random text, it's discarded.
        socket
            .send(Message::Text("DISCARD_ME".to_string()))
            .await
            .unwrap();
        // Send some invalid binary
        // Invalid protocol type.
        socket.send(Message::Binary(vec![5, 4])).await.unwrap();
        // Invalid sync type.
        socket.send(Message::Binary(vec![0, 5])).await.unwrap();
        // Invalid content.
        socket.send(Message::Binary(vec![0, 1, 0])).await.unwrap();
        socket.send(Message::Binary(vec![0, 1, 1])).await.unwrap();
        socket
            .send(Message::Binary(vec![0, 0, 4, 2, 2, 2, 2]))
            .await
            .unwrap();
        socket
            .send(Message::Binary(vec![0, 1, 4, 2, 2, 2, 2]))
            .await
            .unwrap();

        close_socket(socket).await;
    }

    // Open up enough sockets to hit the limit.
    let mut sockets = Vec::new();
    for _ in 0..97 {
        let (mut socket, response) = tokio_tungstenite::connect_async(req.clone()).await.unwrap();
        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
        assert_eq!(
            read_sync_request(&mut socket).await,
            doc_1.transact().state_vector()
        );
        sockets.push(socket)
    }

    // And then verify the next one is rejected.
    let (mut socket_4, response) = tokio_tungstenite::connect_async(req.clone()).await.unwrap();
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    let close = socket_4.next().await.unwrap().unwrap();
    let Message::Close(Some(close)) = close else {
        panic!("Expected overload close, got: {close:?}");
    };
    assert_eq!(close.code, CloseCode::Again);
    assert_eq!(close.reason, "Too many active clients.");
    futures::SinkExt::close(&mut socket_4).await.unwrap();
    // Validate the socket is terminated
    assert!(socket_4.next().await.is_none());
    assert!(socket_4.is_terminated());

    // Close the sockets.
    close_socket(socket_1).await;
    close_socket_without_details(socket_3).await;
    let mut sockets = sockets.iter_mut();
    for _ in 0..48 {
        close_socket(sockets.next().unwrap()).await;
    }

    // Initated server shutdown.
    tracing::info!("Sending server shutdown signal...");
    closer.send(()).unwrap();
    // The server will close the client, but we need to respond.
    respond_closed_socket(socket_2).await;
    for socket in sockets {
        respond_closed_socket(socket).await;
    }

    // Finally, the server should gracefully stop.
    serve.await.unwrap();
    Ok(())
}

async fn read_sync_request(socket: &mut Socket) -> StateVector {
    let sync_request = socket.next().await.unwrap().unwrap();

    let Message::Binary(sync_request) = sync_request else {
        panic!("Expected binary sync_request, got: {sync_request:?}");
    };
    assert!(!sync_request.is_empty());
    let mut decoder = DecoderV1::from(sync_request.as_slice());
    match decoder.read_var().unwrap() {
        MSG_SYNC => match decoder.read_var().unwrap() {
            MSG_SYNC_REQUEST => {
                return StateVector::decode_v1(decoder.read_buf().unwrap()).unwrap();
            }
            invalid_type => panic!("Invalid sync type: {invalid_type}"),
        },
        invalid_type => panic!("Invalid message protocol type: {invalid_type}"),
    }
}

async fn read_sync_response(socket: &mut Socket) -> Update {
    let sync_response = socket.next().await.unwrap().unwrap();
    let Message::Binary(sync_response) = sync_response else {
        panic!("Expected binary sync_response, got: {sync_response:?}");
    };
    assert!(!sync_response.is_empty());
    let mut decoder = DecoderV1::from(sync_response.as_slice());
    match decoder.read_var().unwrap() {
        MSG_SYNC => match decoder.read_var().unwrap() {
            MSG_SYNC_RESPONSE => {
                return Update::decode_v2(decoder.read_buf().unwrap()).unwrap();
            }
            invalid_type => panic!("Invalid sync type: {invalid_type}"),
        },
        invalid_type => panic!("Invalid message protocol type: {invalid_type}"),
    }
}

async fn read_sync_update(socket: &mut Socket) -> Update {
    let sync_update = socket.next().await.unwrap().unwrap();
    let Message::Binary(sync_update) = sync_update else {
        panic!("Expected binary sync_update, got: {sync_update:?}");
    };
    assert!(!sync_update.is_empty());
    let mut decoder = DecoderV1::from(sync_update.as_slice());
    match decoder.read_var().unwrap() {
        MSG_SYNC => match decoder.read_var().unwrap() {
            MSG_SYNC_UPDATE => {
                return Update::decode_v2(decoder.read_buf().unwrap()).unwrap();
            }
            invalid_type => panic!("Invalid sync type: {invalid_type}"),
        },
        invalid_type => panic!("Invalid message protocol type: {invalid_type}"),
    }
}

async fn close_socket(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) {
    socket
        .close(Some(CloseFrame {
            code: CloseCode::Normal,
            reason: std::borrow::Cow::Borrowed("all done"),
        }))
        .await
        .unwrap();
    // Read the final close message.
    let close = socket.next().await.unwrap().unwrap();
    let Message::Close(Some(close)) = close else {
        panic!("Expected close frame, got: {close:?}");
    };
    assert_eq!(close.code, CloseCode::Normal);
    assert_eq!(close.reason, "all done");

    // Validate the socket is terminated
    assert!(socket.next().await.is_none());
    assert!(socket.is_terminated());
}

async fn close_socket_without_details(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) {
    socket.close(None).await.unwrap();
    // Read the final close message.
    let close = socket.next().await.unwrap().unwrap();
    let Message::Close(None) = close else {
        panic!("Expected close frame, got: {close:?}");
    };

    // Validate the socket is terminated
    assert!(socket.next().await.is_none());
    assert!(socket.is_terminated());
}

async fn respond_closed_socket(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) {
    let close = socket.next().await.unwrap().unwrap();
    let Message::Close(Some(close)) = close else {
        panic!("Expected close frame, got: {close:?}");
    };
    assert_eq!(close.code, CloseCode::Restart);
    assert_eq!(close.reason, "The server is shutting down.");
    futures::SinkExt::close(socket).await.unwrap();
    // Validate the socket is terminated
    assert!(socket.next().await.is_none());
    assert!(socket.is_terminated());
}
