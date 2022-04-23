# выкладка платформы для  тестирования production syncario
tar -czf rooms-configuration.tgz -C ../../server/matches/Factory/example/ .
helm --kubeconfig /Users/kviring/Documents/.kube/stage.yaml -n syncario-production upgrade \
--install syncario-production Platform \
-f Platform/values-syncario-production.yaml \
--set global.grpcDomain=syncario.production.cheetah.games \
--set global.platformImageVersion=999.999.999 \
--set global.roomsConfiguration=`cat rooms-configuration.tgz | openssl enc -A -base64` \
