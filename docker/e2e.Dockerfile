FROM rust:1.78.0-bullseye

# Install Python and create a virtual environment for test dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        python3 \
        python3-venv && \
    rm -rf /var/lib/apt/lists/*

# Prepare virtual environment with Python requirements for the e2e tests
COPY e2e/requirements.txt /tmp/requirements.txt
RUN python3 -m venv /opt/evi-venv && \
    /opt/evi-venv/bin/pip install --no-cache-dir -r /tmp/requirements.txt && \
    rm /tmp/requirements.txt

# Ensure the venv executables come first on PATH
ENV PATH="/opt/evi-venv/bin:$PATH"

WORKDIR /evi

ENTRYPOINT ["/bin/bash", "-c", "cargo build --verbose && pytest e2e --verbose"]
