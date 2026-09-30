# serve: -p path-and-query-extractors
curl -i http://127.0.0.1:3000/books/-1
curl -i "http://127.0.0.1:3000/search?q=rust&page=-2"
