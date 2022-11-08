use crate::items;
use actix::{Message, Recipient};
use bitrix_channels::Channel;

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct ProtobufMessage(pub items::ResponseBatch);

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct SubscribeChannelMessage(pub Vec<Channel>, pub Recipient<ProtobufMessage>);

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct SendPullMessage(pub Vec<Channel>, pub ProtobufMessage);
