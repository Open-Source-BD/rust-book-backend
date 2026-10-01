# serve: -p middleware-and-tower-layers --example mw-timeout-test
curl -i http://127.0.0.1:3000/fast
curl -i http://127.0.0.1:3000/slow
