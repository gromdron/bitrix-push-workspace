use std::collections::HashMap;

use actix::prelude::*;
use actix_broker::BrokerSubscribe;
use bitrix_channels::Channel;
use crate::message::{ProtobufMessage, SendPullMessage, SubscribeChannelMessage};

type Client = Recipient<ProtobufMessage>;
type Subscribers = Vec<Client>;

#[derive(Default)]
pub struct WsPullServer {
    channels: HashMap<String, Subscribers>,
}

impl WsPullServer {
    fn take_subscribers(&mut self, channel_name: Channel) -> Option<Subscribers> {
        let subscribers = self.channels.get_mut(&channel_name.to_string())?;
        let subscribers = std::mem::take(subscribers);
        Some(subscribers)
    }

    fn add_client_to_channel(&mut self, channel_name: Channel, client: Client) {
        let subscribers = self.channels.entry(channel_name.to_string()).or_default();
        subscribers.push(client);
    }

    fn send_pull_message(&mut self, channel_name: Channel, msg: ProtobufMessage) -> Option<()> {
        let mut subscribers = self.take_subscribers(channel_name.clone())?;

        for client in subscribers.drain(..) {
            match client.try_send(msg.clone()) {
                Ok(()) => self.add_client_to_channel(channel_name.clone(), client),
                Err(error_text) => {
                    log::debug!("WsPullServer::send_pull_message => Channel: {channel_name:?} => {error_text:?}");
                }
            };
        }

        Some(())
    }
}

impl Actor for WsPullServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscribe_system_async::<SendPullMessage>(ctx);
    }
}

impl Handler<SubscribeChannelMessage> for WsPullServer {
    type Result = MessageResult<SubscribeChannelMessage>;

    fn handle(&mut self, msg: SubscribeChannelMessage, _ctx: &mut Self::Context) -> Self::Result {
        let SubscribeChannelMessage(channels, client) = msg;

        for channel_name in channels {
            self.add_client_to_channel(channel_name.clone(), client.clone());
        }

        MessageResult(())
    }
}

impl Handler<SendPullMessage> for WsPullServer {
    type Result = ();

    fn handle(&mut self, msg: SendPullMessage, _ctx: &mut Self::Context) {
        let SendPullMessage(channel_names, protobuf_msg) = msg;

        log::debug!(
            "WsPullServer[Handler[<SendPullMessage>]]::handle => channel_names {channel_names:?}"
        );

        for channel_name in channel_names {
            self.send_pull_message(channel_name.clone(), protobuf_msg.clone());
        }
    }
}

impl SystemService for WsPullServer {}
impl Supervised for WsPullServer {}
