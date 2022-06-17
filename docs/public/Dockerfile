# образ для запуска
FROM squidfunk/mkdocs-material:8.0.4 as  build
RUN mkdir /tmp/site
RUN mkdir /tmp/public
COPY . /tmp/site/
RUN cd /tmp/site && mkdocs build -d /tmp/public

FROM nginx:1.22.0
COPY --from=build /tmp/public /usr/share/nginx/html/