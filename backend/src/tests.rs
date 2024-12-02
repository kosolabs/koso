use std::{mem, net::SocketAddr, time::Duration};

use crate::{
    api::{
        collab::msg_sync::{self, MSG_SYNC, MSG_SYNC_REQUEST, MSG_SYNC_RESPONSE, MSG_SYNC_UPDATE},
        google::test_utils::{encode_token, testonly_key_set, Claims, KID_1, PEM_1},
        model::{CreateProject, Project, ProjectExport, Task},
        yproxy::YDocProxy,
    },
    server::{self, Config},
};
use anyhow::{anyhow, Result};
use axum::http::HeaderValue;
use futures::{stream::FusedStream, SinkExt, StreamExt};
use reqwest::{Client, Response, StatusCode};
use serde_json::Value;
use sqlx::PgPool;
use tokio::{
    net::TcpStream,
    sync::{
        oneshot::{self, channel},
        Mutex,
    },
    task::JoinHandle,
};
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
    updates::{
        decoder::{Decode as _, DecoderV1},
        encoder::Encode,
    },
    ReadTxn, StateVector, Update,
};

#[test_log::test(sqlx::test)]
fn kosolib_link_test() {
    assert_eq!(kosolib::add(1, 1), 2);
}

#[test_log::test(sqlx::test)]
async fn database_connectivity_test(pool: PgPool) -> sqlx::Result<()> {
    let users: Vec<(String,)> = sqlx::query_as("SELECT email FROM users")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(users.len(), 0);
    Ok(())
}

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[test_log::test(sqlx::test)]
async fn api_test(pool: PgPool) -> sqlx::Result<()> {
    let (server, addr) = start_server(&pool).await;
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

    let claims = Claims::default();
    let token: String = encode_token(&claims, KID_1, PEM_1).unwrap();
    // Log in
    {
        let res = client
            .post(format!("http://{addr}/api/auth/login"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::OK);
        set_user_invited(&claims.email, &pool).await.unwrap();
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
        set_user_invited(&claims.email, &pool).await.unwrap();
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

    server.shutdown_and_wait().await.unwrap();
    Ok(())
}

#[test_log::test(sqlx::test)]
async fn not_invite_user(pool: PgPool) -> sqlx::Result<()> {
    let (server, addr) = start_server(&pool).await;
    let client = Client::default();

    let claims = Claims::default();
    let token: String = encode_token(&claims, KID_1, PEM_1).unwrap();

    // Log in a user
    {
        let res = client
            .post(format!("http://{addr}/api/auth/login"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::OK);
    };

    // Check that create projects rejects the not invited user.
    {
        let res = client
            .post(format!("http://{addr}/api/projects"))
            .bearer_auth(&token)
            .header("Content-Type", "application/json")
            .body("{\"name\":\"Test Project\"}")
            .send()
            .await
            .expect("Failed to send request.");
        assert_not_invited(res).await;
    }

    // Check that list projects rejects the not invited user.
    {
        let res = client
            .get(format!("http://{addr}/api/projects"))
            .bearer_auth(&token)
            .header("Content-Type", "application/json")
            .send()
            .await
            .expect("Failed to send request.");
        assert_not_invited(res).await;
    }

    server.shutdown_and_wait().await.unwrap();
    Ok(())
}

async fn assert_not_invited(res: Response) {
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
    let error: Value = serde_json::from_str(res.text().await.unwrap().as_str()).unwrap();
    let error = error.as_object().unwrap();
    assert_eq!(error.get("status").unwrap().as_i64().unwrap(), 403);
    let details = error.get("details").unwrap().as_array().unwrap();
    assert_eq!(details.len(), 1);
    let detail = details.first().unwrap().as_object().unwrap();
    assert_eq!(
        detail.get("reason").unwrap().as_str().unwrap(),
        "NOT_INVITED"
    );
    assert!(!detail.get("msg").unwrap().as_str().unwrap().is_empty());
}

#[test_log::test(sqlx::test)]
async fn create_and_delete_project(pool: PgPool) -> sqlx::Result<()> {
    let (mut server, addr) = start_server(&pool).await;
    let client = Client::default();

    let token = login(&client, &addr, &pool).await.unwrap();

    let project = create_project(&client, &addr, &token, "A Project to Delete")
        .await
        .unwrap();
    assert_eq!(project.name, "A Project to Delete");
    assert_eq!(project.deleted_on, None);

    let project = delete_project(&client, &addr, &token, &project.project_id)
        .await
        .unwrap();
    assert_eq!(project.name, "A Project to Delete");
    assert_ne!(project.deleted_on, None);

    server.start_shutdown().await;
    server.wait_for_shutdown().await.unwrap();
    Ok(())
}

#[test_log::test(sqlx::test)]
async fn ws_test(pool: PgPool) -> sqlx::Result<()> {
    let (mut server, addr) = start_server(&pool).await;
    let client = Client::default();

    let claims = Claims::default();
    let token: String = encode_token(&claims, KID_1, PEM_1).unwrap();

    // Log in and share the project
    let project_id = {
        let res = client
            .post(format!("http://{addr}/api/auth/login"))
            .bearer_auth(&token)
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::OK);
        set_user_invited(&claims.email, &pool).await.unwrap();

        let project_id = {
            let res = client
                .post(format!("http://{addr}/api/projects"))
                .bearer_auth(&token)
                .header("Content-Type", "application/json")
                .body("{\"name\":\"Test Project\"}")
                .send()
                .await
                .expect("Failed to send request.");
            assert_eq!(res.status(), StatusCode::OK);
            let project: Value = serde_json::from_str(res.text().await.unwrap().as_str()).unwrap();
            let project = project.as_object().unwrap();
            project
                .get("projectId")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string()
        };

        let res = client
            .patch(format!("http://{addr}/api/projects/{project_id}/users"))
            .bearer_auth(&token)
            .header("Content-Type", "application/json")
            .body(format!(
                "{{\"projectId\":\"{project_id}\", \"addEmails\":[\"{}\"], \"removeEmails\":[]}}",
                claims.email
            ))
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::OK);

        project_id
    };

    // Test that unauthenticated users are rejected.
    {
        let req = format!("ws://{addr}/api/ws/projects/{project_id}")
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
        let close = next_with_timeout(&mut socket).await.unwrap().unwrap();
        let Message::Close(Some(close)) = close else {
            panic!("Expected close frame, got: {close:?}");
        };
        assert_eq!(close.code, CloseCode::Iana(3000));
        assert_eq!(close.reason, "Unauthorized.");
        futures::SinkExt::close(&mut socket).await.unwrap();
        assert!(next_with_timeout(&mut socket).await.unwrap().is_none());
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
    let ydoc_1 = YDocProxy::new();

    let (mut socket_2, response) = tokio_tungstenite::connect_async(req.clone()).await.unwrap();
    let socket_2 = &mut socket_2;
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    let ydoc_2 = YDocProxy::new();
    {
        let mut txn_2 = ydoc_2.transact_mut();
        ydoc_2.set(
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
                url: None,
                kind: None,
            },
        );
        ydoc_2.set(
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
                url: None,
                kind: None,
            },
        );
    }

    // Read the initial sync_request.
    assert_eq!(read_sync_request(socket_1).await, StateVector::default());
    // Send our own sync request
    socket_1
        .send(Message::Binary(msg_sync::sync_request(
            &ydoc_1.transact().state_vector(),
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
            &ydoc_2
                .transact()
                .encode_state_as_update_v2(&StateVector::default()),
        )))
        .await
        .unwrap();

    // Read the broadcast update applied by socket_2.
    let sync_update = read_sync_update(socket_1).await;
    ydoc_1.transact_mut().apply_update(sync_update).unwrap();
    assert_eq!(
        ydoc_1.to_graph(&ydoc_1.transact()).unwrap(),
        ydoc_2.to_graph(&ydoc_2.transact()).unwrap()
    );

    // Open a third socket and verify the sync
    let (mut socket_3, response) = tokio_tungstenite::connect_async(req.clone()).await.unwrap();
    let socket_3 = &mut socket_3;
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    let ydoc_3 = YDocProxy::new();
    assert_eq!(
        read_sync_request(socket_3).await,
        ydoc_1.transact().state_vector()
    );
    // Send our own sync request
    socket_3
        .send(Message::Binary(msg_sync::sync_request(
            &ydoc_3.transact().state_vector(),
        )))
        .await
        .unwrap();
    let sync_response = read_sync_response(socket_3).await;
    ydoc_3.transact_mut().apply_update(sync_response).unwrap();
    assert_eq!(
        ydoc_1.to_graph(&ydoc_1.transact()).unwrap(),
        ydoc_3.to_graph(&ydoc_3.transact()).unwrap()
    );

    // Everything is up to date, subsequent syncs should yield empty updates.
    socket_3
        .send(Message::Binary(msg_sync::sync_request(
            &ydoc_3.transact().state_vector(),
        )))
        .await
        .unwrap();
    assert_eq!(read_sync_response(socket_3).await, Update::default());

    // Export the project.
    {
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
        let mut txn = ydoc_1.transact_mut();
        ydoc_1.set(
            &mut txn,
            &Task {
                id: format!("id{i}"),
                num: format!("{i}"),
                name: format!("Task {i}"),
                children: vec![],
                assignee: None,
                reporter: None,
                status: None,
                status_time: None,
                url: None,
                kind: None,
            },
        );
        let update = txn.encode_update_v2();
        socket_1
            .send(Message::Binary(msg_sync::sync_update(&update)))
            .await
            .unwrap();
    }
    for _ in 0..10 {
        ydoc_2
            .transact_mut()
            .apply_update(read_sync_update(socket_2).await)
            .unwrap();
        ydoc_3
            .transact_mut()
            .apply_update(read_sync_update(socket_3).await)
            .unwrap();
    }
    assert_eq!(
        ydoc_1.to_graph(&ydoc_1.transact()).unwrap(),
        ydoc_3.to_graph(&ydoc_3.transact()).unwrap()
    );
    assert_eq!(
        ydoc_1.to_graph(&ydoc_1.transact()).unwrap(),
        ydoc_2.to_graph(&ydoc_2.transact()).unwrap()
    );

    // Test other valid and invalid message types.
    {
        let (mut socket, response) = tokio_tungstenite::connect_async(req.clone()).await.unwrap();
        let socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>> = &mut socket;
        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
        assert_eq!(
            read_sync_request(socket).await,
            ydoc_1.transact().state_vector()
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
            ydoc_1.transact().state_vector()
        );
        sockets.push(socket)
    }

    // And then verify the next one is rejected.
    let (mut socket_4, response) = tokio_tungstenite::connect_async(req.clone()).await.unwrap();
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    let close = next_with_timeout(&mut socket_4).await.unwrap().unwrap();
    let Message::Close(Some(close)) = close else {
        panic!("Expected overload close, got: {close:?}");
    };
    assert_eq!(close.code, CloseCode::Again);
    assert_eq!(close.reason, "Too many active clients.");
    futures::SinkExt::close(&mut socket_4).await.unwrap();
    // Validate the socket is terminated
    assert!(next_with_timeout(&mut socket_4).await.unwrap().is_none());
    assert!(socket_4.is_terminated());

    // Close the sockets.
    close_socket(socket_1).await;
    close_socket_without_details(socket_3).await;
    let mut sockets = sockets.iter_mut();
    for _ in 0..48 {
        close_socket(sockets.next().unwrap()).await;
    }

    server.start_shutdown().await;
    // The server will close the client, but we need to respond.
    respond_closed_socket(socket_2).await;
    for socket in sockets {
        respond_closed_socket(socket).await;
    }
    server.wait_for_shutdown().await.unwrap();
    Ok(())
}

#[test_log::test(sqlx::test)]
async fn plugin_test(pool: PgPool) -> Result<()> {
    let (server, addr) = start_server(&pool).await;
    let client = Client::default();

    let res = client
        .post(format!("http://{addr}/plugins/github/poll"))
        .send()
        .await
        .expect("Failed to poll.");
    assert_eq!(res.status(), StatusCode::OK);

    server.shutdown_and_wait().await.unwrap();

    Ok(())
}

async fn read_sync_request(socket: &mut Socket) -> StateVector {
    let sync_request = next_with_timeout(socket).await.unwrap().unwrap();

    let Message::Binary(sync_request) = sync_request else {
        panic!("Expected binary sync_request, got: {sync_request:?}");
    };
    assert!(!sync_request.is_empty());
    let mut decoder = DecoderV1::from(sync_request.as_slice());
    match decoder.read_var().unwrap() {
        MSG_SYNC => match decoder.read_var().unwrap() {
            MSG_SYNC_REQUEST => StateVector::decode_v1(decoder.read_buf().unwrap()).unwrap(),
            invalid_type => panic!("Invalid sync type: {invalid_type}"),
        },
        invalid_type => panic!("Invalid message protocol type: {invalid_type}"),
    }
}

async fn read_sync_response(socket: &mut Socket) -> Update {
    let sync_response = next_with_timeout(socket).await.unwrap().unwrap();
    let Message::Binary(sync_response) = sync_response else {
        panic!("Expected binary sync_response, got: {sync_response:?}");
    };
    assert!(!sync_response.is_empty());
    let mut decoder = DecoderV1::from(sync_response.as_slice());
    match decoder.read_var().unwrap() {
        MSG_SYNC => match decoder.read_var().unwrap() {
            MSG_SYNC_RESPONSE => Update::decode_v2(decoder.read_buf().unwrap()).unwrap(),
            invalid_type => panic!("Invalid sync type: {invalid_type}"),
        },
        invalid_type => panic!("Invalid message protocol type: {invalid_type}"),
    }
}

async fn read_sync_update(socket: &mut Socket) -> Update {
    let sync_update = next_with_timeout(socket).await.unwrap().unwrap();
    let Message::Binary(sync_update) = sync_update else {
        panic!("Expected binary sync_update, got: {sync_update:?}");
    };
    assert!(!sync_update.is_empty());
    let mut decoder = DecoderV1::from(sync_update.as_slice());
    match decoder.read_var().unwrap() {
        MSG_SYNC => match decoder.read_var().unwrap() {
            MSG_SYNC_UPDATE => Update::decode_v2(decoder.read_buf().unwrap()).unwrap(),
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
    let close = next_with_timeout(socket).await.unwrap().unwrap();
    let Message::Close(Some(close)) = close else {
        panic!("Expected close frame, got: {close:?}");
    };
    assert_eq!(close.code, CloseCode::Normal);
    assert_eq!(close.reason, "all done");

    // Validate the socket is terminated
    assert!(next_with_timeout(socket).await.unwrap().is_none());
    assert!(socket.is_terminated());
}

async fn close_socket_without_details(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) {
    socket.close(None).await.unwrap();
    // Read the final close message.
    let close = next_with_timeout(socket).await.unwrap().unwrap();
    let Message::Close(None) = close else {
        panic!("Expected close frame, got: {close:?}");
    };

    // Validate the socket is terminated
    assert!(next_with_timeout(socket).await.unwrap().is_none());
    assert!(socket.is_terminated());
}

async fn respond_closed_socket(socket: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) {
    let close = next_with_timeout(socket).await.unwrap().unwrap();
    let Message::Close(Some(close)) = close else {
        panic!("Expected close frame, got: {close:?}");
    };
    assert_eq!(close.code, CloseCode::Restart);
    assert_eq!(close.reason, "The server is shutting down.");
    futures::SinkExt::close(socket).await.unwrap();
    // Validate the socket is terminated
    assert!(next_with_timeout(socket).await.unwrap().is_none());
    assert!(socket.is_terminated());
}

async fn next_with_timeout(socket: &mut Socket) -> Result<Option<Message>> {
    match tokio::time::timeout(Duration::from_secs(22), socket.next()).await {
        Ok(Some(Ok(msg))) => Ok(Some(msg)),
        Ok(None) => Ok(None),
        Ok(Some(Err(e))) => Err(anyhow!("error reading from socket: {e}")),
        Err(e) => Err(anyhow!(
            "Timed out reading from socket after 22 seconds: {e}"
        )),
    }
}

async fn delete_project(
    client: &Client,
    addr: &SocketAddr,
    token: &str,
    project_id: &str,
) -> Result<Project> {
    let res = client
        .delete(format!("http://{addr}/api/projects/{project_id}"))
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .send()
        .await
        .expect("Failed to send request.");
    assert_eq!(res.status(), StatusCode::OK);
    Ok(serde_json::from_str(res.text().await.unwrap().as_str()).unwrap())
}

async fn create_project(
    client: &Client,
    addr: &SocketAddr,
    token: &str,
    name: &str,
) -> Result<Project> {
    let res = client
        .post(format!("http://{addr}/api/projects"))
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .body(format!("{{\"name\":\"{name}\"}}"))
        .send()
        .await
        .expect("Failed to send request.");
    assert_eq!(res.status(), StatusCode::OK);
    Ok(serde_json::from_str(res.text().await.unwrap().as_str()).unwrap())
}

async fn login(client: &Client, addr: &SocketAddr, pool: &PgPool) -> Result<String> {
    let claims = Claims::default();
    let token: String = encode_token(&claims, KID_1, PEM_1).unwrap();

    // Login
    let res = client
        .post(format!("http://{addr}/api/auth/login"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("Failed to send request.");
    assert_eq!(res.status(), StatusCode::OK);
    set_user_invited(&claims.email, pool).await.unwrap();
    Ok(token)
}

async fn set_user_invited(email: &str, pool: &PgPool) -> Result<()> {
    sqlx::query("UPDATE users SET invited=TRUE WHERE email=$1")
        .bind(email)
        .execute(pool)
        .await?;
    Ok(())
}

struct ServerHandle {
    closer: Mutex<oneshot::Sender<()>>,
    serve: JoinHandle<()>,
}

impl ServerHandle {
    async fn shutdown_and_wait(mut self) -> Result<()> {
        self.start_shutdown().await;
        self.wait_for_shutdown().await
    }

    async fn start_shutdown(&mut self) {
        tracing::info!("Sending server shutdown signal...");
        let closer = mem::replace(&mut *self.closer.lock().await, channel::<()>().0);
        closer.send(()).unwrap();
    }

    async fn wait_for_shutdown(self) -> Result<()> {
        match tokio::time::timeout(Duration::from_secs(20), self.serve).await {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow!(
                "Timed out waiting for shutdown after 20 seconds: {e}"
            )),
        }
    }
}

async fn start_server(pool: &PgPool) -> (ServerHandle, SocketAddr) {
    let (closer, close_signal) = channel::<()>();
    let (addr, serve) = server::start_main_server(Config {
        pool: Some(Box::leak(Box::new(pool.clone()))),
        port: Some(0),
        shutdown_signal: Some(close_signal),
        key_set: Some(testonly_key_set().await.unwrap()),
    })
    .await;

    (
        ServerHandle {
            closer: Mutex::new(closer),
            serve,
        },
        addr,
    )
}
