FROM rust:1.96.1 AS rust-build

RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates tar && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /assets
RUN --mount=type=secret,id=github_token \
		set -eu; \
		token="$(cat /run/secrets/github_token)"; \
		url="https://github.com/smartango/gatearwayman-gui/releases/download/v1.0.1/gatearwayman-gui-v1.0.1.tar.gz"; \
		curl --fail --show-error --location \
			-H "Authorization: Bearer ${token}" \
			-H "Accept: application/octet-stream" \
			"${url}" \
			-o /tmp/gatearwayman-gui.tar.gz; \
		tar -xzf /tmp/gatearwayman-gui.tar.gz -C /assets; \
		rm -f /tmp/gatearwayman-gui.tar.gz

WORKDIR /app

ADD . /app

RUN ASSETS=/assets cargo build --release


FROM scratch

COPY --from=rust-build /app/target/release/apimanager-service /apimanager-service
