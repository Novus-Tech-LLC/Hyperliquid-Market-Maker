use std::env;
use tokio::{sync::{mpsc::{unbounded_channel, UnboundedSender}, broadcast::{self, Sender as BroadcastSender}}, time::Duration};
use actix::{Actor, StreamHandler, Handler, Message};
use actix_cors::Cors;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder, Error as ActixError};
use actix::ActorContext;
use actix::AsyncContext;
use actix_web_actors::ws;
use dotenv::dotenv;
use env_logger;
use log::{error, info};
use serde_json;
use hyperliquid_rust_bot::{
    Bot, BotEvent, UpdateFrontend, LocalWallet, Wallet, BaseUrl,
};
use hyperliquid_rust_bot::strategy::{Strategy, CustomStrategy, Risk, Style, Stance};

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();

    let url = BaseUrl::Mainnet;
    let wallet: LocalWallet = env::var("PRIVATE_KEY")?.parse()?;
    let pubkey = env::var("WALLET")?;
    let wallet = Wallet::new(url, pubkey, wallet).await?;

    let (mut bot, cmd_sender) = Bot::new(wallet).await?;
    let (update_tx, mut update_rx) = unbounded_channel::<UpdateFrontend>();
    tokio::spawn(async move { bot.start(update_tx).await });

    let (bcast_tx, _) = broadcast::channel::<UpdateFrontend>(128);
    let bcast_cl = bcast_tx.clone();

    let mut _dummy_rx = bcast_tx.subscribe();
    tokio::spawn(async move {
        loop {
            let _ = _dummy_rx.recv().await;
        }
    });

    tokio::spawn(async move {
        while let Some(update) = update_rx.recv().await {
            if let Err(err) = bcast_tx.send(update) {
                error!("broadcast send error: {}", err);
            }
        }
    });

    let cmd_data = web::Data::new(cmd_sender.clone());
    let bcast_data = web::Data::new(bcast_cl.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(cmd_data.clone())
            .app_data(bcast_data.clone())
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials(),
            )
            .route("/command", web::post().to(execute))
            .route("/ws", web::get().to(ws_route))
    })
    .bind(("127.0.0.1", 8090))?
    .run()
    .await?;

    Ok(())
}

async fn execute(
    raw: web::Bytes,
    sender: web::Data<UnboundedSender<BotEvent>>,
) -> impl Responder {
    // Log the raw request body
    let body_str = String::from_utf8_lossy(&raw);
    println!("Incoming raw body: {}", body_str);

    // Try to deserialize
    match serde_json::from_slice::<BotEvent>(&raw) {
        Ok(event) => {
            if let Err(err) = sender.send(event) {
                error!("failed to send command: {}", err);
                return HttpResponse::InternalServerError().finish();
            }
            HttpResponse::Ok().finish()
        }
        Err(err) => {
            error!("Failed to deserialize BotEvent: {}", err);
            HttpResponse::BadRequest().body(format!("Invalid BotEvent: {}", err))
        }
    }
}

async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    bcast: web::Data<BroadcastSender<UpdateFrontend>>,
) -> Result<HttpResponse, ActixError> {
    let rx = bcast.subscribe();
    let ws = MyWebSocket { rx };
    ws::start(ws, &req, stream)
}

#[derive(Message)]
#[rtype(result = "()")] 
struct ServerMessage(String);

struct MyWebSocket {
    rx: broadcast::Receiver<UpdateFrontend>,
}

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(Duration::from_secs(30), |_, ctx| ctx.ping(b""));

        let mut rx = self.rx.resubscribe();
        let addr = ctx.address();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(update) => {
                        if let Ok(text) = serde_json::to_string(&update) {
                            println!("\n{}\n", text);
                            addr.do_send(ServerMessage(text));
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(cnt)) => {
                        error!("missed {} messages", cnt);
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            addr.do_send(ServerMessage("__SERVER_CLOSED__".into()));
        });
    }
}

impl Handler<ServerMessage> for MyWebSocket {
    type Result = (); 

    fn handle(&mut self, msg: ServerMessage, ctx: &mut Self::Context) {
        if msg.0 == "__SERVER_CLOSED__" {
            ctx.close(None);
            ctx.stop();
        } else {
            ctx.text(msg.0);
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(ws::Message::Ping(p)) = msg {
            ctx.pong(&p);
        } else if let Ok(ws::Message::Close(reason)) = msg {
            ctx.close(reason);
            ctx.stop();
        }
    }
}


