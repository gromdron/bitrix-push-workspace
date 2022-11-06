use actix::{fut, prelude::*};
use actix_web_actors::ws;
use prost::Message;
use bitrix_actix_protobuf;

use bitrix_channels::Channel;

use crate::{
    message::{ProtobufMessage, SubscribeChannelMessage},
    server::WsPullServer,
};

#[derive(Default, Debug)]
pub struct WsPullSession {
    id: usize,
    room: String,
    pub channels: Vec<Channel>,
    name: Option<String>,
}

impl Actor for WsPullSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let subscribe_msg =
            SubscribeChannelMessage(self.channels.clone(), ctx.address().recipient());

        WsPullServer::from_registry()
            .send(subscribe_msg)
            .into_actor(self)
            .then(|_id, _act, _ctx| fut::ready(()))
            .wait(ctx);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        log::info!(
            "WsPullSession closed for {}({}) in room {}",
            self.name.clone().unwrap_or_else(|| "anon".to_string()),
            self.id,
            self.room
        );
    }
}

impl Handler<ProtobufMessage> for WsPullSession {
    type Result = ();

    fn handle(&mut self, msg: ProtobufMessage, ctx: &mut Self::Context) {
        let mut body = Vec::new();
        let encode_result = msg
            .0
            .encode(&mut body)
            .map_err(bitrix_actix_protobuf::ProtoBufPayloadError::Serialize);

        if encode_result.is_ok() {
            ctx.binary(body);
        } else {
            log::error!("Couldn't encode message: {encode_result:#?}");
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsPullSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        log::debug!("WEBSOCKET MESSAGE: {msg:?}");

        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}
