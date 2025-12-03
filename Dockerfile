# ---------------------------------------------------
# 1. ETAPA DE CONSTRUCCIÓN (BUILDER)
# ---------------------------------------------------
FROM rust:1-slim-bookworm as builder

WORKDIR /app

# Instalar dependencias del sistema necesarias para compilar (OpenSSL es clave para reqwest/neo4rs)
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Truco de caché: Copiar solo manifiestos primero para cachear dependencias
COPY Cargo.toml ./

# Crear un main dummy para compilar solo las dependencias
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Ahora copiamos el código real
COPY src ./src
# IMPORTANTE: Copiamos los templates para que estén disponibles si se chequean en build time
COPY templates ./templates 

# Actualizamos la fecha del archivo main.rs para forzar recompilación del código propio
RUN touch src/main.rs

# Compilamos el binario real
RUN cargo build --release

# ---------------------------------------------------
# 2. ETAPA DE EJECUCIÓN (RUNNER)
# ---------------------------------------------------
FROM debian:bookworm-slim

WORKDIR /app

# Instalar certificados CA (para HTTPS con OpenAI) y OpenSSL runtime
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

# Crear usuario no-root por seguridad
RUN useradd -m -u 1000 appuser
USER appuser

# Copiar el binario desde la etapa builder
COPY --from=builder /app/target/release/graph-rag-backend /app/server

# IMPORTANTE: Copiar la carpeta de plantillas a la imagen final
# El código Rust busca "templates/**/*.html", así que debe existir en /app/templates
COPY --from=builder /app/templates ./templates

# Configuración de entorno
ENV RUST_LOG=info
ENV PORT=3000
ENV HOST=0.0.0.0

# Exponer puerto (Render inyectará la variable PORT, pero esto es documental)
EXPOSE 3000

# Arrancar
ENTRYPOINT ["./server"]