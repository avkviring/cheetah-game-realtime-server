# Тестирование

Так как основной принцип разработки - минимальное количество разработчиков, то основное направление - автоматические
тесты.

Их должно быть ровно столько, чтобы любой код, которые их прошел, можно было бы отправить на prod без ручной проверки и
спокойно уйти спать :-)

## Уровни тестирования

### Unit тесты в компонентах

- пользуемся тем что дает платформа
- должны быть по-возможности быстрыми
- не должно быть случайных падений

### Интеграционные тесты в компонентах

- используем реальные базы данных (через docker)
- используем mock внешних сервисов

### E2E тесты с полным развертываем сервера и клиента.

- собираем docker images для всех компонентов
- запускаем сервер через helm на kubernetes в digital ocean
- запускаем Unity с проектом clients/Unity и в ней запускаем интеграционные тесты
- все это выполняется автоматически .github/workflows/test.integration.yml