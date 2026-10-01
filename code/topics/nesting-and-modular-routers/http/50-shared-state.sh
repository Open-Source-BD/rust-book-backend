# serve: -p nesting-and-modular-routers --example nest-shared-state
curl -i http://127.0.0.1:3000/health
curl -i http://127.0.0.1:3000/api/books/7
curl -i http://127.0.0.1:3000/api/books/8
curl -i http://127.0.0.1:3000/health
