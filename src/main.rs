use tokio::{
    // select,
    time::{self, Duration},
    sync::{mpsc, oneshot},
};
use trait_async::trait_async;

#[tokio::main]
async fn main() {
    // let obj_1 = Objeto::new();
    // let obj_2 = Objeto::new();
    // println!("Objeto 1 hace Hold por 5 segundos");
    // obj_1.hold(5).await;
    // println!("Objeto 2 hace Hold por 3 segundos");
    // obj_2.hold(3).await;
    // // obj_1.passivate().await;
    // // obj_2.activate(&obj_1).await;
    // let mut delay = time::delay_for(Duration::from_millis(50));

    // loop {
    //     select! {
    //         _ = &mut delay => {
    //             println!("Operation Timeout");
    //             break;
    //         }
    //         _ = some_async_work() => {
    //             println!("Some Async Work Completed");
    //         }
    //     }
    // }
    
    let (tx, mut rx) = mpsc::channel::<oneshot::Sender<()>>(1);

    let mut handles = vec![];
    tokio::spawn(async move {
        while let Some(response) = rx.recv().await {
            let response = response.send(());
            match response {
                Ok(_) => println!("Mensaje enviado"),
                Err(e) => eprintln!("Error mandando el mensaje\nmotivo: {:?}", e),
            }
        }
    });

    for _ in 0..10 {
        let mut obj_1 = Objeto::new(tx.clone());
        handles.push(tokio::spawn(async move {
            obj_1.passivate().await;
        }));
    }
    

    for handle in handles.drain(..){
        // let result = handle.await;
        // match result {
        //     Ok(_) => println!("Handle resolvio OK"),
        //     Err(e) => eprintln!("Handle fallo con motivo: {:?}", e),
        // };
        handle.await.unwrap();
    }

    

    // let obj_3 = Objeto::new();
    // task::spawn(async move {
    //     obj_3.passivate();
    // });


    
}

// struct Controlador {
//     cola: Vec<oneshot::Sender<()>>
// }

// async fn some_async_work() {
//     println!("hee hee");
// }

struct Objeto {
    channel: mpsc::Sender<oneshot::Sender<()>>
}

impl Objeto {
    fn new(channel: mpsc::Sender<oneshot::Sender<()>>) -> Self {
        Self {
            channel,
        }
    }
}

#[trait_async]
trait Pausable {
    async fn hold(&self, t: u64);
    async fn passivate(&mut self);
    // async fn activate(&self, c: &Objeto);
    async fn activate<T>(&self, c: T)
    where
        T: Pausable + Sync + Send + 'trait_async;
}

#[trait_async]
impl Pausable for Objeto {
    async fn hold(&self, t: u64) {
        time::delay_for(Duration::from_secs(t)).await;
        println!("{} segundos han pasado", t);
    }

    async fn passivate(&mut self) {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.channel.send(resp_tx).await.ok().unwrap();
        let _ = resp_rx.await.unwrap();
    }

    // async fn activate(&self, c: &Objeto);
    async fn activate<T>(&self, c: T)
    where
        T: Pausable + Send + Sync + 'trait_async,
    {
        let _ = c;
        todo!("Hacer la wea de funcion activate");
    }
}
