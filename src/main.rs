use tokio::sync::{mpsc, oneshot};
use trait_async::trait_async;
use hashbrown::HashMap;

#[macro_use]
extern crate log;


#[tokio::main]
async fn main() {
    env_logger::init();
    info!("Iniciando Programa");
    let (transmisor, mut receptor) = mpsc::channel::<(Command, oneshot::Sender<u64>)>(1);
    tokio::spawn(async move {
        let mut procesos = HashMap::new();
        let mut resultado;
        loop {
            resultado = receptor.recv().await;
            match resultado {
                Some((command, response)) => {
                    match command {
                        Command::Dormir(id) => { 
                            debug!("Llego command dormir para id: {}", id);
                            let _ = procesos.insert(id, response); 
                        },
                        Command::Despertar(id) => { 
                            debug!("Llego command despertar para id: {}", id);
                            let resultado = procesos.remove(&id);
                            match resultado {
                                Some(channel) => { 
                                    debug!("Mandando respuestas!");
                                    channel.send(42).unwrap(); 
                                    response.send(0).unwrap();
                                },
                                None => { 
                                    warn!("Proceso con ID: {} no esta durmiendo", id); 
                                    response.send(42).unwrap();
                                },
                            };
                        },
                    };
                },
                None => {
                    debug!("Todos los recievers han sido dropeados, rompiendo loop");
                    break;
                },
            };
        }

    });

    let mut handles = vec![];

    let clone = transmisor.clone();
    handles.push(tokio::spawn(async move {
        let mut obj_1 = Objeto::new(clone, 1);
        obj_1.passivate().await;
    }));

    let clone = transmisor.clone();
    handles.push(tokio::spawn(async move {
        let mut obj_2 = Objeto::new(clone, 2);
        obj_2.activate(1).await;
    }));

    for handle in handles.drain(..) {
        handle.await.unwrap();
    }
}
// Representacion de la corrutina
// Tiene un channel para enviar cosas al controlador
struct Objeto {
    channel: mpsc::Sender<(Command, oneshot::Sender<u64>)>,
    id: u64,
}

impl Objeto {
    fn new(
        channel: mpsc::Sender<(Command, oneshot::Sender<u64>)>,
        id: u64,
    ) -> Self {
        Self {
            channel,
            id,
        }
    }
}

enum Command {
    // Incrementar{id:u64, counter: u64},
    Dormir(u64),
    Despertar(u64),
}

#[trait_async]
trait Pausable {
    async fn hold(&self, t: u64);
    async fn passivate(&mut self);
    async fn activate(&mut self, c: u64);
    // async fn activate<T>(&self, c: T)
    // where
    //     T: Pausable + Sync + Send + 'trait_async;
}

#[trait_async]
impl Pausable for Objeto {
    async fn hold(&self, t: u64) {
        let _ = t;
        todo!("Implementar Funcion Hold")
    }

    async fn passivate(&mut self) {
        debug!("Passivate ID: {}", self.id);
        // let mut counter: u64 = 0;
        //     dbg!(counter);
        // let (resp_tx, resp_rx) = oneshot::channel::<u64>();
        // self.channel.send((counter, resp_tx)).await.unwrap();
        // counter = resp_rx.await.unwrap();

        let (tx, rx) = oneshot::channel();
        self.channel.send((Command::Dormir(self.id), tx)).await.ok().unwrap();
        let _ = rx.await.unwrap();
        // let result = rx.await.unwrap();
        // dbg!(result);
        // self.channel.send((Command::Incrementar{id: 1,counter}, tx)).await.ok().unwrap();
    }

    async fn activate(&mut self, c: u64) {
        debug!("Activate ID: {} -> {}", self.id, c);
        // async fn activate<T>(&self, c: T) {
        // where
        //     T: Pausable + Send + Sync + 'trait_async,s
        // {
        // let _ = c;
        // todo!("Implementar Funcion Activate");
        let mut result = 1;
        loop {
            // dbg!(result);
            if result == 0 { break; }
            let (tx, rx) = oneshot::channel();
            self.channel.send((Command::Despertar(c), tx)).await.ok().unwrap();
            result = rx.await.unwrap();
        }
        

        // let result = rx.await;
        // match result {
        //     Ok(_) => info!("Activate Termino con exito"),
        //     Err(e) => eprintln!("Fracaso, why?: {:?}", e),
        // };
    }
}
