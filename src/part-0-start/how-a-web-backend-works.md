# How a web backend works

> **Beginner** · Part 0 · Before you start

## By the end of this lesson

- You can explain what happens between typing a URL and seeing a result.
- You can read an HTTP request and response line by line.
- You know what JSON, REST and status codes are, and can send a request with curl.

## What & why

Clients, servers, HTTP requests and responses, JSON, REST and status codes — the ideas every later lesson uses.

You open a shop's app and tap a coffee mug. A second later you see its name and price. Something
happened in that second: your phone asked a computer somewhere for "product 42", and that computer
looked it up and answered. This lesson is about that conversation: what the question looks like,
what the answer looks like, and the rules both sides follow. Most of the Rust you write later in
this book reads a question like this or writes an answer like this, so it's worth seeing them with
your own eyes first.

Back to the restaurant from [How to use this book](how-to-use-this-book.md), with every role named:

| In the restaurant… | On the web… | What it is |
|---|---|---|
| The customer | The [**client**](../glossary.md#client) | The program that asks: a browser, a phone app, or curl in your terminal. |
| The order slip | The [**request**](../glossary.md#request) | The message the client sends: what it wants, written in a fixed format. |
| The kitchen | The [**server**](../glossary.md#server) | The program that waits for requests and answers them. Our [backend](../glossary.md#backend) runs here. |
| The cook for that dish | The [**handler**](../glossary.md#handler) | The one function in the server that deals with this kind of order. |
| The pantry | The [**database**](../glossary.md#database) | Where the data (the "ingredients") is stored between orders. |
| The plate | The [**response**](../glossary.md#response) | The message the server sends back, with the data the client asked for. |
| The waiter's note on the plate | The [**status code**](../glossary.md#status-code) | A number that says how it went: "served" (200), "we don't have that" (404), "the kitchen is on fire" (500). |

The rules for writing order slips and plates are called [**HTTP**](../glossary.md#http) (HyperText
Transfer Protocol). A *protocol* is an agreed set of rules for a conversation, like "say hello,
then order, then say thank you". Because every browser, phone and server follows the same HTTP
rules, any client can talk to any server.

Here's the whole trip when you type a **URL** (a full web address, such as
`https://shop.example.com/products/42`) into a browser:

1. The browser (the client) turns the URL into a request and sends it across the internet to the
   server.
2. The server reads the request and picks the handler for it.
3. The handler does the work, often asking the database for the data it needs.
4. The server sends a response: a status code and the data.
5. The browser reads the status code, then shows you the data.

The rest of this lesson zooms in on steps 1 and 4: what a request and a response really look like.

## The idea, slowly

### Step 1: a request, written out

An HTTP request is plain text. This is the whole request a shop app sends to ask for product 42,
with its price in US dollars:

```text
GET /products/42?currency=usd HTTP/1.1
Host: shop.example.com
Accept: application/json

```

### Line by line

`GET /products/42?currency=usd HTTP/1.1` (the *request line*)
- **What:** the first line says what the client wants. It has three parts separated by spaces:
  the **method** `GET`, the **target** `/products/42?currency=usd`, and the **version**
  `HTTP/1.1`.
- **Why:** the server reads this line first to decide what to do and which code should do it.
- **How:**
  - `GET` is the **method**, the verb of the request. `GET` means "give me something; don't change
    anything". You'll meet `POST` (create), `PUT` (replace), `DELETE` (delete) in Steps 4 and 6.
  - `/products/42` is the **path**: *which thing* you're talking about. Read it like folders:
    `/products` is the collection of all products, and the **path segment** `42` (one piece
    between slashes) picks product number 42 from it.
  - `?currency=usd` is the **query string**: everything after the `?`. It holds optional extra
    options as `name=value` pairs, joined with `&` when there are several
    (`?currency=usd&lang=en`). Here it says "show me the price in US dollars". The path says
    *what*; the query string says *how you'd like it*.
  - `HTTP/1.1` is the **version** of the HTTP rules the client is using. You'll also see
    `HTTP/2`, a newer version that means the same things but travels in a more compact form.
- **Remove it and…** the server has no idea what you want. A request without a request line is
  not a request.

`Host: shop.example.com`
- **What:** a **header**: one `Name: value` line of extra information about the request. This one
  names the website the request is for.
- **Why:** one server computer often hosts many websites. `Host` tells it which one you meant.
- **How:** headers come right after the request line, one per line, in any order. Header names
  don't care about upper or lower case: `Host` and `host` mean the same.
- **Remove it and…** an HTTP/1.1 server answers `400 Bad Request`, because the rules make `Host`
  compulsory.

`Accept: application/json`
- **What:** a header saying which format the client would like the answer in.
- **Why:** the same product could be sent as a web page or as data. This app wants data, in a
  format called JSON (Step 5).
- **How:** `application/json` is a **media type**, a standard name for a data format.
  `text/html` is a web page, `text/plain` is plain text, and `*/*` means "anything is fine".
- **Remove it and…** the server picks a format itself. Most backends send JSON anyway, so this is
  polite rather than required.

*(the empty line at the end)*
- **What:** one completely empty line. It's in the box above, even though you can't see it.
- **Why:** it marks "the headers are finished". Anything after it is the **body**: the request's
  data, such as a new product to save. This `GET` has no body, so the request ends here.
- **How:** the server reads header lines until it meets an empty one.
- **Remove it and…** the server keeps waiting for more headers, and the request never finishes.

So a request has five parts: **method, path (with its optional query string), version, headers,
body**. Keep that list in mind: every web [framework](../glossary.md#framework) (a library that
does the repeated work of reading requests and sending responses for you) hands you these five
things.

### Step 2: a response, written out

The server looks up product 42 and sends back this response:

```text
HTTP/1.1 200 OK
Content-Type: application/json

{"id":42,"name":"Mug","price":1299}
```

### Line by line

`HTTP/1.1 200 OK` (the *status line*)
- **What:** the version, the status code `200`, and a short human-readable phrase `OK`.
- **Why:** the client checks the status code *before* reading anything else. `200` means "it
  worked, here's what you asked for".
- **How:** the number is for programs, the phrase is for humans. Programs only look at the number.
  Step 7 lists the codes you'll use.
- **Remove it and…** it's not an HTTP response; the client reports a broken answer.

`Content-Type: application/json`
- **What:** a header saying what format the body is in.
- **Why:** the body is a row of characters. Without a label, the client can't know whether it's
  JSON, a web page or an image.
- **How:** the same media type names as `Accept` in Step 1. The request said "I'd like JSON"; the
  response says "this is JSON".
- **Remove it and…** many clients refuse to read the body as JSON, or show it as plain text.

*(the empty line)*
- **What:** the end of the headers, exactly as in the request.
- **Why:** it separates the headers from the body.
- **How:** everything after it is the body.
- **Remove it and…** the client would read the body as one more (broken) header.

`{"id":42,"name":"Mug","price":1299}`
- **What:** the **body**, the data itself: product 42 is called "Mug" and costs 1299.
- **Why:** this is what the app shows you. It's written in JSON, a text format for data (Step 5).
- **How:** `"price":1299` is the price in **cents**: 1299 cents is $12.99. Money is stored as a
  whole number of the smallest unit (cents, pence, paisa) because computers store numbers with a
  decimal point (*floating-point* numbers) in a way that can't hold most decimals exactly. Ask any
  programming language for `0.1 + 0.2` and you get `0.30000000000000004`. Tiny errors like that
  are fine for a temperature, but not for someone's bank balance. Whole numbers are always exact,
  so the book counts money in cents and turns it into `12.99` only when showing it to a person.
- **Remove it and…** the client gets a successful answer with nothing in it.

### Step 3: send a real request with curl

Enough reading. You installed curl in [Your toolbox](your-toolbox.md); now use it to send a real
request to **httpbin.org**, a free public server built for practising HTTP. Its `/get` address
answers every `GET` by describing the request it received.

```bash
curl -i https://httpbin.org/get
```

### Line by line

`curl -i https://httpbin.org/get`
- **What:** sends a `GET` request to `https://httpbin.org/get` and prints the response.
- **Why:** it's the fastest way to see a real response, with nothing in the way.
- **How:** curl sends `GET` unless you tell it otherwise. The URL holds three things: `https` is how to connect (HTTP, locked with encryption so nobody in between can
  read it), `httpbin.org` becomes the `Host` header, and `/get` is the path. `-i` (*include*)
  asks curl to print the status line and headers too, not only the body.
- **Remove it and…** (the `-i`) you see only the body, and miss the status code and headers.

### Run it

```bash
curl -i https://httpbin.org/get
```

```text
HTTP/2 200 
date: Sat, 26 Sep 2026 07:32:22 GMT
content-type: application/json
content-length: 254
server: gunicorn/19.9.0
access-control-allow-origin: *
access-control-allow-credentials: true

{
  "args": {}, 
  "headers": {
    "Accept": "*/*", 
    "Host": "httpbin.org", 
    "User-Agent": "curl/8.7.1", 
    "X-Amzn-Trace-Id": "Root=1-6ab77506-1a557de90831b0e2488637d8"
  }, 
  "origin": "203.0.113.7", 
  "url": "https://httpbin.org/get"
}
```

Lay it next to the response from Step 2; it's the same shape:

- `HTTP/2 200` is the status line. curl and httpbin agreed to use **HTTP/2**, which leaves out
  the `OK` phrase (programs only need the number). Same meaning: success.
- The lines from `date:` to `access-control-allow-credentials:` are headers. HTTP/2 writes header
  names in lower case; `content-type: application/json` is the same header as in Step 2.
  `content-length: 254` is the size of the body in bytes (roughly, characters), `date` is when it
  was sent, and `server` names the program that answered. The two `access-control-…` headers are
  for browsers; you'll meet them much later.
- The empty line ends the headers.
- Everything after it is the JSON body. httpbin describes your request back to you: `args` is the
  query string (empty here), `headers` lists the headers curl sent (curl added `Accept: */*`,
  "any format", and `User-Agent`, its own name), `origin` is your internet address as httpbin saw
  it (we've replaced ours with an example address), and `url` is what you asked for.

- ✅ If the first line is `HTTP/2 200` (or `HTTP/1.1 200 OK`) and you see a JSON body, you're
  right. Your `date`, `X-Amzn-Trace-Id` and `origin` will differ.
- ❌ If you see `Could not resolve host: httpbin.org`, you're offline (or a firewall blocks it).
  Skip the httpbin steps; nothing later depends on them. In the next lesson you'll run your own
  server on your own computer, and curl will work with no internet at all.

### Step 4: send data with a POST

`GET` only asks. To *send* data (a new product, say), you use `POST` with a body. httpbin's `/post`
address echoes back whatever you send:

```bash
curl -i -X POST https://httpbin.org/post -H 'Content-Type: application/json' -d '{"name":"Mug"}'
```

### Line by line

`-X POST`
- **What:** sets the method to `POST`.
- **Why:** `POST` means "here is something new; please create it". Our server will treat it very
  differently from `GET`.
- **How:** `-X` is curl's short form of `--request`; whatever follows it replaces the method on
  the request line.
- **Remove it and…** curl still sends `POST`, because `-d` below switches it on. Writing `-X POST`
  anyway makes the command say exactly what it does.

`-H 'Content-Type: application/json'`
- **What:** adds a header to the request.
- **Why:** it tells the server "the body I'm sending is JSON". Without it, many servers refuse the
  body or misread it (see **Common mistakes**).
- **How:** `-H` (*header*) takes one `Name: value` line. The single quotes keep the space and
  colon together as one piece for your terminal. Repeat `-H` for more headers.
- **Remove it and…** curl labels the body `application/x-www-form-urlencoded` (the format of an
  old-style web form), and the server doesn't treat it as JSON.

`-d '{"name":"Mug"}'`
- **What:** the body to send: a JSON object with one field, `name`.
- **Why:** it's the data of the new product.
- **How:** `-d` (*data*) puts the text after it into the body, and adds a `Content-Length` header
  for you. The single quotes stop your terminal from touching the `{`, `}` and double quotes
  inside.
- **Remove it and…** you send a `POST` with an empty body: "create this" with nothing to create.

Written out, the request curl sends looks like this. It has all five parts from Step 1, including
a body at last:

```text
POST /post HTTP/1.1
Host: httpbin.org
User-Agent: curl/8.7.1
Accept: */*
Content-Type: application/json
Content-Length: 14

{"name":"Mug"}
```

### Run it

```bash
curl -i -X POST https://httpbin.org/post -H 'Content-Type: application/json' -d '{"name":"Mug"}'
```

```text
HTTP/2 200 
date: Sat, 26 Sep 2026 07:32:07 GMT
content-type: application/json
content-length: 425
server: gunicorn/19.9.0
access-control-allow-origin: *
access-control-allow-credentials: true

{
  "args": {}, 
  "data": "{\"name\":\"Mug\"}", 
  "files": {}, 
  "form": {}, 
  "headers": {
    "Accept": "*/*", 
    "Content-Length": "14", 
    "Content-Type": "application/json", 
    "Host": "httpbin.org", 
    "User-Agent": "curl/8.7.1", 
    "X-Amzn-Trace-Id": "Root=1-6ab774f7-76dd90d1480144cf1c7468de"
  }, 
  "json": {
    "name": "Mug"
  }, 
  "origin": "203.0.113.7", 
  "url": "https://httpbin.org/post"
}
```

`data` is your body exactly as it arrived (the `\"` are how JSON writes a `"` inside text), and
`json` is httpbin reading that body as JSON: an object with `name` set to `"Mug"`. It could only do
that because your `Content-Type` header said the body was JSON.

- ✅ If `json` shows `"name": "Mug"`, you're right: your body arrived and was understood.
- ❌ If `json` is `null` and your text appears under `form` instead, the `-H 'Content-Type: …'`
  part is missing or mistyped.
- ❌ On Windows PowerShell, type `curl.exe` instead of `curl` (in older PowerShell, `curl` is a
  different command). If the JSON still arrives mangled, run the book's curl commands in Git Bash
  or WSL instead, where the quoting works exactly as shown.

### Step 5: JSON, the language of the body

[**JSON**](../glossary.md#json) (JavaScript Object Notation) is the text format almost every
backend uses for bodies. Despite the name, it has nothing to do with JavaScript any more: Rust,
Python, Go and every other language read and write it. Here is a fuller product:

```json
{
  "id": 42,
  "name": "Mug",
  "price": 1299,
  "in_stock": true,
  "description": null,
  "tags": ["kitchen", "gift"]
}
```

### Line by line

`{` … `}`
- **What:** an **object**: a group of named values, like a Rust struct.
- **Why:** it keeps all the facts about one product together.
- **How:** inside the braces, each entry is `"name": value`, and entries are separated by commas.
  The names (called *keys*) are always text in double quotes. There's no comma after the last
  entry.
- **Remove it and…** (the braces) the values are loose instead of one thing, and a JSON reader
  rejects them.

`"id": 42` and `"price": 1299`
- **What:** two **numbers**.
- **Why:** an id and a price in cents (Step 2) are counts, so they're numbers, not text.
- **How:** numbers are written without quotes. JSON has one kind of number; `42`, `-3` and `2.5`
  are all numbers.
- **Remove it and…** (put quotes around it instead) `"42"` becomes text, not a number, and a Rust
  backend expecting a number refuses it.

`"name": "Mug"`
- **What:** a **string**, meaning a piece of text.
- **Why:** a name is text.
- **How:** strings always use double quotes. Single quotes aren't allowed in JSON.
- **Remove it and…** (the quotes) `Mug` on its own is not valid JSON; the reader stops with an
  error.

`"in_stock": true`
- **What:** a **boolean**: `true` or `false`, nothing else.
- **Why:** "is it in stock?" is a yes-or-no question.
- **How:** written in lower case, without quotes.
- **Remove it and…** you'd have to invent your own code, like `"yes"`, that every client must
  guess.

`"description": null`
- **What:** **null** means "no value here".
- **Why:** this product has no description yet. `null` says so honestly, instead of an empty
  string that looks like a real (blank) description.
- **How:** written `null`, lower case, no quotes. In Rust this will become `None`, the empty side
  of an `Option`.
- **Remove it and…** (the whole line) that's also allowed: a missing key and `null` both usually
  mean "no value". Each backend decides which to use.

`"tags": ["kitchen", "gift"]`
- **What:** an **array**: an ordered list of values, in square brackets, separated by commas.
- **Why:** a product can have any number of tags.
- **How:** an array can hold any JSON values, even objects: `[{"id": 1}, {"id": 2}]` is a list of
  two objects. This one is an array *nested* inside the object.
- **Remove it and…** you could only store one tag, or you'd squash them into one string like
  `"kitchen,gift"` that every client has to split.

That's the whole of JSON: **objects, arrays, strings, numbers, booleans and null**. Six things, and
you can nest them as deep as you like.

### Step 6: REST, a naming plan for endpoints

A backend answers many different requests. Each method-plus-path it understands is called an
[**endpoint**](../glossary.md#endpoint), and the full set of endpoints is the backend's
[**API**](../glossary.md#api): the list of things other programs can ask it. You could name
endpoints anything (`/getAllProducts`, `/deleteProductNow`), but most backends follow a style
called [**REST**](../glossary.md#rest): **the path names the thing, the method says what to do
with it**. These are the product endpoints of ShopRS, the online-shop backend you'll build in
Part B:

```text
GET     /products          list all products
GET     /products/{id}     get one product
POST    /products          create a new product
PUT     /products/{id}     replace one product
DELETE  /products/{id}     delete one product
```

### Line by line

`GET /products`
- **What:** returns every product, as a JSON array of objects.
- **Why:** the shop's home page needs the list.
- **How:** `/products` means "the whole collection"; `GET` means "read, don't change".
- **Remove it and…** a client can only see a product if it already knows its id.

`GET /products/{id}`
- **What:** returns one product, for example `GET /products/42`.
- **Why:** the product page needs one product's details.
- **How:** `{id}` is a *placeholder*: in a real request it's replaced by a number. Adding an id to
  the collection's path narrows it down to one item.
- **Remove it and…** clients have to download every product to show one.

`POST /products`
- **What:** creates a new product from the JSON body, and answers `201 Created`.
- **Why:** the shop owner adds new stock.
- **How:** you `POST` to the *collection*, because the new product has no id yet: the server
  picks one.
- **Remove it and…** there's no way to add products.

`PUT /products/{id}`
- **What:** replaces product `{id}` with the JSON body.
- **Why:** the owner corrects a name or changes a price.
- **How:** the path says *which* product; the body is its complete new version.
- **Remove it and…** products can never change.

`DELETE /products/{id}`
- **What:** deletes product `{id}`, and answers `204 No Content`.
- **Why:** the owner stops selling something.
- **How:** no body is needed; the path already says which product.
- **Remove it and…** products can never be removed.

Notice there are only **two paths** and **four methods**. Once you know the plan, you can guess the
endpoints for users, orders or carts without reading any documentation. That predictability is the
whole point of REST.

### Step 7: status codes, the waiter's note

Every response starts with a status code. The first digit tells you the family: **2xx** worked,
**4xx** the *client* made a mistake (fix the request), **5xx** the *server* broke (not your fault).
These ten cover almost everything this book's backends send:

| Code | Name | When to send it |
|---|---|---|
| `200` | OK | The request worked, and here's the answer (a `GET`, a `PUT`). |
| `201` | Created | A `POST` created something new. |
| `204` | No Content | It worked, and there's nothing to send back (a `DELETE`). |
| `400` | Bad Request | The request is broken: the body isn't valid JSON, say. |
| `401` | Unauthorized | We don't know who you are: you're not logged in. |
| `403` | Forbidden | We know who you are, and you're not allowed to do this. |
| `404` | Not Found | There's nothing at this path (no product 999). |
| `409` | Conflict | It clashes with what's already there: that email is already registered. |
| `422` | Unprocessable Content | Valid JSON, but the values break the rules: a negative price. |
| `500` | Internal Server Error | Something broke inside the server. The client can't fix it. |

You don't need to memorise these now. Come back to this table whenever a later lesson picks one.

## You might be wondering…

**"GET vs POST: what's the real difference?"**
`GET` *reads*: it must never change anything, so it's safe to repeat, to bookmark, and for
browsers to fetch in advance. It normally has no body; any options go in the query string. `POST`
*creates*: it carries the new data in its body, and sending it twice creates two things. That's why
a browser warns you before re-sending a form.

**"Why JSON and not HTML?"**
HTML is a finished *page*: text already arranged with fonts and layout, for a person to look at.
JSON is plain *data*, for a program to use. A backend's clients are programs (a phone app, a
website's code), and each wants to lay out the data its own way: the app shows the mug in a small
card, the website in a big grid. Send data, and let each client draw it.

**"What's the difference between 401 and 403?"**
`401` means "who are you?": you didn't log in, or your login has expired. Logging in can fix it.
`403` means "I know who you are, and the answer is no": you're logged in as a customer and asked
for an admin page. Logging in again won't help. (The name `401 Unauthorized` is confusing; think of
it as "unauthenticated".)

**"Is `?currency=usd` part of the path?"**
No. The path is `/products/42` and ends at the `?`. The query string is extra, optional options.
The backend decides *which* endpoint answers by the method and path alone, then reads the query
string inside it. So `/products/42` and `/products/42?currency=usd` reach the same code.

**"Where's the port? I've seen addresses like `localhost:3000`."**
Good catch. A [**port**](../glossary.md#port) is the numbered door the server listens behind. It's
hidden in these URLs because `https` uses port 443 unless you say otherwise (and `http` uses 80).
Your own server in the next lesson will listen on port 3000, so its address says so out loud:
`http://127.0.0.1:3000/`, where `127.0.0.1` is the numeric address of
[localhost](../glossary.md#localhost), meaning "this computer".

## Coming from another language?

The requests in this lesson are exactly what your language's HTTP tools send. In JavaScript:

```js
const res = await fetch("https://httpbin.org/post", {
  method: "POST",
  headers: { "Content-Type": "application/json" },
  body: JSON.stringify({ name: "Mug" }),
});
console.log(res.status, await res.json());
```

In Python, with the `requests` library:

```python
import requests

res = requests.post("https://httpbin.org/post", json={"name": "Mug"})
print(res.status_code, res.json())
```

Each part lines up with the curl command from Step 4: `method` is `-X`, `headers` is `-H`, `body`
is `-d`. Python's `json=` sets the `Content-Type` header for you. Java's `HttpClient`, Go's
`net/http` and Rust's own `reqwest` [crate](../glossary.md#crate) (a Rust library; see *Go deeper*) all produce the same request-line,
headers, empty-line, body text on the wire. Once you can read that text, you can debug any of
them.

## Common mistakes

**Sending JSON without `Content-Type: application/json`.**
Here's the `POST` from Step 4 with the `-H` part left out:

```bash
curl -i -X POST https://httpbin.org/post -d '{"name":"Mug"}'
```

```text
…
  "data": "", 
  "files": {}, 
  "form": {
    "{\"name\":\"Mug\"}": ""
  }, 
  "headers": {
    "Accept": "*/*", 
    "Content-Length": "14", 
    "Content-Type": "application/x-www-form-urlencoded", 
…
  "json": null, 
…
```

Without a `Content-Type`, curl labels the body as a web form, so httpbin reads your JSON as one
strange form field and `json` is `null`. A Rust backend expecting JSON goes further and rejects it
with `415 Unsupported Media Type`. **Fix:** always send `-H 'Content-Type: application/json'` with a
JSON body.

**Using GET to change data.**
`GET /products/42/delete` looks convenient, but browsers, search engines and link previews fetch
`GET` addresses on their own, to check links or load pages early. A preview could delete your
product. **Fix:** anything that changes data uses `POST`, `PUT` or `DELETE`.

**Returning 200 for errors.**
A response like `200 OK` with the body `{"error":"not found"}` tells every program "success". The
app shows an empty product, monitoring tools see no problems, and retries never happen. **Fix:**
the status code must tell the truth: `404` when it's missing, `422` when the input breaks a rule,
`500` when the server broke. The body can then add the details.

**Putting secrets in the query string.**
`GET /login?password=hunter2` puts the password in the URL, and URLs are saved everywhere: browser
history, server logs, bookmarks, screenshots. **Fix:** send secrets in the body of a `POST`, or in a
header, and always over `https`. Part B shows the right way to log in.

## More examples

### Create a product: `201` and a `Location`

A `POST` that creates something answers `201`, and a `Location` header saying where the new thing
now lives.

```text
POST /products HTTP/1.1
Host: shop.example.com
Content-Type: application/json

{"name":"Teapot","price":2450}
```

```text
HTTP/1.1 201 Created
Location: /products/43
Content-Type: application/json

{"id":43,"name":"Teapot","price":2450}
```

The server picked the id `43`. The client can `GET /products/43` any time to fetch it again.

### Not found: `404`

Asking for a product that doesn't exist is not a server failure, so it's a `4xx`.

```text
GET /products/999 HTTP/1.1
Host: shop.example.com
```

```text
HTTP/1.1 404 Not Found
Content-Type: application/json

{"error":"product 999 not found"}
```

The status code carries the meaning; the small JSON body helps a human reading the logs.

### Bad input: `422` with an error body

The JSON is valid, but the values break the shop's rules. A good error body says exactly what to
fix.

```text
POST /products HTTP/1.1
Host: shop.example.com
Content-Type: application/json

{"name":"","price":-5}
```

```text
HTTP/1.1 422 Unprocessable Content
Content-Type: application/json

{"error":"invalid product","fields":{"name":"must not be empty","price":"must be 0 or more"}}
```

If the body weren't valid JSON at all (a missing `}`, say), the answer would be `400 Bad Request`
instead.

### Delete: `204` and no body

A successful `DELETE` has nothing to send back, and `204` says exactly that.

```text
DELETE /products/43 HTTP/1.1
Host: shop.example.com
```

```text
HTTP/1.1 204 No Content

```

No `Content-Type`, no body: the status line alone says "done".

## Your turn

### 🟢 Guided

httpbin can answer with any status code you ask for: `/status/` followed by the code. Run this and
write down the status code and its name:

```bash
curl -i https://httpbin.org/status/404
```

<details><summary>Solution</summary>

```text
HTTP/2 404 
date: Sat, 26 Sep 2026 07:32:08 GMT
content-type: text/html; charset=utf-8
content-length: 0
server: gunicorn/19.9.0
access-control-allow-origin: *
access-control-allow-credentials: true

```

The status is **`404`, Not Found**: "there's nothing at this path". HTTP/2 leaves out the `Not Found`
phrase, so the name comes from the table in Step 7. `content-length: 0` means the body is empty.
Try `/status/201` and `/status/500` too, and check each against the table.

</details>

### 🟡 Tweak

Change the `POST` from Step 4 so it sends a JSON object of your own, with at least one string, one
number and one array. Find your data in httpbin's answer.

<details><summary>Solution</summary>

For example, a person called Ada who is 36 and likes tea and Rust:

```bash
curl -i -X POST https://httpbin.org/post -H 'Content-Type: application/json' -d '{"name":"Ada","likes":["tea","rust"],"age":36}'
```

```text
HTTP/2 200 
date: Sat, 26 Sep 2026 07:32:25 GMT
content-type: application/json
content-length: 531
server: gunicorn/19.9.0
access-control-allow-origin: *
access-control-allow-credentials: true

{
  "args": {}, 
  "data": "{\"name\":\"Ada\",\"likes\":[\"tea\",\"rust\"],\"age\":36}", 
  "files": {}, 
  "form": {}, 
  "headers": {
    "Accept": "*/*", 
    "Content-Length": "46", 
    "Content-Type": "application/json", 
    "Host": "httpbin.org", 
    "User-Agent": "curl/8.7.1", 
    "X-Amzn-Trace-Id": "Root=1-6ab77509-59f8dfb206e819644f28e455"
  }, 
  "json": {
    "age": 36, 
    "likes": [
      "tea", 
      "rust"
    ], 
    "name": "Ada"
  }, 
  "origin": "203.0.113.7", 
  "url": "https://httpbin.org/post"
}
```

Your data is in two places: `data` holds the exact text you sent, and `json` holds httpbin's
reading of it. Notice that `json` lists the keys in a different order (`age`, `likes`, `name`).
That's fine: the order of keys in a JSON object carries no meaning. If `json` is `null`, check that
your body is valid JSON (double quotes, no comma after the last entry) and that the `-H` part is
there.

</details>

### 🔴 From scratch

No command this time. Write, as plain text like Steps 1 and 2, the request a shop app sends to
**add two of product 42 to my shopping cart**, and the response the server sends back when it
works. Decide the method, path, headers, body and status code yourself.

<details><summary>Solution</summary>

```text
POST /cart/items HTTP/1.1
Host: shop.example.com
Content-Type: application/json

{"product_id":42,"quantity":2}
```

```text
HTTP/1.1 201 Created
Location: /cart/items/7
Content-Type: application/json

{"id":7,"product_id":42,"quantity":2}
```

Why these choices:

- **`POST`**, because you're creating something new: a line in the cart.
- **`/cart/items`**, because REST names the *collection* you're adding to. The cart is yours
  (the server knows who you are from your login), so its path needs no user id.
- **`Content-Type: application/json`**, because the body is JSON.
- **The body** names the product and how many. It doesn't include a price: the server looks that
  up itself, so nobody can send a price of `1`.
- **`201 Created`**, with the new item's id and a `Location` header saying where it lives.

Other sensible answers exist: `PUT /cart/items/42` with `{"quantity":2}` is also RESTful, if you
treat "product 42 in my cart" as a thing you set. What matters is that the method matches the
action, the path names a thing, and the status tells the truth.

</details>

## Quick check

<div class="quiz" data-topic="how-a-web-backend-works"></div>

## Remember this

- A client sends a **request** (method, path + query string, version, headers, body); a server
  sends back a **response** (status code, headers, body). Both are plain text, split by an empty
  line.
- **JSON** has six kinds of value: objects, arrays, strings, numbers, booleans and null. Keep
  money in whole cents.
- **REST:** the path names the thing (`/products/42`), the method says what to do (`GET`, `POST`,
  `PUT`, `DELETE`).
- **Status codes:** 2xx worked, 4xx the client's mistake, 5xx the server's fault. Send the one that
  tells the truth.
- `curl -i` shows you the full response; add `-X`, `-H` and `-d` to send data.

## Go deeper

- [Rust for Humans: Serde and JSON](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/serde-and-json.html)
- [Rust for Humans: HTTP clients with reqwest](https://open-source-bd.github.io/rustbook-for-human/runtime-and-ecosystem/http-clients-reqwest.html)
- [MDN: An overview of HTTP](https://developer.mozilla.org/en-US/docs/Web/HTTP/Guides/Overview) — A friendly deep dive.
- [MDN: HTTP status codes](https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Status) — Every code explained.
- [httpbin.org](https://httpbin.org/) — Every practice endpoint httpbin offers.
- [JSON.org](https://www.json.org/json-en.html) — The whole JSON format on one page.

**Next:**

- [Tour of the stack](../part-0-start/tour-of-the-stack.md)
