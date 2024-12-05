FROM mcr.microsoft.com/devcontainers/rust:1-bullseye

# Install additional OS packages
RUN apt-get update && export DEBIAN_FRONTEND=noninteractive \
    && apt-get -y install --no-install-recommends \
        postgresql-client \
        libpq-dev \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

# Install global rust tools
RUN cargo install diesel_cli --no-default-features --features postgres \
    && cargo install cargo-edit \
    && cargo install cargo-watch \
    && cargo install cargo-audit

# Create non-root user
ARG USERNAME=vscode
ARG USER_UID=1000
ARG USER_GID=$USER_UID

RUN groupadd --gid $USER_GID $USERNAME \
    && useradd --uid $USER_UID --gid $USER_GID -m $USERNAME \
    && apt-get update \
    && apt-get install -y sudo \
    && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME \
    && chmod 0440 /etc/sudoers.d/$USERNAME

USER $USERNAME
