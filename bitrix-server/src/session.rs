use actix::{fut, prelude::*};
use actix_web_actors::ws;
use bitrix_actix_protobuf;
use prost::Message;
use uuid::Uuid;

use bitrix_channels::Channel;

use crate::{
    message::{ProtobufMessage, SubscribeChannelMessage},
    server::WsPullServer,
};

#[derive(Debug)]
pub struct WsSession {
    id: Uuid,
    pub channels: Vec<Channel>,
}

impl WsSession {
    pub fn get_target(&self) -> String {
        format!("WsSession:{}",self.id.clone())
    }
    pub fn get_channels(&self) -> Vec<Channel> {
        self.channels.clone()
    }
    pub fn set_channels(&mut self, channels: Vec<Channel>) {
        self.channels = channels;
    }
}

impl Default for WsSession {
    fn default() -> Self {
        WsSession {
            id: Uuid::new_v4(),
            channels: Vec::new()
        }
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::debug!(target: self.get_target().as_str(), "Started");

        WsPullServer::from_registry()
            .send(
                SubscribeChannelMessage(
                    self.channels.clone(),
                    ctx.address().recipient()
                )
            )
            .into_actor(self)
            .then(|_id, _act, _ctx| fut::ready(()))
            .wait(ctx);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        log::debug!(target: self.get_target().as_str(), "Stopped");
    }
}

impl Handler<ProtobufMessage> for WsSession {
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
            log::error!(target: self.get_target().as_str(), "Couldn't encode message: {encode_result:#?}");
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(err) => {
                log::error!(target: self.get_target().as_str(), "stream handler error: {}", err);
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            },
            ws::Message::Text(_) => {
                log::error!(target: self.get_target().as_str(), "We don't support 'text' message type now");
                log::trace!(target: self.get_target().as_str(), "Message: {msg:?}");
            },
            ws::Message::Binary(_) => {
                log::error!(target: self.get_target().as_str(), "We don't support 'binary' message type now");
                log::trace!(target: self.get_target().as_str(), "Message: {msg:?}");
            },
            _ => {}
        }
    }
}
