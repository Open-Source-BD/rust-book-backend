# serve: -p routes-and-methods --example routes-any
curl -i http://127.0.0.1:3000/ping
curl -i -X POST http://127.0.0.1:3000/ping
curl -i -X DELETE http://127.0.0.1:3000/ping
curl -i -X PATCH http://127.0.0.1:3000/ping
