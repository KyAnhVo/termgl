FROM rust:1.87-slim-bookworm AS builder
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch
COPY src/ src/
COPY examples/ examples/
RUN cargo build --release --examples && \
    strip target/release/examples/planet \
          target/release/examples/solar-system \
          target/release/examples/show_mesh

FROM debian:trixie-slim AS runner
RUN apt-get update && apt-get install -y --no-install-recommends libgcc-s1 bash && \
    rm -rf /var/lib/apt/lists/*
WORKDIR /termgl
COPY --from=builder \
     /build/target/release/examples/planet \
     /build/target/release/examples/solar-system \
     /build/target/release/examples/show_mesh \
     ./target/release/examples/
COPY examples/assets/ examples/assets/
COPY earth.sh solar-system.sh view-mesh.sh ./
RUN chmod +x earth.sh solar-system.sh view-mesh.sh \
             target/release/examples/planet \
             target/release/examples/solar-system \
             target/release/examples/show_mesh
COPY RUN-EXAMPLES-INSTRUCTION.md REPORT.md readme.md ./
COPY run.sh ./
RUN chmod +x run.sh
ENV TERM=xterm-256color
ENTRYPOINT ["./run.sh"]
CMD []
