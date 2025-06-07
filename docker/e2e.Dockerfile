FROM rust:latest

# Install Python
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        python3 \
        python3-pip && \
    rm -rf /var/lib/apt/lists/*

# Install Python requirements for e2e tests
COPY e2e/requirements.txt /tmp/requirements.txt
RUN pip3 install --no-cache-dir -r /tmp/requirements.txt && rm /tmp/requirements.txt

WORKDIR /evi

ENTRYPOINT ["/bin/bash", "-c", "cargo build --verbose && pytest e2e --verbose"]
