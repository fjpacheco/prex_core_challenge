use actix::{Actor, AsyncContext, Handler, Message, StreamHandler};
use actix_web::{Error, HttpRequest, HttpResponse, web};
use actix_web_actors::ws;
use tokio::sync::broadcast;

lazy_static::lazy_static! {
    static ref LOG_BROADCAST_SENDER: broadcast::Sender<LogMessage> = {
        let (sender, _) = broadcast::channel(32);
        sender
    };
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum LogMessage {
    Log(Vec<u8>),
    Stop(std::net::SocketAddr),
}

pub struct WebSocketWriter {
    sender: broadcast::Sender<LogMessage>,
}

impl Default for WebSocketWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketWriter {
    pub fn new() -> Self {
        Self {
            sender: LOG_BROADCAST_SENDER.clone(),
        }
    }
}

impl std::io::Write for WebSocketWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let buf_len = buf.len();
        if self.sender.receiver_count() > 0 {
            let _ = self.sender.send(LogMessage::Log(buf.to_vec()));
        }
        Ok(buf_len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct WebSocketActor {
    socket_addr: std::net::SocketAddr,
}

impl WebSocketActor {
    pub fn new(addr: std::net::SocketAddr) -> Self {
        Self { socket_addr: addr }
    }
}

impl Actor for WebSocketActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        let self_socket_addr = self.socket_addr;
        tokio::task::spawn(async move {
            let mut rx = LOG_BROADCAST_SENDER.subscribe();
            tracing::info!("[{}] Ready to listen logs and send to WS", self_socket_addr);
            loop {
                let line = rx.recv().await;
                match line {
                    Ok(LogMessage::Log(line)) => {
                        addr.do_send(LogLineMessage(line));
                    }
                    Ok(LogMessage::Stop(socket_addr)) => {
                        if self_socket_addr == socket_addr {
                            tracing::info!(
                                "[{}] No more lines received! Bye....",
                                self_socket_addr
                            );
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!("[{}] Error receiving log line: {:?}", self_socket_addr, e);
                        break;
                    }
                }
            }
            tracing::info!("[{}] Bye bye WS connection!", self_socket_addr);
        });
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::info!("[{}] Stopping myself WS connection", self.socket_addr);

        let _ = LOG_BROADCAST_SENDER.send(LogMessage::Stop(self.socket_addr));

        tracing::info!("[{}] WS connection stopped", self.socket_addr);
    }
}

struct LogLineMessage(Vec<u8>);

impl Message for LogLineMessage {
    type Result = ();
}

impl Handler<LogLineMessage> for WebSocketActor {
    type Result = ();

    fn handle(&mut self, msg: LogLineMessage, ctx: &mut Self::Context) {
        ctx.binary(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Close(_)) = msg {
            ctx.close(None);
        }
    }
}

#[macro_export]
macro_rules! WS_LOG_HANDLER {
    () => {
        actix_web::web::get().to($crate::infrastructure::inbound::http::ws_logger::ws_log_handler)
    };
}
pub const WS_LOG_HANDLER_ROUTE: &str = "/logs/ws";

pub async fn ws_log_handler(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let addr: std::net::SocketAddr = req.peer_addr().unwrap();
    tracing::info!("[{}] New websocket connection", addr);
    ws::start(WebSocketActor::new(addr), &req, stream)
}
