Отображение сетевых команд для отладки взаимодействия между клиентом и сервером.

- по-умолчанию трассировка отключена
- трассировка осуществляется в INFO канал серверной системы логирования (по-умолчания для сервера включен режим ERROR)

### Включение трассировки всех команд на сервере

```bash
server --log-level INFO --trace-all-network-commands
```

### Настройка правил трассировки

Для более детальной трассировки используется yaml файл с описанием правил.

```bash
server --command-trace config/tracer-config.yml --log-level INFO
```


```yaml
# разрешение на показ трейса по-умолчанию
default: allow | deny 

# правила для отображения трейсов
# применяется по-порядку, как только правило применимо к команде - процесс обработки прекращается
# поле action - обязательное, остальные нет
# если поле не задано - оно игнорируется при проверки
rules:
  - action: allow | deny
    user: some_user_key
    direction: sc | cs
    field_id: 1
    field_type: long | float | structure | event
    command: Create | Created | SetLong |IncrementLongValue | CompareAndSetLongValue | SetFloat | IncrementFloatValue | SetStruct | Event | Delete | AttachToRoom
```

