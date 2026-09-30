# serve: -p json-and-serde --example json-orders
curl -i -X POST http://127.0.0.1:3000/orders -H 'content-type: application/json' -d '{"items":[{"book_id":1,"qty":2}]}'
curl -i -X POST http://127.0.0.1:3000/orders -H 'content-type: application/json' -d '{"items":[{"book_id":1,"qty":2},{"book_id":7,"qty":3}]}'
