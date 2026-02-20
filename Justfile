start_fresh_postgres:
    podman run --rm --replace -d \
        --name postgres \
        -e POSTGRES_USER=loco \
        -e POSTGRES_PASSWORD=loco \
        -e POSTGRES_DB=gooncityhub \
        -p 5432:5432 \
        postgres:18

connect_to_db:
    usql postgres://loco:loco@localhost:5432/gooncityhub
