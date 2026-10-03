# serve: -p a2-build-todo-api --example step5
curl -i -X POST http://127.0.0.1:3000/todos -H 'content-type: application/json' -d '{"title":""}'
curl -i -X POST http://127.0.0.1:3000/todos -H 'content-type: application/json' -d '{"title":"buy milk"}'
curl -i -X PATCH http://127.0.0.1:3000/todos/1 -H 'content-type: application/json' -d '{"title":""}'
curl -i -X PATCH http://127.0.0.1:3000/todos/1 -H 'content-type: application/json' -d '{"done":true}'
