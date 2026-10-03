# serve: -p a2-build-todo-api --example step4
curl -i -X POST http://127.0.0.1:3000/todos -H 'content-type: application/json' -d '{"title":"buy milk"}'
curl -i -X POST http://127.0.0.1:3000/todos -H 'content-type: application/json' -d '{"title":"walk the dog"}'
curl -i -X PATCH http://127.0.0.1:3000/todos/1 -H 'content-type: application/json' -d '{"done":true}'
curl -i -X PATCH http://127.0.0.1:3000/todos/2 -H 'content-type: application/json' -d '{"title":"walk the dog twice"}'
curl -i -X DELETE http://127.0.0.1:3000/todos/1
curl -i -X DELETE http://127.0.0.1:3000/todos/1
curl -i -X POST http://127.0.0.1:3000/todos -H 'content-type: application/json' -d '{"title":"water the plants"}'
curl -i http://127.0.0.1:3000/todos
