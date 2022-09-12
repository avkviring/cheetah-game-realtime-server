using System;
using Cheetah.Matches.Realtime.EmbeddedServer.FFI;
using Cheetah.Matches.Realtime.EmbeddedServer.Impl;

namespace Cheetah.Matches.Realtime.EmbeddedServer.API
{
    /// <summary>
    /// Класс для запуска встроенного realtime сервера, в основном используется для тестов.
    /// Однако может использоваться и для production целей.
    ///
    /// Порядок использования:
    ///  - создаем экземпляр сервера (можно один сервер, на один тест)
    ///  - создаем комнату
    ///  - создаем необходимое количество пользователей в команте
    ///  - соединяемся с сервером от имени клиента(ов) для тестирования
    ///  - удаляем сервер для освобождения ресурсов
    /// </summary>
    public class EmbeddedServer
    {
        private readonly Server.Description description;

        public EmbeddedServer()
        {
            var code = Server.RunNewServer(ref description);
            if (code != Server.ResultCode.Ok)
            {
                throw new Exception("Cannot run embedded server. Error code " + code);
            }
        }

        public ServerRoom CreateRoom()
        {
            ulong roomId = 0;
            Utils.CheckResult(Room.CreateRoom(description.id, ref roomId));
            return new ServerRoomImpl(description, roomId);
        }

        public void Destroy()
        {
            Utils.CheckResult(Server.DestroyServer(description.id));
        }

        public string GetGameHost()
        {
            unsafe
            {
                return
                    $"{description.serverIp[0]}.{description.serverIp[1]}.{description.serverIp[2]}.{description.serverIp[3]}";
            }
        }

        public uint GetGamePort()
        {
            return description.gamePort;
        }
    }
}