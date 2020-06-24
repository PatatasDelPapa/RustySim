# RustySim

### Esta parte presenta problemas con lifetimes
### Command necesita enviar una referencia del objeto al que se le realiza el command (En este caso dormir)
### Pero no podemos hacer que el objeto actual mande una referencia de si mismo a traves del channel
```
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
```
### Quizas con otro diseño se puedan utilizar referencias
