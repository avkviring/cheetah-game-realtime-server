# Формат кодирования фреймов

## Требования

## Контекст и постановка проблемы

Есть frame для обмена данными между клиентом и сервером, необходимо разработать формат передачи для минимизации трафика.
На данный момент используется сжатие, однако, как кажется (гипотеза) упаковка данных на основе знания их структуры
должна быть более эффективна.

- максимальный размер фрейма - 500 байт.
- криптография добавляет 25 байт
- команды создания объекта повторяют идентификатор объекта
- также каждый раз повторяется тип канала, хотя меняется он крайне редко

### Состав фрейма

- FrameId - идентификатор фрейма
    - каждый клиент тратит 60 фреймов в секунду
    - id фрейма не должен повториться за сессию (обсуждаемо, netcode к примеру это допускает)
    - u24 скорее всего хватит :-)
- Headers - управляющие заголовки
    - 7 типов (будет незначительно больше)
    - заголовков обычно мало (1-10)
- Commands - команды
    - на данный момент 30 штук, будет больше
    - направление S2C,C2S
    - объектные команды (самые частые)
        - тип канала
        - создатель команды
        - id объекта
            - id: u32
            - owner
                - room
                - user
                    - id: u16
        - id:u16 - поля

## Решение

### Сохраняем изменения (дельты)

Считаем что некоторые данные редко меняется между командами одного фрейма. Особенно редко они меняются при создании 
и загрузке объектов. Пример таких данных:

- ObjectId
- FieldId
- Creator
- Channel (8 вариантов)
    - Sequence
    - Group
- Command

То есть нам не надо сохранять ObjectId для каждой команды, достаточно для каждой команды указать использует она свой
object_id или может использовать object_id из предыдущей команды.

### Используем битовое кодирование для общей информации

- channel_type(u4) - тип канала
- command_type(u6) - тип команды
- creator(u4)
    - 00 - используется текущий
    - 01 - равен владельцу игрового объекта
    - 11 - используется новый
- если следующее поле равно 1 - то необходимо вычитать данные сразу же после флага:
    - object(u1)
    - field(u1)
    - channel_group(u1)

### Плюсы

- эффективно работает на командах создания объекта
- даст экономию на field_id для пакетов синхронизации (если field_id будет один)

### Минусы

- для посылки команды синхронизации состояния возможно потребуется сортировка команд по-пользователям


