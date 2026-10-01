# serve: -p nesting-and-modular-routers --example nest-layer-one-router
curl -i http://127.0.0.1:3000/api/books/7
curl -i http://127.0.0.1:3000/health
curl -i http://127.0.0.1:3000/api/books/7/extra
curl -i http://127.0.0.1:3000/nope
