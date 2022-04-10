## Обновление/инсталляция сервисов для мониторинга
## параметр - имя домена, пароль администратора для grafana, пароль для администрирования prometheus
helm -n monitoring upgrade --install monitoring Monitoring \
  --kubeconfig /Users/kviring/Documents/.kube/stage.yaml \
  --set global.prometheusAdminAuth=$(echo $3 | htpasswd -n -i admin | openssl enc -A -base64) \
  --set kube-prometheus-stack.prometheus.ingress.hosts[0]=prometheus.$1 \
  --set kube-prometheus-stack.prometheus.ingress.tls[0].hosts=\{prometheus.$1\} \
  --set kube-prometheus-stack.prometheus.ingress.tls[0].secretName=prometheus-ingress \
  --set kube-prometheus-stack.grafana.ingress.hosts[0]=grafana.$1 \
  --set kube-prometheus-stack.grafana.ingress.tls[0].hosts=\{grafana.$1\} \
  --set kube-prometheus-stack.grafana.ingress.tls[0].secretName=grafana-ingress \
  --set kube-prometheus-stack.alertmanager.ingress.hosts[0]=alertmanager.$1 \
  --set kube-prometheus-stack.alertmanager.ingress.tls[0].hosts=\{alertmanager.$1\} \
  --set kube-prometheus-stack.alertmanager.ingress.tls[0].secretName=alertmanager-ingress \
  --set kube-prometheus-stack.grafana.adminPassword=$2

