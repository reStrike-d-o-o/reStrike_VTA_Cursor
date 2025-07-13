# Custom Dockerfile to add mpv to the dev container
FROM mcr.microsoft.com/devcontainers/universal:2

# Install mpv (for Ubuntu/Debian base)
RUN apt-get update && \
    apt-get install -y mpv && \
    apt-get clean && rm -rf /var/lib/apt/lists/* 