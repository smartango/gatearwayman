FROM rust:1.96.1 AS rust-build

RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates tar && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /assets
RUN --mount=type=secret,id=github_token \
		curl -fsSL \
			-H "Authorization: Bearer $(cat /run/secrets/github_token)" \
			-H "Accept: application/octet-stream" \
			"https://github.com/smartango/gatearwayman-gui/releases/download/v1.0.1/gatearwayman-gui-v1.0.1.tar.gz" \
		| tar -xz -C /assets

WORKDIR /app

ADD . /app

RUN ASSETS=/assets cargo build --release


FROM scratch

COPY --from=rust-build /app/target/release/apimanager-service /apimanager-service
