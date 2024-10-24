use libp2p::{
    mdns::{tokio::Behaviour as Mdns, Config as MdnsConfig},
    request_response::{
        Behaviour as RequestResponse, Codec as RequestResponseCodec, Config as RequestResponseConfig,
        Event as RequestResponseEvent, ProtocolName, ProtocolSupport,
    },
    swarm::NetworkBehaviour,
    PeerId,
};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use std::{io, iter};

// Use futures::io
use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug, Clone)]
pub struct MyProtocol();

impl ProtocolName for MyProtocol {
    fn protocol_name(&self) -> &[u8] {
        b"/my-protocol/1.0.0"
    }
}

#[derive(Clone)]
pub struct MyCodec();

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MyRequest(pub Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MyResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    Identity { name: String },
    IdentityAck { ack: bool },
    Command { command: String },
    CommandAck { ack: bool },
}

#[async_trait]
impl RequestResponseCodec for MyCodec {
    type Protocol = MyProtocol;
    type Request = MyRequest;
    type Response = MyResponse;

    async fn read_request<T>(
        &mut self,
        _: &MyProtocol,
        io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send, // Added + Send
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        Ok(MyRequest(buf))
    }

    async fn read_response<T>(
        &mut self,
        _: &MyProtocol,
        io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send, // Added + Send
    {
        let mut buf = Vec::new();
        io.read_to_end(&mut buf).await?;
        Ok(MyResponse(buf))
    }

    async fn write_request<T>(
        &mut self,
        _: &MyProtocol,
        io: &mut T,
        MyRequest(data): MyRequest,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send, // Added + Send
    {
        io.write_all(&data).await?;
        io.flush().await
    }

    async fn write_response<T>(
        &mut self,
        _: &MyProtocol,
        io: &mut T,
        MyResponse(data): MyResponse,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send, // Added + Send
    {
        io.write_all(&data).await?;
        io.flush().await
    }
}

#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "MyBehaviourEvent")]
pub struct MyBehaviour {
    pub request_response: RequestResponse<MyCodec>,
    pub mdns: Mdns,
}

// Let the macro generate MyBehaviourEvent

impl MyBehaviour {
    pub fn new(local_peer_id: PeerId) -> Result<Self, io::Error> {
        // Setup the request-response behaviour
        let protocols = iter::once((MyProtocol(), ProtocolSupport::Full));
        let cfg = RequestResponseConfig::default();
        let request_response = RequestResponse::new(MyCodec(), protocols, cfg);

        // Setup mDNS
        let mdns = Mdns::new(MdnsConfig::default(), local_peer_id)?;

        Ok(MyBehaviour {
            request_response,
            mdns,
        })
    }
}
