# serve: -p a2-build-todo-api --example todo-newest-first
curl -i -X POST http://127.0.0.1:3000/api/todos -H 'content-type: application/json' -d '{"title":"buy milk"}'
curl -i -X POST http://127.0.0.1:3000/api/todos -H 'content-type: application/json' -d '{"title":"walk the dog"}'
curl -i "http://127.0.0.1:3000/api/todos?order=newest"
curl -i "http://127.0.0.1:3000/api/todos?order=random"
