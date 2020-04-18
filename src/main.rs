extern crate actix;
extern crate actix_web;
extern crate actix_web_actors;

use actix::prelude::*;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;

use std::env;
use std::fs;
use std::vec::Vec;

mod server;

struct WsChatSession {
    /// unique session id
    id: usize,
    // player name
    name: String,
    /// Chat server
    addr: Addr<server::GameServer>,
}

/// Entry point for our route
async fn chat_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::GameServer>>,
) -> Result<HttpResponse, Error> {
    println!("{:?}", &req);
    ws::start(
        WsChatSession {
            id: 0,
            name: String::new(),
            addr: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<server::Message> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        let msg = match msg {
            Err(_) => {
                println!("Stopped at line 63");
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        println!("WEBSOCKET MESSAGE: {:?}", msg);
        match msg {
            ws::Message::Ping(msg) => ctx.pong(&msg),
            ws::Message::Pong(_) => (),
            ws::Message::Text(text) => {
                let m = text.trim();
                // we check for /sss type of messages
                if m.starts_with('/') {
                    let v: Vec<&str> = m.split(' ').collect();
                    match v[0] {
                        "/join" => {
                            if v.len() == 2 {
                                let addr = ctx.address();
                                self.addr
                                    .send(server::Join {
                                        addr: addr.recipient(),
                                        name: v[1].to_string(),
                                    })
                                    .into_actor(self)
                                    .then(|res, act, ctx| {
                                        match res {
                                            Ok(res) => if res > 0 { act.id = res as usize } else { ctx.stop() },
                                            // something is wrong with chat server
                                            _ => ctx.stop(),
                                        }
                                        fut::ready(())
                                    })
                                    .wait(ctx);
                            } else {
                                ctx.text("ERROR: Malformed Join request");
                            }
                        },
                        "/play" => {
                            if v.len() == 2 {
                                self.addr.send(server::PlayCard {
                                    id: self.id,
                                    card_index: v[1].parse::<usize>().unwrap(),
                                })
                                .into_actor(self)
                                .then(|res, act, ctx| {
                                    match res {
                                        Ok(res) => ctx.text(res),
                                        // something is wrong with chat server
                                        _ => ctx.stop(),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx);
                            } else {
                                ctx.text("ERROR: Malformed Play request");
                            }
                        },
                        "/discard" => {
                            if v.len() == 2 {
                                self.addr.send(server::Discard {
                                    id: self.id,
                                    card_index: v[1].parse::<usize>().unwrap(),
                                })
                                .into_actor(self)
                                .then(|res, act, ctx| {
                                    match res {
                                        Ok(res) => ctx.text(res),
                                        // something is wrong with chat server
                                        _ => ctx.stop(),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx);
                            } else {
                                ctx.text("ERROR: Malformed Discard request");
                            }
                        },
                        "/swap" => {
                            if v.len() == 3 {
                                self.addr.send(server::Swap {
                                    id: self.id,
                                    index1: v[1].parse::<usize>().unwrap(),
                                    index2: v[2].parse::<usize>().unwrap(),
                                })
                                .into_actor(self)
                                .then(|res, act, ctx| {
                                    match res {
                                        Ok(res) => ctx.text(res),
                                        // something is wrong with chat server
                                        _ => ctx.stop(),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx);
                            } else {
                                ctx.text("ERROR: Malformed Swap request");
                            }
                        },
                        "/hint" => {
                            if v.len() == 3 {
                                self.addr.send(server::GiveHint {
                                    id: self.id,
                                    other_player: v[2].to_string(),
                                    hint: v[1].to_string(),
                                })
                                .into_actor(self)
                                .then(|res, act, ctx| {
                                    match res {
                                        Ok(res) => ctx.text(res),
                                        // something is wrong with chat server
                                        _ => ctx.stop(),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx);
                            } else {
                                ctx.text("ERROR: Malformed Hint request");
                            }
                        },
                        _ => ctx.text(format!("ERROR: unknown command: {:?}", m)),
                    }
                } else {
                    println!("Problem!");
                }
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(_) => (),
            ws::Message::Continuation(_) => (),
            ws::Message::Nop => (),
        }
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let ip = &args[1];
    let server = server::GameServer::default().start();

    HttpServer::new(move || {
        App::new()
            .data(server.clone())
            .route("/socketserver", web::get().to(chat_route))
            .route("/", web::get().to(enter))
            .route("/cards.js", web::get().to(script))
            .route("/style.css", web::get().to(style))
    })
    .bind(ip)?
    .run()
    .await
}

async fn enter() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(fs::read_to_string("enter.html").unwrap())
}

async fn playing() -> impl Responder {
    HttpResponse::Ok()
    .content_type("text/html")
    .body(fs::read_to_string("index.html").unwrap())
}

async fn script() -> impl Responder {
    HttpResponse::Ok()
    .content_type("text/javascript")
    .body(fs::read_to_string("cards.js").unwrap())
}

async fn style() -> impl Responder {
    HttpResponse::Ok()
    .content_type("text/css")
    .body(fs::read_to_string("style.css").unwrap())
}

