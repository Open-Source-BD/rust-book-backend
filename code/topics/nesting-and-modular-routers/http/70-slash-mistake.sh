# serve: -p nesting-and-modular-routers --example nest-slash-mistake
curl -i http://127.0.0.1:3000/api/books
curl -i http://127.0.0.1:3000/api/books/
curl -i http://127.0.0.1:3000/api/books/7
