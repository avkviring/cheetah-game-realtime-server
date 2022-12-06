# Битвы реального времени

"Умный" ретранслятор команд между клиентами. Фактически p2p без прикладной логике на сервере.

# Сборка клиентов для локального тестирования

Так как бинарные файлы не хранятся в репозитории, то перед началом интеграционного тестирования клиента из Unity
необходимо собрать rust клиент для требуемой платформы. Для этого следует использовать build-*.sh скрипты.

# Параметры запуска сервера

- env SUPER_MEMBER_KEY - ключ супер пользователя, если задан - то в каждую комнату при ее создании добавляется супер
  пользователь с заданным ключем, используется для подключения плагинов в режиме локальной разработки.