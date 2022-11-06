#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Sender {
    #[prost(enumeration="SenderType", tag="1")]
    pub r#type: i32,
    #[prost(bytes="vec", tag="2")]
    pub id: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SenderType {
    Unknown = 0,
    Client = 1,
    Backend = 2,
}
impl SenderType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SenderType::Unknown => "UNKNOWN",
            SenderType::Client => "CLIENT",
            SenderType::Backend => "BACKEND",
        }
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct License {
    #[prost(uint32, tag="1")]
    pub id: u32,
    #[prost(string, tag="2")]
    pub client_id: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub security_key: ::prost::alloc::string::String,
    #[prost(string, tag="4")]
    pub security_algo: ::prost::alloc::string::String,
    #[prost(fixed32, tag="5")]
    pub date_to: u32,
    #[prost(string, tag="6")]
    pub site_url: ::prost::alloc::string::String,
    #[prost(string, tag="7")]
    pub verification_query: ::prost::alloc::string::String,
    #[prost(fixed32, tag="8")]
    pub last_check: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ResponseBatch {
    #[prost(message, repeated, tag="1")]
    pub responses: ::prost::alloc::vec::Vec<Response>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Response {
    #[prost(oneof="response::Command", tags="1, 2, 3, 4")]
    pub command: ::core::option::Option<response::Command>,
}
/// Nested message and enum types in `Response`.
pub mod response {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Command {
        #[prost(message, tag="1")]
        OutgoingMessages(super::OutgoingMessagesResponse),
        #[prost(message, tag="2")]
        ChannelStats(super::ChannelStatsResponse),
        #[prost(message, tag="3")]
        ServerStats(super::JsonResponse),
        #[prost(string, tag="4")]
        Json(::prost::alloc::string::String),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutgoingMessagesResponse {
    #[prost(message, repeated, tag="1")]
    pub messages: ::prost::alloc::vec::Vec<OutgoingMessage>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct OutgoingMessage {
    #[prost(bytes="vec", tag="1")]
    pub id: ::prost::alloc::vec::Vec<u8>,
    #[prost(string, tag="2")]
    pub body: ::prost::alloc::string::String,
    #[prost(uint32, tag="3")]
    pub expiry: u32,
    #[prost(fixed32, tag="4")]
    pub created: u32,
    #[prost(message, optional, tag="5")]
    pub sender: ::core::option::Option<Sender>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChannelStatsResponse {
    #[prost(message, repeated, tag="1")]
    pub channels: ::prost::alloc::vec::Vec<ChannelStats>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChannelStats {
    #[prost(bytes="vec", tag="1")]
    pub id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag="2")]
    pub is_private: bool,
    #[prost(bool, tag="3")]
    pub is_online: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct JsonResponse {
    #[prost(string, tag="1")]
    pub json: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Receiver {
    #[prost(bytes="vec", tag="1")]
    pub id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag="2")]
    pub is_private: bool,
    #[prost(bytes="vec", tag="3")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NotificationBatch {
    #[prost(message, repeated, tag="1")]
    pub notifications: ::prost::alloc::vec::Vec<Notification>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Notification {
    #[prost(oneof="notification::Command", tags="1, 2")]
    pub command: ::core::option::Option<notification::Command>,
}
/// Nested message and enum types in `Notification`.
pub mod notification {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Command {
        #[prost(message, tag="1")]
        IpcMessages(super::IpcMessages),
        #[prost(message, tag="2")]
        IpcLicenses(super::IpcLicenses),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IpcMessages {
    #[prost(message, repeated, tag="1")]
    pub messages: ::prost::alloc::vec::Vec<IpcMessage>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IpcMessage {
    #[prost(message, repeated, tag="1")]
    pub receivers: ::prost::alloc::vec::Vec<Receiver>,
    #[prost(bytes="vec", tag="2")]
    pub outgoing_message_id: ::prost::alloc::vec::Vec<u8>,
    #[prost(message, optional, tag="3")]
    pub outgoing_message: ::core::option::Option<OutgoingMessage>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IpcLicenses {
    #[prost(message, repeated, tag="1")]
    pub licenses: ::prost::alloc::vec::Vec<IpcLicense>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IpcLicense {
    #[prost(message, optional, tag="1")]
    pub license: ::core::option::Option<License>,
    #[prost(string, tag="2")]
    pub action: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RequestBatch {
    #[prost(message, repeated, tag="1")]
    pub requests: ::prost::alloc::vec::Vec<Request>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Request {
    #[prost(oneof="request::Command", tags="1, 2, 3, 4")]
    pub command: ::core::option::Option<request::Command>,
}
/// Nested message and enum types in `Request`.
pub mod request {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Command {
        #[prost(message, tag="1")]
        IncomingMessages(super::IncomingMessagesRequest),
        #[prost(message, tag="2")]
        ChannelStats(super::ChannelStatsRequest),
        #[prost(message, tag="3")]
        ServerStats(super::ServerStatsRequest),
        #[prost(message, tag="4")]
        Registration(super::RegisterRequest),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IncomingMessagesRequest {
    #[prost(message, repeated, tag="1")]
    pub messages: ::prost::alloc::vec::Vec<IncomingMessage>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct IncomingMessage {
    #[prost(message, repeated, tag="1")]
    pub receivers: ::prost::alloc::vec::Vec<Receiver>,
    #[prost(message, optional, tag="2")]
    pub sender: ::core::option::Option<Sender>,
    #[prost(string, tag="3")]
    pub body: ::prost::alloc::string::String,
    #[prost(uint32, tag="4")]
    pub expiry: u32,
    #[prost(string, tag="5")]
    pub r#type: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChannelStatsRequest {
    #[prost(message, repeated, tag="1")]
    pub channels: ::prost::alloc::vec::Vec<ChannelId>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ChannelId {
    #[prost(bytes="vec", tag="1")]
    pub id: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag="2")]
    pub is_private: bool,
    #[prost(bytes="vec", tag="3")]
    pub signature: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ServerStatsRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisterRequest {
    #[prost(string, tag="1")]
    pub verification_query: ::prost::alloc::string::String,
}
