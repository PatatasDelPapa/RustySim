use hashbrown::HashMap;
use tokio::sync::{mpsc, oneshot};
use trait_async::trait_async;

#[macro_use]
extern crate log;

#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Iniciando Programa");
    let (transmisor, mut receptor) = mpsc::channel::<(Command, oneshot::Sender<Response>)>(1);
    tokio::spawn(async move {
        let mut procesos = HashMap::new();
        let mut resultado;
        loop {
            resultado = receptor.recv().await;
            match resultado {
                Some((command, response)) => {
                    match command {
                        Command::Dormir(obj) => {
                            debug!("Llego command dormir para id: {}", obj.id);
                            let _ = procesos.insert(obj, response);
                        }
                        Command::Despertar(obj) => {
                            debug!("Llego command despertar para id: {}", obj.id);
                            let resultado = procesos.remove(&obj);
                            match resultado {
                                Some(channel) => {
                                    debug!("Mandando respuestas!");
                                    channel.send(Response::Romper).unwrap();
                                    response.send(Response::Romper).unwrap();
                                }
                                None => {
                                    warn!("Proceso con ID: {} no esta durmiendo", obj.id);
                                    response.send(Response::Continuar).unwrap();
                                }
                            };
                        }
                    };
                }
                None => {
                    debug!("Todos los recievers han sido dropeados, rompiendo loop");
                    break;
                }
            };
        }
    });

    let mut handles = vec![];

    let clone = transmisor.clone();
    let obj_1 = Objeto::new(1);
    let obj_1_ref = &obj_1;
    handles.push(tokio::spawn(async move {
        obj_1.passivate(clone).await;
    }));


    let clone = transmisor.clone();
    handles.push(tokio::spawn(async move {
        let mut obj_2 = Objeto::new(2);
        obj_2.activate(*&obj_1_ref, clone).await;
    }));

    for handle in handles.drain(..) {
        handle.await.unwrap();
    }
}
// Representacion de la corrutina
// Tiene un channel para enviar cosas al controlador
#[derive(PartialEq, Eq, Hash)]
struct Objeto {
    id: u64,
}

impl Objeto {
    fn new(id: u64) -> Self {
        Self { id }
    }
}

enum Command<'a> {
    Dormir(&'a Objeto),
    Despertar(&'a Objeto),
}

#[derive(Debug)]
enum Response {
    Continuar,
    Romper,
}

#[trait_async]
trait Pausable {
    fn id(&self) -> u64;
    async fn hold(&self, t: u64);
    async fn passivate<'a>(&'a mut self, mut channel: mpsc::Sender<(Command<'a>, oneshot::Sender<Response>)>);
    async fn activate<'a>(&mut self, c: &'a Objeto, channel: mpsc::Sender<(Command<'a>, oneshot::Sender<Response>)>);
    // async fn activate<T>(&self, c: T)
    // where
    //     T: Pausable + Sync + Send + 'trait_async;
}

#[trait_async]
impl Pausable for Objeto {
    fn id(&self) -> u64 {
        self.id
    }
    async fn hold(&self, t: u64) {
        let _ = t;
        todo!("Implementar Funcion Hold")
    }

    async fn passivate<'a>(&'a mut self, mut channel: mpsc::Sender<(Command<'a>, oneshot::Sender<Response>)>) {
        debug!("Passivate ID: {}", self.id);
        let mut result;
        let (tx, rx) = oneshot::channel();
        loop {
            channel
                .send((Command::Dormir(&self), tx))
                .await
                .ok()
                .unwrap();
            result = rx.await.unwrap();
            if let Response::Romper = result {
                break;
            }
        }
    }

    async fn activate<'a>(&mut self, c: &'a Objeto, mut channel: mpsc::Sender<(Command<'a>, oneshot::Sender<Response>)>) {
        debug!("Activate ID: {} -> {}", self.id, c.id());
        // async fn activate<T>(&self, c: T) {
        // where
        //     T: Pausable + Send + Sync + 'trait_async,s
        let mut result;
        loop {
            // dbg!(result);
            let (tx, rx) = oneshot::channel();
            channel
                .send((Command::Despertar(&c), tx))
                .await
                .ok()
                .unwrap();
            result = rx.await.unwrap();
            if let Response::Romper = result {
                break;
            }
        }
    }
}

// mpsc::Sender<(Command, oneshot::Sender<Response>)>