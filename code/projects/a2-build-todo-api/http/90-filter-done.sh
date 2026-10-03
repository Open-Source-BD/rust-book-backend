# serve: -p a2-build-todo-api --example todo-filter-done
curl -i -X POST http://127.0.0.1:3000/api/todos -H 'content-type: application/json' -d '{"title":"buy milk"}'
curl -i -X POST http://127.0.0.1:3000/api/todos -H 'content-type: application/json' -d '{"title":"walk the dog"}'
curl -i -X PATCH http://127.0.0.1:3000/api/todos/1 -H 'content-type: application/json' -d '{"done":true}'
curl -i "http://127.0.0.1:3000/api/todos?done=true"
curl -i "http://127.0.0.1:3000/api/todos?done=false"
curl -i "http://127.0.0.1:3000/api/todos?done=maybe"
