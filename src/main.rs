#![forbid(unsafe_code)]

/* LISTA DE TODO:

    Calcular bien el tiempo.
    Hacer que la simulacion funcione
        No haga deadlock
        System siempre ande extrayendo eventos de la FEL
        System escuche a nuevos eventos, opciones son:
            - select!
            - try_recv()
    Mejorar la estructura del proyecto
        Ver como tener un lib.rs y main.rs en un mismo proyectos

*/

// Seccion de imports
use async_trait::async_trait;
use priority_queue::PriorityQueue;
// use tokio::sync::{mpsc, oneshot};
use flume::{Receiver, Sender};
use tracing::{debug, error, info, instrument, trace};

// Type Alias

type Message = (Command, Sender<()>);
type Channel = Sender<Message>;
type Id = u32;
type Time = u32;

// Import de modulos propios
mod utils;

struct Objeto {
    channel: Channel,
    id: u32,
}

impl Objeto {
    fn new(channel: Channel, id: u32) -> Self {
        Self { channel, id }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum Command {
    Hold(Id, Time),
    Passivate(Id),
    Activate(Id),
    Advance,
}

struct System {
    // Channel para recibir commands de las rutinas
    reciever: Receiver<Message>,

    // Id usado para que system se reconosca a si mismo en la FEL
    id: Id,

    // Lista que mantiene los objetos en estado Passivate
    // Los guarda como un HashMap K = u32, V = Sender
    // Sender es un channel utilizado para reactivar la ejecucion del objeto.
    passivate_list: hashbrown::HashMap<u32, Sender<()>>,

    // Lista de eventos futuros que guarda la id del objeto y la prioridad donde el numero mas pequeño
    // es aquel que tiene mayor prioridad (eso significa el uso de std::cmp::Reverse)
    future_event_list: PriorityQueue<u32, std::cmp::Reverse<u32>>,

    // Lista auxiliar para asociar el id con el channel utilizado para reanudad la ejecución
    // del objeto con la id ingresada
    future_event_list_aux: hashbrown::HashMap<u32, Sender<()>>,

    // Tiempo de simulación.
    time: Time,
}

impl System {
    fn new(reciever: Receiver<Message>, id: Id) -> Self {
        let passivate_list = hashbrown::HashMap::<u32, Sender<()>>::new();
        let future_event_list = PriorityQueue::<u32, std::cmp::Reverse<u32>>::new();
        let future_event_list_aux = hashbrown::HashMap::<u32, Sender<()>>::new();
        let time = 0;

        Self {
            reciever,
            id,
            passivate_list,
            future_event_list,
            future_event_list_aux,
            time,
        }
    }

    #[instrument(skip(self))]
    async fn system(&mut self, max: Time) {
        self.future_event_list.push(0, std::cmp::Reverse(max));

        use flume::TryRecvError;
        loop {
            match self.reciever.try_recv() {
                Ok((command, sender)) => {
                    debug!("Tiempo de simulación actual: {}", self.time);
                    // Si el tiempo de simulación es mayor al tiempo maximo de simulación, la simulación termina
                    if self.time > max {
                        break;
                    }
                    match command {
                        Command::Hold(id, t) => {
                            trace!("Llego command Hold para id: {}", id);
                            // Una lista auxiliar para asociar una id con el channel utilizado para reanudar la ejecucion del objeto
                            self.future_event_list_aux.insert(id, sender);
                            // El tiempo siempre aumenta y se busca obtener el que tiene menor tiempo por ello usamos std::cmp::Reverse
                            self.future_event_list.push(id, std::cmp::Reverse(t));
                        }
                        Command::Passivate(id) => {
                            self.passivate_list.insert(id, sender);
                        }
                        Command::Activate(id) => {
                            trace!("Llego command Activate para id: {}", id);
                            match self.passivate_list.remove(&id) {
                                Some(channel) => {
                                    trace!(
                                        "Enviando respuestas para command Activate con id: {}",
                                        id
                                    );
                                    // Se despierta al objeto en la lista de passivates.
                                    channel.send(()).unwrap();
                                    // El objeto que solicito esta accion deja de esperar.
                                    sender.send(()).unwrap();
                                }
                                None => {
                                    error!("Id: {} no se encuentra haciendo un Passivate", id);
                                    panic!("Id: {} no se encuentra haciendo un Passivate", id)
                                }
                            }
                        }
                        // Aqui es donde mas tengo dudas.
                        Command::Advance => {
                            match self.future_event_list.pop() {
                                Some((id, _)) if id == self.id => {
                                    info!("Simulación terminada");
                                    break;
                                }
                                Some((id, t)) => {
                                    self.time += t.0;
                                    let channel = self.future_event_list_aux.remove(&id)
                                        .expect("Cualquier id ingresada en la FEL debe existir en la FEL_aux");
                                    channel.send(()).unwrap();
                                    // TODO: Debera el que envia un advance esperar a que esta rutina le responda?
                                    //       Probablemente si
                                    sender.send(()).unwrap();
                                }
                                None => {
                                    panic!("La FEL se encuentra vacia, realizar un advance en estas condiciones es un error");
                                }
                            }
                        }
                    };
                }
                Err(e) if e == TryRecvError::Empty => {}
                Err(e) => {
                    error!("Todos los senders han sido cerrados no se puede seguir recibiendo mensajes. Error = {}", e);
                    panic!("Los senders no pueden ser detenidos antes que system");
                }
            }
        }
        info!("El tiempo simulado es: {}", self.time);
    }
}

// Some((command, sender))

// TODO:
//  Probablemente esta funcion desaparesca y System se encargue por su cuenta (es decir sin channels)
//  de seguir extrayendo eventos de la FEL.
// async fn _advance(channel: Sender<Message>) {
//     trace!("Ha iniciado un Advance");
//     // Creamos el command que recive system
//     let command = Command::Advance;
//     // Creamos los channels ocupados para realizar la operación
//     let (sender, reciever) = flume::bounded(1);
//     // Preparamos el command y el channel que system ocupa para responder en una sola variable
//     let statement = (command, sender);
//     // Lo enviamos a system usando el channel tipo sender del objeto
//     let result = channel.send_async(statement).await;
//     debug!(
//         "Se ha enviado un Advance. Completo exitosamente?: {} ",
//         result.is_ok()
//     );
//     // Si enviar el mensaje falla crashear el programa.
//     if let Err(e) = result {
//         error!("Mandar Activate fallo! Error: {}", e);
//         panic!("Mandar Activate fallo! Error: {}", e);
//     }
//     // El objeto se queda esperando a que system le responda
//     trace!("Operacion: Advance | Estado: Esperando a que system responda");
//     reciever
//         .into_recv_async()
//         .await
//         .expect("Esperar a que system responda no debiera de fallar");
//     // TODO: Debera este objeto esperar a que system le responda?
//     //       Puedo hacer que esta funcion devuelva el channel y dejar que el usuario decida.
//     //       Por ahora lo dejare esperando.
//     debug!("Se ha terminado Advance");
// }
#[async_trait]
trait Simulable {
    async fn init(&self);
    async fn inner_body(&self);
    async fn hold(&self, t: Time);
    async fn passivate(&self);
    // Para volver esto generico probablemente
    // necesite un bound parecido a este
    // T: Simulable + Sync + Send + 'trait_async;
    async fn activate(&self, other: Id);
}

#[async_trait]
impl Simulable for Objeto {
    #[instrument(skip(self))]
    async fn hold(&self, time: Time) {
        trace!("Id: {} Ha iniciado un Hold con tiempo: {}", self.id, time);
        // Creamos el command que recive system
        let command = Command::Hold(self.id, time);
        // Creamos los channels ocupados para realizar la operación
        let (sender, reciever) = flume::bounded::<()>(1);
        // Preparamos el command y el channel que system ocupa para responder en una sola variable
        let statement = (command, sender);
        // Lo enviamos a system usando el channel tipo sender del objeto
        let result = self.channel.send_async(statement).await;
        debug!(
            "Id: {} Ha enviado un Hold con tiempo: {}. Completo Exitosamente?: {}",
            self.id,
            time,
            result.is_ok()
        );
        // Si enviar el mensaje falla crashear el programa.
        if let Err(e) = result {
            error!("Mandar Hold fallo! Error: {}", e);
            panic!("Mandar Hold fallo! Error: {}", e);
        }
        // El objeto se queda esperando a que system le responda.
        trace!(
            "Id: {} | Operacion: Hold por tiempo: {} | Estado: Esperando a que system responda",
            self.id,
            time
        );
        reciever
            .into_recv_async()
            .await
            .expect("Esperar a que system responda no debiera de fallar");
        // TODO: Debera este objeto esperar a que system le responda?
        //       Puedo hacer que esta funcion devuelva el channel y dejar que el usuario decida.
        //       Por ahora lo dejare esperando.
        debug!("Id: {} ha terminado de realizar Hold", self.id);
    }

    #[instrument(skip(self))]
    async fn passivate(&self) {
        trace!("Id: {} Ha iniciado un Passivate", self.id);
        // Creamos el command que recive system
        let command = Command::Passivate(self.id);
        // Creamos los channels ocupados para realizar la operación
        let (sender, reciever) = flume::bounded::<()>(1);
        // Preparamos el command y el channel que system ocupa para responder en una sola variable
        let statement = (command, sender);
        // Lo enviamos a system usando el channel tipo sender del objeto
        let result = self.channel.send_async(statement).await;
        debug!(
            "Id: {} ha enviado un Passivate. Completo exitosamente?: {} ",
            self.id,
            result.is_ok()
        );
        // Si enviar el mensaje falla crashear el programa.
        if let Err(e) = result {
            error!("Mandar Passivate fallo! Error: {}", e);
            panic!("Mandar Passivate fallo! Error: {}", e);
        }
        // El objeto se queda esperando a que system le responda.
        trace!(
            "Id: {} | Operacion: Passivate | Estado: Esperando a que system responda",
            self.id
        );
        reciever
            .into_recv_async()
            .await
            .expect("Esperar a que system responda no debiera de fallar");
        // TODO: Debera este objeto esperar a que system le responda?
        //       Puedo hacer que esta funcion devuelva el channel y dejar que el usuario decida.
        //       Por ahora lo dejare esperando.
        debug!("Id: {} Ha terminado de realizar Passivate", self.id);
    }

    #[instrument(skip(self))]
    async fn activate(&self, other_id: Id) {
        trace!(
            "Id: {} Ha iniciado un Activate para Id: {}",
            self.id,
            other_id
        );
        // Creamos el command que recive system
        let command = Command::Activate(other_id);
        // Creamos los channels ocupados para realizar la operación
        let (sender, reciever) = flume::bounded::<()>(1);
        // Preparamos el command y el channel que system ocupa para responder en una sola variable
        let statement = (command, sender);
        // Lo enviamos a system usando el channel tipo sender del objeto
        let result = self.channel.send_async(statement).await;
        debug!(
            "Id: {} ha enviado un Activate. Completo exitosamente?: {} ",
            self.id,
            result.is_ok()
        );
        // Si enviar el mensaje falla crashear el programa.
        if let Err(e) = result {
            error!("Mandar Activate fallo! Error: {}", e);
            panic!("Mandar Activate fallo! Error: {}", e);
        }
        // El objeto se queda esperando a que system le responda
        trace!(
            "Id: {} | Operacion: Activate hacia Id {} | Estado: Esperando a que system responda",
            self.id,
            other_id
        );
        reciever
            .into_recv_async()
            .await
            .expect("Esperar a que system responda no debiera de fallar");
        // TODO: Debera este objeto esperar a que system le responda?
        //       Puedo hacer que esta funcion devuelva el channel y dejar que el usuario decida.
        //       Por ahora lo dejare esperando.
        debug!("Id: {} ha terminado de realizar Activate", self.id);
    }

    #[instrument(skip(self))]
    async fn init(&self) {
        self.hold(0).await;
    }

    #[instrument(skip(self))]
    async fn inner_body(&self) {
        todo!()
        // loop {
        /*
            Aqui va la implementación del objeto
            por ejemplo:
                self.hold(5).await;
                self.passivate().await;
                self.activate(2).await;
        */
        // }
    }
}

#[tokio::main]
async fn main() {
    utils::log_init();
    let (sender, reciever) = flume::bounded::<Message>(1);
    tokio::spawn(async move {
        System::new(reciever, 0).system(50).await;
    });
    let clone = sender.clone();
    let _ = Objeto::new(clone, 1);
    let _ = sender;

    println!("Hello World");
}
