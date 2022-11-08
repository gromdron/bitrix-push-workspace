
use actix_web::http::header::{ContentType, HeaderValue};
use actix_web::{web, Error, HttpRequest, HttpResponse, Responder};
use actix_web_actors::ws;
use futures_util::stream::StreamExt as _;
use serde::{Deserialize, Serialize};

use bitrix_channels::Parser;
use actix_broker::{Broker, SystemBroker};
use bitrix_actix_protobuf;
use bitrix_channels::Channel;

use crate::{
    utils,
    items,
    message::{SendPullMessage, ProtobufMessage},
    session::WsPullSession,
};

/*
Finally we need to get requests:

POST /pub/ -> Application.publish. Trusted request.
POST /pub/?binaryMode=true -> Application.processClientRequest. Trusted request.
GET /pub/ -> Application.getChannelStats. Trusted request.

POST /rest/ -> Application.processClientRequest. Untrusted request.
Client websocket -> Application.processClientRequest. Untrusted request.

GET /server-stat/ -> Application.getServerStats. Trusted request.

GET /sub/ -> Application.subscribe. Long Polling requests.
GET UPGRADE /sub/ -> Application.subscribe. Websocket requests.
*/

pub fn routes_configure(cfg: &mut web::ServiceConfig) {
    /* Easy healthcheck */
    cfg.service(web::resource("/").route(web::get().to(|| HttpResponse::Ok())))
        .service(web::scope("/bitrix")
            .service(web::resource("/pub/").to(publication))
            .service(web::resource("/subws/").to(sub_ws))
       );
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct SubscribeWsQueryString {
    #[serde(rename(deserialize = "CHANNEL_ID"))]
    channel_ids: String,
    #[serde(rename(deserialize = "binaryMode"))]
    is_binary: Option<bool>,
    revision: Option<i32>,
    mid: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct PublishQueryString {
    #[serde(rename(deserialize = "CHANNEL_ID"))]
    channel_ids: Option<String>,
    #[serde(rename(deserialize = "binaryMode"))]
    is_binary: Option<bool>,
}

async fn publication(
    req: HttpRequest,
    mut payload: web::Payload,
    query: web::Query<PublishQueryString>,
    parser: web::Data<Parser>
) -> Result<HttpResponse, Error> {
    if query.is_binary.is_some() {
        let requests_batch = bitrix_actix_protobuf::ProtoBufMessage::<items::RequestBatch>::new(
            &req,
            &mut payload.into_inner(),
        )
        .await;

        let requests = match requests_batch {
            Ok(request_batch) => request_batch.requests,
            Err(error_kind) => {
                log::error!("Got binary that couldn't decode. Error: {error_kind}");
                return Ok(HttpResponse::BadRequest()
                    .content_type(ContentType::plaintext())
                    .body(error_kind.to_string()));
            }
        };

        log::debug!("Parsed protobuf: {requests:?}");

        for request in requests {
            if request.command.is_none() {
                log::warn!("Receive empty command");
                continue;
            }

            let request_command = request.command.unwrap();

            match request_command {
                items::request::Command::IncomingMessages(incoming_message_request) => {
                    log::debug!("Process income messages request: {incoming_message_request:?}");

                    for income_message in incoming_message_request.messages.into_iter() {
                        log::debug!("Process income message request: {income_message:?}");

                        let mut channel_ids = Vec::new();

                        for receiver in income_message.receivers {
                            match Channel::try_from(receiver.id) {
                                Ok(channel) => {
                                    channel_ids.push(channel);
                                }
                                Err(_) => continue,
                            }
                        }

                        if channel_ids.is_empty() {
                            continue;
                        }

                        let protobuf_message = items::ResponseBatch {
                            responses: vec![items::Response {
                                command: Some(items::response::Command::OutgoingMessages(
                                    items::OutgoingMessagesResponse {
                                        messages: vec![items::OutgoingMessage {
                                            id: utils::get_message_id(),
                                            body: income_message.body,
                                            expiry: income_message.expiry,
                                            created: 0,
                                            sender: Some(items::Sender {
                                                r#type: items::SenderType::Backend as i32,
                                                id: vec![],
                                            }),
                                        }],
                                    },
                                )),
                            }],
                        };

                        Broker::<SystemBroker>::issue_async(SendPullMessage(
                            channel_ids,
                            ProtobufMessage(protobuf_message),
                        ));
                    }
                }
                items::request::Command::ChannelStats(channel_stats_request) => {
                    log::debug!("Process channel stats request: {channel_stats_request:?}");
                }
                items::request::Command::ServerStats(server_stats_request) => {
                    log::debug!("Process server stats request: {server_stats_request:?}");
                }
                _ => {
                    log::error!(
                        "{}",
                        format!("Got strange command: {:#?}. Ignore.", request_command).to_string()
                    );
                    continue;
                }
            }
        }
    } else {
        /* Trying to publish nonbinary message without channels */
        if query.channel_ids.is_none() {
            return Ok(HttpResponse::BadRequest()
                .insert_header(("X-PUSH-ERR", "[EPR001] Channel ids is missed"))
                .content_type(ContentType::plaintext())
                .finish());
        }

        let channel_ids = query.channel_ids.as_ref().unwrap();

        let parse_channelds_result = parser.parse(channel_ids.clone());

        log::trace!("Channels from request: {parse_channelds_result:?}");

        if parse_channelds_result.is_err() {
            return Ok(HttpResponse::BadRequest()
                .insert_header(("X-PUSH-ERR", format!("[EPR002] Channel ids parser error: {}", parse_channelds_result.err().unwrap())))
                .content_type(ContentType::plaintext())
                .finish());
        }

        let mut bytes = web::BytesMut::new();
        while let Some(item) = payload.next().await {
            bytes.extend_from_slice(&item?);
        }

        log::debug!("Got push request {req:?},\r\n{bytes:?}");

        let protobuf_message = items::ResponseBatch {
            responses: vec![items::Response {
                command: Some(items::response::Command::OutgoingMessages(
                    items::OutgoingMessagesResponse {
                        messages: vec![items::OutgoingMessage {
                            id: utils::get_message_id(),
                            body: std::str::from_utf8(&bytes).unwrap().to_string(),
                            expiry: req
                                .headers()
                                .get("message-expiry")
                                .unwrap_or(&HeaderValue::from_str("0")?)
                                .to_str()
                                .unwrap()
                                .parse::<u32>()
                                .unwrap(),
                            created: 0,
                            sender: Some(items::Sender {
                                r#type: items::SenderType::Backend as i32,
                                id: vec![],
                            }),
                        }],
                    },
                )),
            }],
        };

        Broker::<SystemBroker>::issue_async(SendPullMessage(
            parse_channelds_result.unwrap(),
            ProtobufMessage(protobuf_message),
        ));
    }

    Ok(HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .finish())
}

async fn sub_ws(
    req: HttpRequest,
    stream: web::Payload,
    query: web::Query<SubscribeWsQueryString>,
    parser: web::Data<Parser>
) -> Result<impl Responder, Error> {
    let mut pull_session = WsPullSession::default();

    let parse_channelds_result = parser.parse(query.channel_ids.clone());

    match parse_channelds_result {
        Ok(channels) => {
            pull_session.channels = channels;
        }
        Err(text) => {
            log::debug!("Parsed channel failed: {}", text);
            return Ok(HttpResponse::BadRequest()
                .insert_header(("X-PUSH-ERR", format!("[ES001] Parse channels error: {}", text)))
                .content_type(ContentType::plaintext())
                .body(text.to_string())
            );
        }
    }

    ws::start(pull_session, &req, stream)
}