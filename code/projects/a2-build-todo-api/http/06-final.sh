# serve: -p a2-build-todo-api
curl -i http://127.0.0.1:3000/health
curl -i -X POST http://127.0.0.1:3000/api/todos -H 'content-type: application/json' -d '{"title":"buy milk"}'
curl -i -X POST http://127.0.0.1:3000/api/todos -H 'content-type: application/json' -d '{"title":"walk the dog"}'
curl -i http://127.0.0.1:3000/api/todos
curl -i -X PATCH http://127.0.0.1:3000/api/todos/1 -H 'content-type: application/json' -d '{"done":true}'
curl -i -X DELETE http://127.0.0.1:3000/api/todos/2
curl -i http://127.0.0.1:3000/api/todos/2
curl -i -X POST http://127.0.0.1:3000/api/todos -H 'content-type: application/json' -d '{"title":""}'
