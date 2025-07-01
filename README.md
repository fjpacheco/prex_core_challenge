[![Rust](https://github.com/fjpacheco/prex_core_challenge/actions/workflows/rust.yaml/badge.svg)](https://github.com/fjpacheco/prex_core_challenge/actions/workflows/rust.yaml) [![codecov](https://codecov.io/gh/fjpacheco/prex_core_challenge/graph/badge.svg?token=JMvMlIJk3L)](https://codecov.io/gh/fjpacheco/prex_core_challenge)

# Prex Core Challenge

Microservicio de pagos (mini procesador de pagos) en **Rust**, implementado con una **arquitectura hexagonal**. Permite gestionar clientes y sus balances, acreditando, debitando, consultando y exportando saldos a través de una API REST. La solución está diseñada para que la persistencia y exportación de datos sea flexible: por defecto, los datos se mantienen en memoria y se exportan a archivos. El proyecto prioriza buenas prácticas de ingeniería de software, aplicando principios de buen diseño y testeo exhaustivo en la lógica de negocio, obteniendo una alta cobertura de pruebas. Ideal como ejemplo de arquitectura limpia, escalable y mantenible en Rust.

<details>
<summary><b>Definición de requerimientos</b></summary>

## Definición de requerimientos

Este proyecto es una implementación para el desafío técnico de Prex. El mismo corresponde a un “mini procesador de pagos” con la capacidad de llevar el saldo de los clientes en memoria y persistirlos (cuando se solicite) en un archivo. Esto es mediante la implementación de un micro servicio que exponga una API REST al usuario (al referirnos a un procesador de pagos, el usuario del mismo es un emisor de tarjetas y/o servicios de pago, no así el cliente final), a través de la cual pueda llevar un registro del saldo de sus clientes.

El servicio recibirá instrucciones a través de su API REST para acreditar o debitar saldo a los clientes. Cada cliente deberá ser creado inicialmente mediante el servicio "new_client" y luego podrá recibir tanto débitos (resta al saldo) como créditos (suma al
saldo).

Se podrá consultar el saldo e información de los clientes mediante el servicio “client_balance”.

El saldo y toda la base de clientes deberá poder persistirse en un archivo mediante el servicio "store_balances". El servicio deberá implementar los siguientes endpoints:


### POST “new_client”

El servicio debe recibir el siguiente body
```json
{
    "name": <STRING>, 
    "birth_date": <NAIVEDATE>, 
    "document": <STRING>, 
    "country": <STRING>
}
```

El número de documento no se puede repetir en la base de clientes.

El servicio debe devolver como respuesta, el ID del cliente creado (se debe generar internamente y ser único).

Realizar las validaciones que se crean necesarias.

### POST “new_credit_transaction”

El servicio debe recibir el siguiente body (JSON):

```json
{
    "client_id": <INT>,
    "amount": <DECIMAL>
}
```

Debe poder encontrar al cliente (o devolver un error adecuado) mediante su ID e incrementar su saldo en el monto especificado. El servicio debe devolver como respuesta el nuevo saldo del cliente. Realizar las validaciones que se crean necesarias.

### POST “new_debit_transaction”

El servicio debe recibir el siguiente body (JSON):

```json
{
    "client_id": <INT>,
    "amount": <DECIMAL>
}
```
Debe poder encontrar al cliente (o devolver un error adecuado) mediante su ID y decrementar su saldo en el monto especificado.

El servicio debe devolver como respuesta el nuevo saldo del cliente. Realizar las validaciones que se crean necesarias.

### POST “store_balances”

Este servicio no recibe ningún input de información.

Debe persistir en archivo todos los IDs de clientes y sus balances. Este servicio debe limpiar los balances de los clientes (sólo su balance), de forma que todos los clientes queden con balance 0 en memoria, pero sus balances previos almacenados en el archivo.

El formato del nombre del archivo deberá ser “DDMMYYYY_FILE COUNTER.DAT”, por ejemplo: “01122023_10.DAT” donde “01122023” corresponde a la fecha (01/12/2023) y el “10” identifica a que es el archivo n° 10 generado (desde el inicio del servicio).

Se presenta a continuación un ejemplo de formato del archivo:

```text
1234567890 100
1234567891 200.25
1234567892 300.50

ID_CLIENTE<espacio>BALANCE<Salto 
de                           línea>
ID_CLIENTE<espacio>BALANCE<Salto
de                           línea>
ID_CLIENTE<espacio>BALANCE<Salto
de                           línea>
ID_CLIENTE<espacio>BALANCE<Salto
de                           línea>
```

### GET “client_balance”

El servicio debe recibir el ID de cliente mediante un parámetro de URL (user_id) y devolver un JSON conteniendo la información del cliente y su balance. El formato del mismo queda a criterio del lector.

</details>

## Ejecución

### Requisitos

- Lenguaje de Rust (v1.71 o superior). 

Este proyecto fue desarrollado en la versión 1.88.0.

### Ejecución del servicio

Para iniciar el servicio de forma local, se debe ejecutar el siguiente comando:

```bash
cargo run
```

Para ejecutar el servicio en modo de desarrollo, y en caso de disponer la tool de `cargo-watch`, se puede ejecutar el siguiente comando para que se reinicie automáticamente el servidor cuando se realicen cambios en el código:

```bash
cargo watch -x run --ignore "*.DAT"
```

Se ignoran los archivos `*.DAT` que genera el servicio para evitar un reinicio innecesario cuando se invoque el endpoint `store_balances`.

### Ejecución de tests

Para ejecutar los tests, se debe ejecutar el siguiente comando:

```bash
cargo test
```

En caso de disponer de forma local la tool de `cargo-llvm-cov`, se puede ejecutar el siguiente comando para obtener el coverage de los tests de forma local:

```bash
cargo llvm-cov --ignore-filename-regex "infrastructure|main.rs"
```

#### Cobertura de tests

La cobertura de tests cercana al 100% en la capa de **Domain** y **Application**. Esto se puede ver en [Codecov](https://app.codecov.io/gh/fjpacheco/prex_core_challenge).

### Variables de entorno

El servicio puede tomar las siguientes variables de entorno:

- `RUST_LOG`: Define el nivel de log del servicio. Por defecto es `info`.
- `HOST`: Define el host del servicio. Por defecto es `127.0.0.1`.
- `PORT`: Define el puerto del servicio. Por defecto es `8080`.
- `FILE_EXPORT_DIRECTORY`: Define el directorio donde se exportarán los archivos. Por defecto es `.` (en el mismo directorio de ejecución del servicio).

## Colección de Postman

Se adjunta una [Colección de Postman](./prex_core_challenge.postman_collection.json) para facilitar las pruebas de la API REST del servicio.

## Decisiones de diseño

### Arquitectura

Fue vital la definición de una arquitectura para este proyecto. A pesar de ser una versión "mini" del desafío, se consideró importante para demostrar el uso de buenas prácticas de la ingeniería de software, mostrando una proactividad e investigación en el desarrollo.

Se eligió una **arquitectura hexagonal**, ya que permite aislar la lógica del negocio de los detalles de infraestructura (como la persistencia y exportación de datos, comunicación con otros servicios, o la forma en que el servicio expone su API hacia el exterior), facilitando el testing, la mantenibilidad y la evolución del sistema.

En este caso, se separó el código en las 3 capas de la arquitectura hexagonal:

- **Domain**: Contiene modelos del dominio (como los DTOs, errores, entidades y valores con sus validaciones) que son las representaciones canónicas de la lógica del negocio. También incluye las traits/ports/interfaces (puntos de entrada y salida para la lógica del negocio).
- **Application**: Encapsula todas las dependencias necesarias encargadas de ejecutar la lógica de negocio con sus casos de uso, exponiéndose como el servicio core de la aplicación.
- **Infrastructure**: Implementa la lógica de infraestructura con los detalles concretos de los traits/ports/interfaces del modelo del dominio, exponiendo la API REST del servicio de aplicación, persistiendo datos en memoria y exportando datos a un archivo. El resultado de estas implementaciones también se lo conoce como los adapters de la arquitectura hexagonal.

Hay muchas variantes de la arquitectura hexagonal, y se puede encontrar una gran cantidad de recursos en internet para entenderla y aplicarla en diferentes contextos. En mi caso adopté la versión de arquitectura mencionada basandome en diversos recursos que adjunto en la sección de [Referencias](#referencias).

### Testing

Los tests se enfocaron en las capas de **Domain** y **Application**, principalmente con **testing unitario** mediante el uso del crate `mockall`.

En la capa de **Infrastructure** se decidió no agregar tests, ya que solo interactúa con detalles externos, fuera de la lógica del negocio con el core de la aplicación. Además, puede ser costoso testearla para este “mini” proyecto. Sin embargo, sería fundamental testear esta capa en caso de buscar implementar **tests de integración**.

### Persistencia de datos

Tal como se enuncian en los requerimientos, los datos de clientes y sus balances se persisten en memoria.

Pero dada la arquitectura definida, no sería complejo agregar una capa de persistencia con una base de datos como PostgreSQL/MySQL/MongoDB/etc. Esto se vio reflejado en el adapter de persistencia en memoria, donde en [in_memory.rs](src/infrastructure/outbound/in_memory.rs) se puede ver cómo se implementa la persistencia en memoria con un simple HashMap, pero que en los tests unitarios se puede ver cómo se implementa el adaptar mediante mocks, pero que también termina siendo en memoria con dos HashMap diferentes.

#### Uso sincrónico de Mutex

> [!IMPORTANT]
Dado que nuestro trait/port/interface de persistencia indica que los clientes y balances pueden ser accedidos concurrentemente por múltiples tareas asincrónicas (gracias al runtime de Tokio), debemos encontrar una forma segura de sincronizar el acceso mutable de los datos compartidos.

Para dicha persistencia se uso un [Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html) sincronico de la libreria estándar de Rust dado que es una [recomendación](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html#which-kind-of-mutex-should-you-use) dada por el mismo crate de `tokio`. 

El motivo se debe a que el [Mutex](https://doc.rust-lang.org/std/sync/struct.Mutex.html) sincrónico de la libreria estándar es mas performante que el [Mutex](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html) asincrónico de tokio para los casos donde solo se acceden a datos sin un acceso de recursos de I/O tales como una base de datos.

Además que en el adaptador/implementación de persistencia en memoria no se cruza con ningún acceso de recursos de I/O, ni tampoco se realiza un .await mientras se tiene tiene el lock del Mutex.

Si tuviéramos un recurso de I/O, como una base de datos, ahí sí conviene usar el Mutex asincrónico para proveer el acceso mutable compartido a esa conexión o pool de conexiones a la base de datos.

> [!TIP]
> Citando a [Alice Ryhl](https://www.linkedin.com/in/aliceryhl/), Maintainer de Tokio: "_You should only use an asynchronous lock if you need to .await something while the lock is locked. Usually, this is not necessary, and you should avoid using an asynchronous lock when you can. Asynchronous locks are a lot slower than blocking locks._"
>
> https://draft.ryhl.io/blog/shared-mutable-state/

### Exportación de datos

Los datos de clientes y sus balances se exportan a un archivo con extensión `.DAT`. 
 
Incluso la exportación de datos a un archivo podría ser reemplazada por una capa de exportación a un servicio de almacenamiento como S3/GCP/Azure/etc o publicarse en un servicio de mensajería como Kafka/RabbitMQ/etc. 

Cabe mencionar que si se reinicia el servidor y ya existían archivos con extensión `.DAT` en el directorio de ejecución, se continuará con el conteo de archivos, es decir, si ya existían 10 archivos, el siguiente archivo se llamará `01012025_11.DAT`.

Además que si no hay clientes para exportar, la API REST retorna un error.

### Validaciones adicionales 

#### Debito y Credito de balances 

Se tomó en cuenta la siguiente consideración para la validación de los requests:

- En el request para acreditar balances se validará que la cantidad a acreditar sea mayor a 0.
- En el request para debitar balances se validará que la cantidad a debitar sea menor a 0.

#### Límites de los campos

Se agregaron límites de longitudes máximos recibidos en los requests para evitar sobrecargar la información que se maneja en el servidor. Además que los campos no pueden ser vacíos e inválidos.

### Dominio: Clientes y Balances

Se decidió que el Cliente y el Balance sean dos entidades separadas dentro del mismo modelo del dominio, pero que están estrechamente relacionadas formando un todo como "Balance del Cliente" como servicio core final. 

Al separar dichas entidades se logra una mayor flexibilidad para agregar en el futuro más información al balance, cumpliendo con el principio de responsabilidad única de los principios SOLID puesto que el cliente solo se encargará de contener información del usuario y el balance se encargará de contener la información del balance del cliente. Pero incluso a la larga si llegan a crecer ambas entidades, y sus operaciones logran ser ser independientes, con la arquitectura definida se podría incluso separar en 2 dominios diferentes, uno para el cliente y otro para el balance. Aunque esto agregaría el coste de la comunicación entre ambos dominios, con el desafío de lograr la atomicidad entre ambas entidades. Dicha separación de dominios puede dar entrada a la separación en una arquitectura de microservicios, separando cada dominio en un microservicio diferente.

Como era a criterio del lector, en el servicio de `GET /client_balance` se decidió obtener toda la información del cliente y su balance para evitar que los consumidores de la API tengan que hacer 2 llamadas para obtener la información del cliente y luego la información del balance. Pero tal como se menciona en el requerimiento, según cómo crezcan dichas entidades, podría ser viable obtener solo información del cliente o solo información del balance con diferentes endpoints.

## Integración continua

Adicionalmente se agregó un workflow de CI para que en cada push a master se realicen las siguientes acciones y validaciones:

- Revisión de formato de código con [rustfmt](https://github.com/rust-lang/rustfmt).
- Revisión de lints de código con [clippy](https://github.com/rust-lang/rust-clippy).
- Build del proyecto correcto.
- Ejecución de tests.
- Generación de cobertura de tests y reporte en [Codecov](https://app.codecov.io/gh/fjpacheco/prex_core_challenge).

A futuro se podría agregar un workflow de CD (Continuous Deployment) para que en cada push a master se realice el despliegue del servicio en la nube y/o en un contenedor de Docker.

### Referencias

Algunos recursos por los que me vi obligado a explorar para entender más sobre la arquitectura hexagonal y sus miles de variantes + otros recursos:

- https://herbertograca.com/2017/11/16/explicit-architecture-01-ddd-hexagonal-onion-clean-cqrs-how-i-put-it-all-together/
- https://www.happycoders.eu/software-craftsmanship/hexagonal-architecture/
- https://www.howtocodeit.com
  - https://www.howtocodeit.com/articles/master-hexagonal-architecture-rust
    - https://github.com/howtocodeit/hexarch/tree/main
  - https://www.howtocodeit.com/articles/ultimate-guide-rust-newtypes
- https://medium.com/the-software-architecture-chronicles/ports-adapters-architecture-d19f2d476eca
- https://codeartify.substack.com/p/folder-structures
- https://medium.com/@oliveraluis11/arquitectura-hexagonal-con-spring-boot-parte-1-57b797eca69c
- https://miladezzat.medium.com/hexagonal-architecture-ports-and-adapters-pattern-5ad2421802ec
- https://dev.to/dyarleniber/hexagonal-architecture-and-clean-architecture-with-examples-48oi
- https://herbertograca.com/2017/09/14/ports-adapters-architecture/
- https://dpc.pw/posts/data-oriented-cleanandhexagonal-architecture-software-in-rust-through-an-example
- https://blog.allegro.tech/2020/05/hexagonal-architecture-by-example.html
- https://github.com/PacktPublishing/Designing-Hexagonal-Architecture-with-Java/tree/main
- https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html#which-kind-of-mutex-should-you-use
- https://draft.ryhl.io/blog/shared-mutable-state/
