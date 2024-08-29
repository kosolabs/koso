use crate::api::collab::msg_sync::{
    MSG_SYNC, MSG_SYNC_REQUEST, MSG_SYNC_RESPONSE, MSG_SYNC_UPDATE,
};
use crate::{
    api::{
        collab::msg_sync,
        google::test_utils::{encode_token, testonly_key_set, Claims, KID_1, PEM_1},
    },
    server::{self, Config},
};
use axum::http::HeaderValue;
use futures::SinkExt;
use futures::{stream::FusedStream, StreamExt};
use reqwest::{Client, StatusCode};
use serde_json::Value;
use sqlx::PgPool;
use tokio::net::TcpStream;
use tokio::sync::oneshot::channel;
use tokio_tungstenite::tungstenite::{
    protocol::{frame::coding::CloseCode, CloseFrame},
    Message,
};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tungstenite::client::IntoClientRequest;
use yrs::types::ToJson;
use yrs::updates::encoder::Encode;
use yrs::{
    encoding::read::Read as _,
    updates::decoder::{Decode as _, DecoderV1},
    StateVector,
};
use yrs::{Doc, Map, MapPrelim, ReadTxn, Transact, Update, WriteTxn};

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
async fn main_server_test(pool: PgPool) -> sqlx::Result<()> {
    let (closer, close_signal) = channel::<()>();
    let (addr, serve) = server::start_main_server(Config {
        pool: Some(Box::leak(Box::new(pool))),
        port: Some(0),
        shutdown_signal: Some(close_signal),
        key_set: Some(testonly_key_set().await.unwrap()),
    })
    .await;
    let client = Client::default();

    // First, try a request without any credentials attached.
    {
        let res = client
            .get(&format!("http://{addr}/api/projects"))
            .send()
            .await
            .expect("Failed to send request.");
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    // Next, make the same request with a valid token.
    {
        let token = encode_token(&Claims::default(), KID_1, PEM_1).unwrap();
        let res = client
            .get(&format!("http://{addr}/api/projects"))
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
            project.get("project_id").unwrap().as_str().unwrap(),
            "koso-staging"
        );
        assert_eq!(
            project.get("name").unwrap().as_str().unwrap(),
            "Koso Staging"
        );
    }

    // Next, validate the websocket endpoint also rejects uauthenticated users.
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

    // Also, ensure that authenticated but unauthorized users are rejected.
    {
        let mut req = format!("ws://{addr}/api/ws/projects/koso-staging")
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
        let mut req = format!("ws://{addr}/api/ws/projects/koso-staging")
            .into_client_request()
            .unwrap();
        let token = encode_token(&Claims::default(), KID_1, PEM_1).unwrap();
        req.headers_mut().insert(
            "authorization",
            HeaderValue::from_str(format!("Bearer {token}").as_str()).unwrap(),
        );

        let (mut socket, response) = tokio_tungstenite::connect_async(req.clone()).await.unwrap();
        let socket = &mut socket;
        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
        assert_eq!(read_sync_request(socket).await, StateVector::default());
        close_socket(socket).await;

        // Abruptly close the socket, this'll trigger the error handling in client_message_handler
        let (_socket, response) = tokio_tungstenite::connect_async(req.clone()).await.unwrap();
        assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    }

    // Finally, run through a valid websocket interaction.
    let mut req = format!("ws://{addr}/api/ws/projects/koso-staging")
        .into_client_request()
        .unwrap();
    let token = encode_token(&Claims::default(), KID_1, PEM_1).unwrap();
    req.headers_mut().insert(
        "authorization",
        HeaderValue::from_str(format!("Bearer {token}").as_str()).unwrap(),
    );

    let (mut socket, response) = tokio_tungstenite::connect_async(req.clone()).await.unwrap();
    let socket = &mut socket;
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    let doc_1: Doc = Doc::new();
    doc_1.transact_mut().get_or_insert_map("graph");

    let (mut socket_2, response) = tokio_tungstenite::connect_async(req.clone()).await.unwrap();
    let socket_2 = &mut socket_2;
    assert_eq!(response.status(), StatusCode::SWITCHING_PROTOCOLS);
    let doc_2: Doc = Doc::new();
    doc_2.transact_mut().get_or_insert_map("graph");
    {
        let mut txn = doc_2.transact_mut();
        let graph = txn.get_or_insert_map("graph");
        graph.insert(&mut txn, "entry1", "value1");
        graph.insert(&mut txn, "entry2", MapPrelim::from([("inner", "value2")]));
    }

    // Read the initial sync_request.
    assert_eq!(read_sync_request(socket).await, StateVector::default());
    // Send our own sync request
    socket
        .send(Message::Binary(msg_sync::sync_request(
            &doc_1.transact().state_vector(),
        )))
        .await
        .unwrap();
    // Read the sync_response.
    assert_eq!(read_sync_response(socket).await, Update::default());
    // Send the sync_response.
    socket
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
    let sync_update = read_sync_update(socket).await;
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
    doc_3.transact_mut().get_or_insert_map("graph");
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

    // Close the sockets.
    close_socket(socket).await;
    close_socket(socket_3).await;

    // Initated server shutdown.
    tracing::info!("Sending server shutdown signal...");
    closer.send(()).unwrap();
    // The server will close the client.
    respond_closed_socket(socket_2).await;
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
