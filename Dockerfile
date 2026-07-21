FROM rust:1.96.1 AS rust-build

## https://github.com/smartango/gatearwayman-gui/releases/download/v1.0.1/gatearwayman-gui-v1.0.1.tar.gz
RUN apt-get update && apt-get install -y --no-install-recommends curl ca-certificates tar jq && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /assets
RUN --mount=type=secret,id=github_token \
		set -eu; \
		token="$(cat /run/secrets/github_token)"; \
		release_tag="v1.0.1"; \
		asset_name="gatearwayman-gui-v1.0.1.tar.gz"; \
		release_api_url="https://api.github.com/repos/smartango/gatearwayman-gui/releases/tags/${release_tag}"; \
		asset_api_url="$(curl --fail --show-error --location \
			-H "Authorization: Bearer ${token}" \
			-H "Accept: application/vnd.github+json" \
			"${release_api_url}" \
			| jq -r --arg name "${asset_name}" '.assets[] | select(.name == $name) | .url' \
			| head -n 1)"; \
		if [ -z "${asset_api_url}" ] || [ "${asset_api_url}" = "null" ]; then \
			echo "Release asset ${asset_name} not found in tag ${release_tag}"; \
			exit 1; \
		fi; \
		curl --fail --show-error --location \
			-H "Authorization: Bearer ${token}" \
			-H "Accept: application/octet-stream" \
			"${asset_api_url}" \
			-o /tmp/gatearwayman-gui.tar.gz; \
		tar -xzf /tmp/gatearwayman-gui.tar.gz -C /assets; \
		rm -f /tmp/gatearwayman-gui.tar.gz

WORKDIR /app

ADD . /app

RUN ASSETS=/assets cargo build --release


FROM scratch

COPY --from=rust-build /app/target/release/apimanager-service /apimanager-service
