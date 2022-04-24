# выкладка платформы для  тестирования production syncario
tar -czf rooms-configuration.tgz -C ../../server/matches/Factory/example/ .
helm -n production upgrade \
  --install production Platform \
  -f Platform/values-production.yaml \
  --set global.grpcDomain=api.syncario.production.cheetah.games \
  --set global.platformImageVersion=999.999.999 \
  --set global.roomsConfiguration=$(cat rooms-configuration.tgz | openssl enc -A -base64)
