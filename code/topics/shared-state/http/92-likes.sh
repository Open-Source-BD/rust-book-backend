# serve: -p shared-state --example state-likes
curl -i -X POST http://127.0.0.1:3000/likes/1
curl -i -X POST http://127.0.0.1:3000/likes/1
curl -i -X POST http://127.0.0.1:3000/likes/2
curl -i http://127.0.0.1:3000/likes/1
curl -i http://127.0.0.1:3000/likes/3
