# serve: -p nesting-and-modular-routers --example nest-two-levels
curl -i http://127.0.0.1:3000/api/v1/books/7
curl -i http://127.0.0.1:3000/api/v2/books/7
curl -i http://127.0.0.1:3000/v1/books/7
