# serve: -p path-and-query-extractors
curl -i "http://127.0.0.1:3000/search?q=rust"
curl -i "http://127.0.0.1:3000/search?q=rust&page=2"
curl -i http://127.0.0.1:3000/search
curl -i "http://127.0.0.1:3000/search?q=hello%20world"
