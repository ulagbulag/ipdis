# Configure environment variables
ARG ALPINE_VERSION="latest"
ARG API_FEATURES=""
ARG PACKAGE="ipdis"

# Be ready for serving
FROM docker.io/alpine:${ALPINE_VERSION} as server

# Configure default environment variables
ENV ipiis_account_primary_address="127.0.0.1:9801"
ENV ipiis_server_port="9801"

# Server Configuration
EXPOSE 9801/tcp
EXPOSE 9801/udp
WORKDIR /usr/local/bin
CMD [ "runtime" ]

# Install dependencies
RUN apk add --no-cache iproute2-tc libgcc libpq

# Be ready for building
FROM docker.io/rust:1-alpine${ALPINE_VERSION} as builder

# Install dependencies
RUN apk add --no-cache libpq-dev musl-dev

# Load environment variables
ARG API_FEATURES
ARG PACKAGE

# Install rust dependencies
RUN mkdir -p /out /plugins \
    && cargo install --no-default-features --features "$API_FEATURES" --root /plugins diesel_cli \
    && mv /plugins/bin/* /out \
    && rm -rf /plugins

# Load source files
ADD . /src
WORKDIR /src

# Build it!
RUN mkdir -p /out \
    # Copy SQL migration files
    && mkdir -p /out/postgres \
    && cp -r ./api/postgres/diesel.toml ./api/postgres/migrations /out/postgres \
    # disable default API features
    && sed -i 's/^\(default = \)\[.*\]/\1\[\]/g' ./api/Cargo.toml \
    # build packages
    && cargo build --all --workspace --release --features "$API_FEATURES" \
    && find ./target/release/ -maxdepth 1 -type f -perm +a=x -print0 | xargs -0 -I {} mv {} /out \
    && mv ./LICENSE-* / \
    && mv /out/${PACKAGE}-runtime /out/runtime \
    && rm -rf /src

# Copy executable files
FROM server
COPY --from=builder /out/* /usr/local/bin/
COPY --from=builder /LICENSE-* /usr/share/licenses/${PACKAGE}/
