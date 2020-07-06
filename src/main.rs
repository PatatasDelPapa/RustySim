#![forbid(unsafe_code)]
use hashbrown::HashMap;
use tokio::sync::{mpsc, oneshot};
use trait_async::trait_async;
use priority_queue::PriorityQueue;
use std::cmp::Reverse;
// use futures::stream::FuturesOrdered;


#[tokio::main]
async fn main() {
    env_logger::init();
    log::info!("Iniciando Programa");
    let (transmisor, mut receptor) = mpsc::channel::<(Command, oneshot::Sender<Response>)>(1);
    tokio::spawn(async move {
        let mut lista_passivates = HashMap::new();
        let mut future_event_list = PriorityQueue::new();
        let mut future_event_list_aux = HashMap::new();
        let mut resultado;
        let mut tiempo: u64 = 0;
        loop {
            resultado = receptor.recv().await;
            match resultado {
                Some((command, response)) => {
                    match command {
                        Command::Dormir(id) => {
                            log::debug!("Llego command passivate para id: {}", id);
                            let _ = lista_passivates.insert(id, response);
                        }
                        Command::Despertar(id) => {
                            log::debug!("Llego command activate para id: {}", id);
                            let resultado = lista_passivates.remove(&id);
                            match resultado {
                                Some(channel) => {
                                    // log::debug!("Mandando respuestas!");
                                    channel.send(Response::Romper).unwrap();
                                    response.send(Response::Romper).unwrap();
                                }
                                None => {
                                    // log::warn!("Proceso con ID: {} no esta durmiendo", id);
                                    response.send(Response::Continuar).unwrap();
                                }
                            };
                        }
                        Command::Hold(id, mut t) => {
                            log::debug!("Llego command hold para id: {} con tiempo: {}", id, t);
                            let _ = future_event_list_aux.insert(id, response);
                            t += tiempo;
                            let _ = future_event_list.push(id, Reverse(t));
                        }
                        Command::Advance => {
                            log::debug!("Llego command Advance");
                            let some_id = future_event_list.pop();
                            // let (id, t) = some_id.unwrap();
                            match some_id {
                                Some((id, t)) => {
                                    log::debug!("ID {} ha salido de la FEL", id);
                                    tiempo += t.0 - tiempo;
                                    let resultado = future_event_list_aux.remove(&id);
                                    if let Some(channel) = resultado {
                                        // log::debug!("Mandando respuesta a channel");
                                        channel.send(Response::Romper).unwrap();
                                        // log::debug!("Mandando Respuesta a response");
                                        response.send(Response::Romper).unwrap();
                                    }
                                }
                                None => {
                                    // log::warn!("No habia ninguna id en la FEL");
                                    response.send(Response::Continuar).unwrap();
                                }
                            }

                        }
                    };
                }
                None => {
                    // log::debug!("Todos los recievers han sido dropeados, rompiendo loop");
                    break;
                }
            };
        }
        log::info!("Tiempo final simulado = {}", tiempo);
    });

    let mut handles = vec![];
    let clone = transmisor.clone();
    handles.push(tokio::spawn(async move {
        let mut obj_1 = Objeto::new(clone, 1);
        obj_1.hold(5).await;
        obj_1.activate(2).await;
        obj_1.passivate().await;
    }));


    let clone = transmisor.clone();
    handles.push(tokio::spawn(async move {
        let mut obj_2 = Objeto::new(clone, 2);
        obj_2.passivate().await;
        obj_2.hold(42).await;
        obj_2.activate(1).await;
    }));

    let mut clone_1 = transmisor;
    handles.push(tokio::spawn(async move {
        advance(&mut clone_1).await;
        advance(&mut clone_1).await;
    }));
    
    // let handles_1 = handles.pop().unwrap();
    // let handles_2 = handles.pop().unwrap();
    // let handles_3 = handles.pop().unwrap();

    // let _ = tokio::join!(
    //     handles_1,
    //     handles_2,
    //     handles_3,
    // );

    for handle in handles.drain(..){
        handle.await.unwrap();
    }

}

async fn advance(channel: &mut mpsc::Sender<(Command, oneshot::Sender<Response>)>) {
    loop {
        let (tx, rx) = oneshot::channel();
        channel.send((Command::Advance, tx))
            .await
            .ok()
            .unwrap();
        let result = rx.await.unwrap();
        if let Response::Romper = result {
            // log::debug!("ADVANCE => Llego romper");
            break;
        } else {
            // log::debug!("ADVANCE => No llego romper");
        }
    }
    // log::debug!("Advance Ends");
}

// Representacion de la corrutina
// Tiene un channel para enviar cosas al controlador
struct Objeto {
    channel: mpsc::Sender<(Command, oneshot::Sender<Response>)>,
    id: u64,
}

impl Objeto {
    fn new(channel: mpsc::Sender<(Command, oneshot::Sender<Response>)>, id: u64) -> Self {
        Self { channel, id }
    }
}

enum Command {
    Dormir(u64),
    Despertar(u64),
    Hold(u64, u64),
    Advance,
}

#[derive(Debug)]
enum Response {
    Continuar,
    Romper,
}

#[trait_async]
trait Pausable {
    async fn hold(&mut self, t: u64);
    async fn passivate(&mut self);
    async fn activate(&mut self, c: u64);
    // async fn activate<T>(&self, c: T)
    // where
    //     T: Pausable + Sync + Send + 'trait_async;
}

#[trait_async]
impl Pausable for Objeto {
    async fn hold(&mut self, t: u64) {
        loop {
            let (tx, rx) = oneshot::channel();
            self.channel
                .send((Command::Hold(self.id, t), tx))
                .await
                .ok()
                .unwrap();
            let result = rx.await.unwrap();
            if let Response::Romper = result {
                break;
            } else {
                // log::debug!("HOLD => No llego romper");
            }
        }
        // log::debug!("Hold id {} Ends", self.id);
    }

    async fn passivate(&mut self) {
        // log::debug!("Passivate ID: {}", self.id);
        let mut result;
        loop {
            let (tx, rx) = oneshot::channel();
            self.channel
                .send((Command::Dormir(self.id), tx))
                .await
                .ok()
                .unwrap();
            result = rx.await.unwrap();
            if let Response::Romper = result {
                break;
            }else{
                // log::debug!("PASSIVATE => No llego romper");
            }
        }
        // log::debug!("Passivate ID {} Ends", self.id);
    }

    async fn activate(&mut self, c: u64) {
        // log::debug!("Activate ID: {} -> {}", self.id, c);
        // async fn activate<T>(&self, c: T) {
        // where
        //     T: Pausable + Send + Sync + 'trait_async,s
        let mut result;
        loop {
            // dbg!(result);
            let (tx, rx) = oneshot::channel();
            self.channel
                .send((Command::Despertar(c), tx))
                .await
                .ok()
                .unwrap();
            result = rx.await.unwrap();
            if let Response::Romper = result {
                break;
            } else {
                // log::debug!("ACTIVATE => No llego romper");
            }
        }
        // log::debug!("Activate id {} to id {} Ends", self.id, c);
    }
}
