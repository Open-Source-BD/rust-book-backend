# serve: -p routes-and-methods --example routes-fallback-uri
curl -i http://127.0.0.1:3000/magazines
curl -i "http://127.0.0.1:3000/magazines/3?page=2"
