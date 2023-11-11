FROM node:18.4 AS builder

WORKDIR /usr/app

RUN npm i rollup @rollup/plugin-node-resolve @rollup/plugin-commonjs @rollup/plugin-json

RUN npm i codemirror @codemirror/state @codemirror/lang-json @codemirror/autocomplete codemirror-json-schema

COPY request.schema.json editor.mjs ./

RUN node_modules/.bin/rollup editor.mjs -f iife -o editor.bundle.js --name editorBundle \
    -p @rollup/plugin-node-resolve \
    -p @rollup/plugin-commonjs \
    -p @rollup/plugin-json

FROM nginx:latest

COPY ./index.html /usr/share/nginx/html/index.html
COPY --from=builder /usr/app/editor.bundle.js /usr/share/nginx/html/