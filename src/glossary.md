# Glossary

Every technical word in this book, in plain language. Lessons link here the first time they use a
word. Terms are in A–Z order.

### API

A set of addresses a program answers, so that other programs — a website, a phone app, another
service — can ask it for data or ask it to do something (short for Application Programming
Interface).
**First used in:** [Introduction](introduction.md)

### Async

Code that can pause while it waits for something slow (the network, the [database](#database)…) so
the computer can do other work in the meantime. In Rust you mark such a function `async fn` and
pause it with `.await`.
**First used in:** [Tour of the stack](part-0-start/tour-of-the-stack.md)

### Backend

The part of a website or app that runs on a [server](#server) instead of on your own device: it
stores data, checks rules, and sends answers back. Think of a restaurant kitchen — customers never
walk into it, but every order goes through it before food comes out.
**First used in:** [How to use this book](part-0-start/how-to-use-this-book.md)

### Client

A program that asks for information or asks for something to be done, and shows the answer to a
person. A web browser or a phone app is a client; it talks to a [server](#server).
**First used in:** [How a web backend works](part-0-start/how-a-web-backend-works.md)

### Container

A small, sealed-off box that runs a program together with its own copy of everything that program
needs, kept separate from the rest of your computer. [Docker](#docker) starts, stops and manages
containers.
**First used in:** [Your toolbox](part-0-start/your-toolbox.md)

### Crate

A package of Rust code that you can add to your own project, written by you or by someone else. It
is Rust's word for what other languages call a library or a package.
**First used in:** [Tour of the stack](part-0-start/tour-of-the-stack.md)

### Database

A program that stores data on disk in an organized way, so the data survives after your program
stops, and lets you search, add, change and delete it safely. This book uses a database called
Postgres.
**First used in:** [Your toolbox](part-0-start/your-toolbox.md)

### Docker

A tool that runs programs inside [containers](#container), so that everyone runs the exact same
setup on their own computer, no matter what is already installed there.
**First used in:** [Your toolbox](part-0-start/your-toolbox.md)

### Endpoint

One specific address that a [server](#server) understands and answers, such as `/users` or
`/health`. Each endpoint usually does one job.
**First used in:** [How a web backend works](part-0-start/how-a-web-backend-works.md)

### Environment variable

A named value that lives outside your code — set on the computer, or in a settings file — that your
program can read when it starts. This book stores the [database](#database)'s location and password
in environment variables, so they are never written directly into the code.
**First used in:** [Your toolbox](part-0-start/your-toolbox.md)

### Framework

A set of tools and rules that handles the repeated parts of a task, such as receiving a
[request](#request) and matching it to a [route](#route), so you write only the parts that are
specific to your app. Axum is the framework this book uses to build a [backend](#backend).
**First used in:** [Tour of the stack](part-0-start/tour-of-the-stack.md)

### Handler

The function that runs when a [request](#request) matches a [route](#route); it receives the
request's data and returns the [response](#response).
**First used in:** [Tour of the stack](part-0-start/tour-of-the-stack.md)

### HTTP

The set of rules computers follow to ask for and send back information over the web (short for
HyperText Transfer Protocol). Every [request](#request) and [response](#response) in this book
travels using HTTP.
**First used in:** [How a web backend works](part-0-start/how-a-web-backend-works.md)

### JSON

A plain-text way of writing structured data, using `{ }` for an object and `[ ]` for a list, that
both people and programs can read. Most [request](#request) and [response](#response) bodies in
this book are JSON.
**First used in:** [How a web backend works](part-0-start/how-a-web-backend-works.md)

### Localhost

A name that always means "this computer" — used when a program on your machine talks to a
[server](#server) that is also running on your machine, without going out to the internet.
**First used in:** [Your toolbox](part-0-start/your-toolbox.md)

### ORM

A library that lets you work with [database](#database) rows as normal Rust structs instead of
writing [SQL](#sql) strings by hand (short for Object-Relational Mapper). SeaORM is the ORM this
book uses.
**First used in:** [Tour of the stack](part-0-start/tour-of-the-stack.md)

### Port

A numbered door on a computer; one program listens behind each door, e.g. our server on 3000 and
Postgres on 5433.
**First used in:** [Your toolbox](part-0-start/your-toolbox.md)

### Request

The message a [client](#client) sends to ask a [server](#server) for something, or ask it to do
something — such as "give me user 4" or "create this order".
**First used in:** [How a web backend works](part-0-start/how-a-web-backend-works.md)

### Response

The message a [server](#server) sends back to the [client](#client) that made a
[request](#request), carrying a [status code](#status-code) and usually some data.
**First used in:** [How a web backend works](part-0-start/how-a-web-backend-works.md)

### REST

A common style for designing a [backend](#backend) where each [endpoint](#endpoint) stands for one
"thing" (like a user or an order), and you act on it using the request's method — GET to read, POST
to create, and so on.
**First used in:** [How a web backend works](part-0-start/how-a-web-backend-works.md)

### Route

A rule that pairs one address and one HTTP method (like GET `/users`) with the
[handler](#handler) that should answer it.
**First used in:** [Tour of the stack](part-0-start/tour-of-the-stack.md)

### Runtime

The program running underneath your code that carries out [async](#async) work, deciding when each
paused task picks back up. Tokio is the runtime this book uses.
**First used in:** [Tour of the stack](part-0-start/tour-of-the-stack.md)

### Server

A program that waits for [request](#request)s and sends back [response](#response)s. It keeps
running in the background, ready for the next request at any time.
**First used in:** [How a web backend works](part-0-start/how-a-web-backend-works.md)

### SQL

The language used to ask a [database](#database) questions and give it instructions, such as "add
this row" or "find every order from this user" (short for Structured Query Language).
**First used in:** [Your toolbox](part-0-start/your-toolbox.md)

### Status code

A short number a [server](#server) sends back with every [response](#response) to say what
happened, such as `200` for success or `404` for "not found".
**First used in:** [How a web backend works](part-0-start/how-a-web-backend-works.md)
