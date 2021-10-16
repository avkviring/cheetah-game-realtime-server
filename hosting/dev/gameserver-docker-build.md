# Сборка docker образов локально

```bash
cd server
./build-images.sh
```

Если компиляция завершается с ошибкой `(signal: 9, SIGKILL: kill)` нужно добавить памяти в docker desktop (только MacOS)
и/или временно закомментировать неиспользуемые сервера в `images.yaml`.

После успешного выполнения команды проверьте docker образы командой `docker images`.
