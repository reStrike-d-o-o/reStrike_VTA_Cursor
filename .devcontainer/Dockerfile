# Custom Dockerfile to add mpv to the dev container
FROM mcr.microsoft.com/devcontainers/universal:2

# Install mpv from the latest source (for Ubuntu/Debian base)
RUN apt-get update && \
    apt-get install -y software-properties-common && \
    add-apt-repository ppa:mpv-player/mpv-stable || echo "PPA not available, using system version" && \
    apt-get update && \
    apt-get install -y mpv && \
    apt-get clean && rm -rf /var/lib/apt/lists/* 