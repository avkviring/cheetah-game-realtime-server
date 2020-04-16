/// события одного игрового цикла
/// накапливаем изменения
/// и когда настанет время - отправляем их клиентам
pub struct EventsCollector {}

impl EventsCollector {

}

impl Default for EventsCollector {
	fn default() -> Self {
		return EventsCollector {};
	}
}