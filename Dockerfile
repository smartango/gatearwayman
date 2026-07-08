FROM node:20-alpine AS react-build

WORKDIR /app

RUN apk add --no-cache git

ADD gui/env-cmdrc /app/.env-cmdrc

ENV NODE_ENV=production
ENV PUBLIC_URL=http://localhost:8080
ENV API_BASE=http://localhost:8081
ENV API_AUTH=http://localhost:8082
ENV BASE_REALURL=http://localhost:8080
# replace [[PUBLIC_URL]] [[API_BASE]] [[API_AUTH]] [[BASE_REALURL]]

RUN sed -i 's/\[\[PUBLIC_URL\]\]/${PUBLIC_URL}/g' .env-cmdrc
RUN sed -i 's/\[\[API_BASE\]\]/${API_BASE}/g' .env-cmdrc
RUN sed -i 's/\[\[API_AUTH\]\]/${API_AUTH}/g' .env-cmdrc
RUN sed -i 's/\[\[BASE_REALURL\]\]/${BASE_REALURL}/g' .env-cmdrc

RUN git clone git@github.com:smartango/gatearwayman-gui.git
WORKDIR /app/gatearwayman-gui
RUN cp ../.env-cmdrc
RUN npm install --legacy-peer-deps
RUN npm run build

FROM rust:1.96.1 AS rust-build

COPY --from=react-build /app/gatearwayman-gui/build /assets

WORKDIR /app

ADD . /app

RUN ASSETS=/assets cargo build --release


FROM scratch

COPY --from=rust-build /app/target/release/apimanager-service /apimanager-service
