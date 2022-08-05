## Выкладка платформы в kubernetes для локальных тестов
helm cheetah-config-creator ../example-config Platform/templates/
helm -n $1 upgrade --install $1 Platform -f Platform/values-dev.yaml --set global.grpcDomain=$1.stage.cheetah.games --set global.platformImageVersion=999.999.999
