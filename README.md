# RustySim

## Estado actual: No funcionando.

La idea es crear una libreria de simulación utilizando async/await
Idealmente permitiriamos la utilización de cualquier runtime
Donde el usuario implemente un trait indicando que su estructura
Es una entidad a simular y alli implemente el comportamiento que esta deba tener

De forma que una vez definido el comportamiento, el usuario active todas las entidades
Posteriormente active System (entidad encargada de llevar a cabo la simulación) para
que la simulación se realice.

