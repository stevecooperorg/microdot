FROM node:latest

RUN npm install -g live-server
RUN mkdir -p /files
EXPOSE 8080

CMD ["live-server", "/files"]