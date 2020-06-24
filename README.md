# RustySim
## Rama que intenta usar referencias

### El programa actualmente presenta estos errores que no se como resolver
```
error[E0597]: `obj_1` does not live long enough
  --> src\main.rs:56:9
   |
52 |     let clone = transmisor.clone();
   |         ----- lifetime `'1` appears in the type of `clone` 
...
56 |         obj_1.passivate(clone).await;
   |         ^^^^^-----------------
   |         |
   |         borrowed value does not live long enough
   |         argument requires that `obj_1` is borrowed for `'1`
57 |     }));
   |     - `obj_1` dropped here while still borrowed

error[E0597]: `obj_1` does not live long enough
  --> src\main.rs:54:21
   |
54 |       let obj_1_ref = &obj_1;
   |                       ^^^^^^ borrowed value does not live long enough
...
61 |       handles.push(tokio::spawn(async move {
   |  _______________________________-
62 | |         let obj_2 = Objeto::new(2);
63 | |         obj_2.activate(*&obj_1_ref, clone).await;
64 | |     }));
   | |_____- argument requires that `obj_1` is borrowed for `'static`     
...
69 |   }
   |   - `obj_1` dropped here while still borrowed

error[E0505]: cannot move out of `obj_1` because it is borrowed
  --> src\main.rs:55:42
   |
54 |       let obj_1_ref = &obj_1;
   |                       ------ borrow of `obj_1` occurs here
55 |       handles.push(tokio::spawn(async move {
   |  __________________________________________^       
56 | |         obj_1.passivate(clone).await;
   | |         ----- move occurs due to use in generator
57 | |     }));
   | |_____^ move out of `obj_1` occurs here
...
61 |       handles.push(tokio::spawn(async move {
   |  _______________________________-
62 | |         let obj_2 = Objeto::new(2);
63 | |         obj_2.activate(*&obj_1_ref, clone).await;
64 | |     }));
   | |_____- argument requires that `obj_1` is borrowed for `'static`
```
### Quizas con otro diseño se puedan utilizar referencias
