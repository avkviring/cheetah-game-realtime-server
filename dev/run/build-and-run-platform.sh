#!/bin/bash
source env.platform

docker-compose --env-file env.platform up -d

# ждем запуска образов
sleep 3
# запускаем сервисы
cargo run --manifest-path=../../components/cerberus/server/Service/Cargo.toml &
cargo run --manifest-path=../../components/auth/server/Service/Cargo.toml &


# выходим по нажатию любой клавиши
read -r -p "" response
docker-compose --env-file env.platform stop
docker-compose --env-file env.platform rm -f
kill $(jobs -p)

