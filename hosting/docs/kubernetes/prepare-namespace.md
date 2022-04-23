# Настройка namespace

### Доступ к реестру

```shell
kubectl create secret docker-registry cheetahdockerregistry \
--namespace *** \ 
--docker-server=docker.registry.cheetah.games  \
--docker-username=***  \
--docker-password=***  \
--docker-email=***
```

### JWT ключи

```shell
openssl ecparam -name prime256v1 -genkey -out private.pem
openssl pkcs8  -topk8 -nocrypt -in private.pem -out private-pkcs8.pem
openssl ec -in private.pem -pubout -out public.pem
kubectl create secret generic jwt \
--namespace *** \
--from-file=public=public.pem \
--from-file=private=private-pkcs8.pem


### Добавить namespace к agones

```shell
helm upgrade my-release agones/agones --reuse-values --set "gameservers.namespaces={default,xbox,ps4}" --namespace agones-system
```