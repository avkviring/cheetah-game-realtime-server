helm dependency update Platform
helm --namespace=ci-test upgrade --install ci-test Platform
