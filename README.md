# REST API with AXUM in RUST

This is a REST API with AXUM built in RUST following the following tutorial:
[Create a High-Performance REST API with Rust](https://www.rustfinity.com/blog/create-high-performance-rest-api-with-rust)

I just an OIDC JWT Token validation layer.

## Getting Started

### Prerequisites

* Rust
* Docker
* Docker Compose

### Running

```shell
cargo install sqlx-cli --no-default-features --features native-tls,postgres
docker compose up -d
```

Then create an env file with the following content:

```shell
cat << EOF > .env
DATABASE_URL=postgres://postgres:password@localhost:5432/rust-axum-rest-api
ISSUER_URL=http://localhost:8080/realms/rest-axum-api
EOF
```

You can now run the application:
```shell
# Run the migrations first
sqlx migrate run
# Then run the application
cargo run
```

Finally, you can test the application with the following commands:
```shell
export ACCESS_TOKEN=$(\
  http --form -A basic -a cli:WOhxh2rBPgVrbeH8cXjjNSX4kp1MLFkd \
  :8080/realms/rest-axum-api/protocol/openid-connect/token \
  grant_type=password \
  username=bob \
  password=password|jq -r .access_token)

http -A bearer -a $ACCESS_TOKEN :8000/posts
http -A bearer -a $ACCESS_TOKEN :8000/posts title='Aweomse post' body='This is a post'
http -A bearer -a $ACCESS_TOKEN :8000/authors name='Bob' 
http -A bearer -a $ACCESS_TOKEN :8000/posts title='Oh my god' body='This is an awsome post' author_id:=1
http -A bearer -a $ACCESS_TOKEN :8000/posts
http -A bearer -a $ACCESS_TOKEN PUT :8000/posts/1 title='Aweomse post' body='I forgot the author' author_id:=1
http -A bearer -a $ACCESS_TOKEN :8000/posts
http -A bearer -a $ACCESS_TOKEN DELETE :8000/posts/2
http -A bearer -a $ACCESS_TOKEN :8000/posts
```

### Stopping

```shell
docker compose down --volumes
```

### Resources

* [Create a High-Performance REST API with Rust](https://www.rustfinity.com/blog/create-high-performance-rest-api-with-rust)
* [Axum JWT OIDC](https://github.com/soya-miyoshi/axum-jwt-oidc)
* [Keycloak export real with sample users](https://github.com/little-pinecone/keycloak-in-docker/tree/master)