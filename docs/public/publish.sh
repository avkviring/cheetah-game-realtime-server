#!/bin/bash
rm -rf site
docker run --rm -it -p 8000:8000 -v "${PWD}":/docs squidfunk/mkdocs-material build -f public-mkdocs.yml
rsync -var site/ root@docs.cheetah.games:/var/www/docs.cheetah.games/html/
