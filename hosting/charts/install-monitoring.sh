## Обновление/инсталляция сервисов для мониторинга
## параметр - production|stage, имя домена, пароль администратора для grafana, пароль для администрирования prometheus
helm -n monitoring upgrade --install monitoring Monitoring \
  -f Monitoring/values-shared.yaml \
  -f Monitoring/values-$1.yaml \
  --set global.prometheusAdminAuth=$(echo $4 | htpasswd -n -i admin | openssl enc -A -base64) \
  --set kube-prometheus-stack.prometheus.ingress.hosts[0]=prometheus.$2 \
  --set kube-prometheus-stack.prometheus.ingress.tls[0].hosts=\{prometheus.$2\} \
  --set kube-prometheus-stack.prometheus.ingress.tls[0].secretName=prometheus-ingress \
  --set kube-prometheus-stack.grafana.ingress.hosts[0]=grafana.$2 \
  --set kube-prometheus-stack.grafana.ingress.tls[0].hosts=\{grafana.$2\} \
  --set kube-prometheus-stack.grafana.ingress.tls[0].secretName=grafana-ingress \
  --set kube-prometheus-stack.alertmanager.ingress.hosts[0]=alertmanager.$2 \
  --set kube-prometheus-stack.alertmanager.ingress.tls[0].hosts=\{alertmanager.$2\} \
  --set kube-prometheus-stack.alertmanager.ingress.tls[0].secretName=alertmanager-ingress \
  --set kube-prometheus-stack.grafana.adminPassword=$3

