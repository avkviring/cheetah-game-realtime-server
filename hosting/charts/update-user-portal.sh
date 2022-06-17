helm -n user-portal upgrade --install user-portal UserPortal --set global.platformImageVersion=$1 --wait-for-jobs --wait
