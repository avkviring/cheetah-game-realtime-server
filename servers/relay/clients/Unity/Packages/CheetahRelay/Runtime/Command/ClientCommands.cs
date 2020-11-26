using System.Runtime.InteropServices;

namespace CheetahRelay
{
    public static class ClientCommands
    {
        /// <summary>
        /// Создать клиента и установить соединение с сервером.
        /// Вновь созданный клиент устанавливается в качестве клиента по-умолчанию.
        /// </summary>
        /// <param name="serverAddress"></param>
        /// <param name="userPublicKey"></param>
        /// <param name="userPrivateKey"></param>
        /// <param name="clientId">Результат - локальный идентификатор клиента</param>
        /// <returns>false - ошибка создания сетевего сокета</returns>
        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "create_client")]
        public static extern bool CreateClient(
            [MarshalAs(UnmanagedType.LPStr)] string serverAddress,
            uint userPublicKey,
            ref RelayBuffer userPrivateKey,
            out ushort clientId
        );

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void OnClientCreate(ushort client);

        [UnmanagedFunctionPointer(CallingConvention.Cdecl)]
        public delegate void OnError();


        /// <summary>
        /// Установить текущего клиента.
        /// Используется в основном для тестов, так как CreateClient также вызывает данный метод
        /// </summary>
        /// <param name="client"></param>
        /// <returns>true - успешно, false - клиента с таким идентификатором не найден</returns>
        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "set_current_client")]
        public static extern bool SetCurrentClient(ushort client);


        /// <summary>
        ///  Получить статус сетевого соединения с сервером
        /// </summary>
        /// <param name="status"> результат</param>
        /// <returns>true - успешно, false - клиента с таким идентификатором не найден</returns>
        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "get_connection_status")]
        public static extern bool GetConnectionStatus(out ConnectionStatus status);
        

        /// <summary>
        /// Обработать входящие команды
        /// </summary>
        /// <returns>false - клиент не найден</returns>
        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "receive")]
        public static extern bool Receive();


        /// <summary>
        /// Разорвать соединение и удалить клиента
        /// </summary>
        /// <returns>false - клиент не найден</returns>
        [DllImport(dllName: Const.Library, CallingConvention = CallingConvention.Cdecl, EntryPoint = "destroy_client")]
        public static extern bool DestroyClient();
    }
}