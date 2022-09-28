using System;
using AOT;
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
        private static string errorMessage;

        public EmbeddedServer()
        {
            if (Server.RunNewServer(ref description, OnError))
            {
                throw new Exception("Cannot run embedded server. " + errorMessage);
            }
        }

        public ServerRoom CreateRoom()
        {
            ulong roomId = 0;
            if (!Room.CreateRoom(description.id, ref roomId, OnError))
            {
                throw new Exception("Cannot create room. " + errorMessage);
            }

            return new ServerRoomImpl(description, roomId);
        }

        public void Destroy()
        {
            if (!Server.DestroyServer(description.id))
            {
                throw new Exception("Embedded server not found");
            }
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

        [MonoPInvokeCallback(typeof(Server.OnServerError))]
        private static void OnError(string message)
        {
            errorMessage = message;
        }
    }
}